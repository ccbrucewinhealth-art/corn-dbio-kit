use anyhow::{anyhow, Result};
use futures_util::TryStreamExt;
use mysql::prelude::Queryable;
use std::collections::HashSet;
use tiberius::{AuthMethod, Client as MssqlClient, ColumnData, Config as MssqlConfig, QueryItem};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::{Cli, DbType};

#[derive(Debug, Clone)]
pub(crate) struct TableStat {
    pub name: String,
    pub rows: usize,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<String>>>,
}

pub(crate) enum DbConnection {
    Mssql(MssqlRuntimeConnection),
    Mysql(mysql::Conn),
    Postgresql(postgres::Client),
    Oracle(OracleRuntimeConnection),
    SqlLite(rusqlite::Connection),
}

pub(crate) struct MssqlRuntimeConnection {
    runtime: tokio::runtime::Runtime,
    client: MssqlClient<Compat<TcpStream>>,
}

pub(crate) struct OracleRuntimeConnection {
    runtime: tokio::runtime::Runtime,
    conn: oracle_rs::Connection,
}

impl DbConnection {
    pub(crate) fn connect(cli: &Cli) -> Result<Self> {
        match cli.db_type {
            DbType::Mssql => connect_mssql(cli),
            DbType::Mysql => connect_mysql(cli),
            DbType::Postgresql => connect_postgresql(cli),
            DbType::Oracle => connect_oracle(cli),
            DbType::SqlLite => Ok(Self::SqlLite(rusqlite::Connection::open(&cli.db_name)?)),
        }
    }

    pub(crate) fn test(&mut self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn execute(&mut self, sql: &str) -> Result<()> {
        match self {
            Self::Mssql(conn) => conn.execute(sql),
            Self::Mysql(conn) => {
                conn.query_drop(sql)?;
                Ok(())
            }
            Self::Postgresql(conn) => {
                conn.batch_execute(sql)?;
                Ok(())
            }
            Self::Oracle(conn) => {
                conn.runtime.block_on(async {
                    conn.conn.execute(sql, &[]).await?;
                    conn.conn.commit().await?;
                    Ok::<(), anyhow::Error>(())
                })?;
                Ok(())
            }
            Self::SqlLite(conn) => {
                conn.execute_batch(sql)?;
                Ok(())
            }
        }
    }

    pub(crate) fn query(&mut self, sql: &str) -> Result<QueryResult> {
        match self {
            Self::Mssql(conn) => conn.query(sql),
            Self::Mysql(conn) => query_mysql(conn, sql),
            Self::Postgresql(conn) => query_postgresql(conn, sql),
            Self::Oracle(conn) => query_oracle(conn, sql),
            Self::SqlLite(conn) => query_sqlite(conn, sql),
        }
    }
}

fn connect_mssql(cli: &Cli) -> Result<DbConnection> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()?;
    let mut config = MssqlConfig::new();
    config.host(&cli.host);
    config.port(cli.port);
    config.database(&cli.db_name);
    config.authentication(AuthMethod::sql_server(&cli.user, &cli.password));
    config.trust_cert();

    let client = runtime.block_on(async {
        let tcp = TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        MssqlClient::connect(config, tcp.compat_write()).await.map_err(anyhow::Error::from)
    })?;
    let mut conn = MssqlRuntimeConnection { runtime, client };
    if !cli.export_collation.trim().is_empty() {
        let col = cli.export_collation.trim().replace('"', "").replace('"', "");
        conn.execute(&format!("SELECT N'' COLLATE {col}"))?;
        conn.execute(&format!("EXEC sp_set_session_context @key=N'DB2SQL_COLLATION', @value=N'{}'", crate::escape_sql_value(&col)))?;
    }
    Ok(DbConnection::Mssql(conn))
}

fn sql_string_literal(db_type: DbType, value: &str) -> String {
    let esc = crate::escape_sql_value(value);
    match db_type {
        DbType::Mssql => format!("N'{}'", esc),
        _ => format!("'{}'", esc),
    }
}

fn mssql_collation_expr(cli: &Cli, expr: &str) -> String {
    if cli.db_type == DbType::Mssql && !cli.export_collation.trim().is_empty() {
        format!("{expr} COLLATE {}", cli.export_collation.trim())
    } else {
        expr.to_string()
    }
}

fn connect_mysql(cli: &Cli) -> Result<DbConnection> {
    let opts = mysql::OptsBuilder::new()
        .ip_or_hostname(Some(cli.host.clone()))
        .tcp_port(cli.port)
        .user(Some(cli.user.clone()))
        .pass(Some(cli.password.clone()))
        .db_name(Some(cli.db_name.clone()));
    Ok(DbConnection::Mysql(mysql::Conn::new(opts)?))
}

fn connect_postgresql(cli: &Cli) -> Result<DbConnection> {
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        cli.host, cli.port, cli.user, cli.password, cli.db_name
    );
    Ok(DbConnection::Postgresql(postgres::Client::connect(
        &conn_str,
        postgres::NoTls,
    )?))
}

fn connect_oracle(cli: &Cli) -> Result<DbConnection> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let config = oracle_rs::Config::new(
        cli.host.clone(),
        cli.port,
        cli.db_name.clone(),
        cli.user.clone(),
        cli.password.clone(),
    );
    let conn = runtime.block_on(oracle_rs::Connection::connect_with_config(config))?;
    Ok(DbConnection::Oracle(OracleRuntimeConnection { runtime, conn }))
}

impl MssqlRuntimeConnection {
    fn execute(&mut self, sql: &str) -> Result<()> {
        self.runtime.block_on(async {
            self.client.execute(sql, &[]).await?;
            if !sql.trim_start().to_ascii_uppercase().starts_with("SELECT") {
                self.client.simple_query("COMMIT TRANSACTION").await?;
            }
            Ok(())
        })
    }

    fn query(&mut self, sql: &str) -> Result<QueryResult> {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.runtime.block_on(async {
                let mut stream = self.client.simple_query(sql).await?;
                let mut columns = Vec::<String>::new();
                let mut rows = Vec::<Vec<Option<String>>>::new();
                while let Some(item) = stream.try_next().await? {
                    if let QueryItem::Row(row) = item {
                        if columns.is_empty() {
                            columns = row
                                .columns()
                                .iter()
                                .map(|c| c.name().to_string())
                                .collect::<Vec<_>>();
                        }
                        let mut out_row = Vec::<Option<String>>::with_capacity(columns.len());
                        for idx in 0..columns.len() {
                            out_row.push(mssql_cell_to_string(&row, idx));
                        }
                        rows.push(out_row);
                    }
                }
                Ok(QueryResult { columns, rows })
            })
        }));
        match r {
            Ok(v) => v,
            Err(_) => Err(anyhow!(
                "mssql query panic (likely unsupported sql_variant / SSVariant type in tiberius). Please CAST sql_variant columns to NVARCHAR in query"
            )),
        }
    }
}

fn mssql_cell_to_string(row: &tiberius::Row, idx: usize) -> Option<String> {
    row.cells().nth(idx).and_then(|(_, data)| column_data_to_string(data))
}

fn column_data_to_string(data: &ColumnData<'static>) -> Option<String> {
    match data {
        ColumnData::U8(v) => v.map(|v| v.to_string()),
        ColumnData::I16(v) => v.map(|v| v.to_string()),
        ColumnData::I32(v) => v.map(|v| v.to_string()),
        ColumnData::I64(v) => v.map(|v| v.to_string()),
        ColumnData::F32(v) => v.map(|v| v.to_string()),
        ColumnData::F64(v) => v.map(|v| v.to_string()),
        ColumnData::Bit(v) => v.map(|v| v.to_string()),
        ColumnData::String(v) => v.as_ref().map(|v| v.to_string()),
        ColumnData::Guid(v) => v.map(|v| v.to_string()),
        ColumnData::Binary(v) => v.as_ref().map(|v| bytes_to_hex(v.as_ref())),
        ColumnData::Numeric(v) => v.as_ref().map(|v| v.to_string()),
        ColumnData::Xml(v) => v.as_ref().map(|v| v.as_ref().clone().into_string()),
        ColumnData::DateTime(v) => v.map(|v| format!("{v:?}")),
        ColumnData::SmallDateTime(v) => v.map(|v| format!("{v:?}")),
        ColumnData::Time(v) => v.map(|v| format!("{v:?}")),
        ColumnData::Date(v) => v.map(|v| format!("{v:?}")),
        ColumnData::DateTime2(v) => v.map(|v| format!("{v:?}")),
        ColumnData::DateTimeOffset(v) => v.map(|v| format!("{v:?}")),
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

fn query_mysql(conn: &mut mysql::Conn, sql: &str) -> Result<QueryResult> {
    let result = conn.query_iter(sql)?;
    let columns = result
        .columns()
        .as_ref()
        .iter()
        .map(|c| c.name_str().to_string())
        .collect::<Vec<_>>();
    let mut rows = Vec::<Vec<Option<String>>>::new();
    for row in result {
        let row = row?;
        let mut out_row = Vec::<Option<String>>::with_capacity(columns.len());
        for idx in 0..columns.len() {
            out_row.push(row.as_ref(idx).map(mysql_value_to_string));
        }
        rows.push(out_row);
    }
    Ok(QueryResult { columns, rows })
}

fn mysql_value_to_string(v: &mysql::Value) -> String {
    match v {
        mysql::Value::NULL => String::new(),
        mysql::Value::Bytes(b) => String::from_utf8_lossy(b).to_string(),
        mysql::Value::Int(v) => v.to_string(),
        mysql::Value::UInt(v) => v.to_string(),
        mysql::Value::Float(v) => v.to_string(),
        mysql::Value::Double(v) => v.to_string(),
        mysql::Value::Date(y, m, d, hh, mm, ss, micros) => format!(
            "{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}.{micros:06}"
        ),
        mysql::Value::Time(neg, days, hh, mm, ss, micros) => format!(
            "{}{} {hh:02}:{mm:02}:{ss:02}.{micros:06}",
            if *neg { "-" } else { "" },
            days
        ),
    }
}

fn query_postgresql(conn: &mut postgres::Client, sql: &str) -> Result<QueryResult> {
    let rows_pg = conn.query(sql, &[])?;
    let columns = rows_pg
        .first()
        .map(|row| row.columns().iter().map(|c| c.name().to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    let mut rows = Vec::<Vec<Option<String>>>::new();
    for row in rows_pg {
        let mut out_row = Vec::<Option<String>>::with_capacity(row.len());
        for idx in 0..row.len() {
            out_row.push(pg_cell_to_string(&row, idx));
        }
        rows.push(out_row);
    }
    Ok(QueryResult { columns, rows })
}

fn query_oracle(conn: &mut OracleRuntimeConnection, sql: &str) -> Result<QueryResult> {
    conn.runtime.block_on(async {
        let result = conn.conn.query(sql, &[]).await?;
        let columns = result
            .columns
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>();
        let rows = result
            .rows
            .iter()
            .map(|row| {
                (0..columns.len())
                    .map(|idx| oracle_cell_to_string(row, idx))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Ok(QueryResult { columns, rows })
    })
}

fn oracle_cell_to_string(row: &oracle_rs::Row, idx: usize) -> Option<String> {
    row.get(idx).and_then(|v| {
        if v.is_null() {
            None
        } else {
            Some(v.to_string())
        }
    })
}

fn pg_cell_to_string(row: &postgres::Row, idx: usize) -> Option<String> {
    row.try_get::<usize, Option<String>>(idx).ok().flatten()
        .or_else(|| row.try_get::<usize, Option<&str>>(idx).ok().flatten().map(|v| v.to_string()))
        .or_else(|| row.try_get::<usize, Option<i64>>(idx).ok().flatten().map(|v| v.to_string()))
        .or_else(|| row.try_get::<usize, Option<i32>>(idx).ok().flatten().map(|v| v.to_string()))
        .or_else(|| row.try_get::<usize, Option<i16>>(idx).ok().flatten().map(|v| v.to_string()))
        .or_else(|| row.try_get::<usize, Option<f32>>(idx).ok().flatten().map(|v| v.to_string()))
        .or_else(|| row.try_get::<usize, Option<f64>>(idx).ok().flatten().map(|v| v.to_string()))
        .or_else(|| row.try_get::<usize, Option<bool>>(idx).ok().flatten().map(|v| v.to_string()))
        .or_else(|| row.try_get::<usize, Option<Vec<u8>>>(idx).ok().flatten().map(|v| bytes_to_hex(&v)))
}

fn query_sqlite(conn: &mut rusqlite::Connection, sql: &str) -> Result<QueryResult> {
    let mut stmt = conn.prepare(sql)?;
    let columns = stmt.column_names().iter().map(|v| v.to_string()).collect::<Vec<_>>();
    let col_count = columns.len();
    let mut rows_sqlite = stmt.query([])?;
    let mut rows = Vec::<Vec<Option<String>>>::new();
    while let Some(row) = rows_sqlite.next()? {
        let mut out_row = Vec::<Option<String>>::with_capacity(col_count);
        for idx in 0..col_count {
            let value = match row.get_ref(idx)? {
                rusqlite::types::ValueRef::Null => None,
                rusqlite::types::ValueRef::Integer(v) => Some(v.to_string()),
                rusqlite::types::ValueRef::Real(v) => Some(v.to_string()),
                rusqlite::types::ValueRef::Text(v) => Some(String::from_utf8_lossy(v).to_string()),
                rusqlite::types::ValueRef::Blob(v) => Some(format!("<{} bytes blob>", v.len())),
            };
            out_row.push(value);
        }
        rows.push(out_row);
    }
    Ok(QueryResult { columns, rows })
}

pub(crate) fn list_objects_sql(db_type: DbType) -> &'static str {
    match db_type {
        DbType::Oracle => {
            "SELECT object_name FROM (SELECT table_name AS object_name FROM user_tables UNION ALL SELECT view_name AS object_name FROM user_views) t ORDER BY object_name"
        }
        DbType::SqlLite => {
            "SELECT name FROM sqlite_master WHERE type IN ('table','view') AND name NOT LIKE 'sqlite_%' ORDER BY name"
        }
        _ => {
            "SELECT table_name FROM information_schema.tables WHERE table_type IN ('BASE TABLE', 'VIEW') ORDER BY table_name"
        }
    }
}

pub(crate) fn list_tables_only_sql(db_type: DbType) -> &'static str {
    match db_type {
        DbType::Mssql => {
            "SELECT table_name FROM information_schema.tables WHERE table_type='BASE TABLE' ORDER BY table_name"
        }
        DbType::Mysql => {
            "SELECT table_name FROM information_schema.tables WHERE table_schema = DATABASE() AND table_type='BASE TABLE' ORDER BY table_name"
        }
        DbType::Postgresql => {
            "SELECT table_name FROM information_schema.tables WHERE table_schema = current_schema() AND table_type='BASE TABLE' ORDER BY table_name"
        }
        DbType::Oracle => "SELECT table_name FROM user_tables ORDER BY table_name",
        DbType::SqlLite => {
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
        }
    }
}

pub(crate) fn list_views_only_sql(db_type: DbType) -> &'static str {
    match db_type {
        DbType::Mssql => {
            "SELECT table_name FROM information_schema.tables WHERE table_type='VIEW' ORDER BY table_name"
        }
        DbType::Mysql => {
            "SELECT table_name FROM information_schema.views WHERE table_schema = DATABASE() ORDER BY table_name"
        }
        DbType::Postgresql => {
            "SELECT table_name FROM information_schema.views WHERE table_schema = current_schema() ORDER BY table_name"
        }
        DbType::Oracle => "SELECT view_name FROM user_views ORDER BY view_name",
        DbType::SqlLite => "SELECT name FROM sqlite_master WHERE type='view' ORDER BY name",
    }
}

pub(crate) fn list_stored_procedures_sql(db_type: DbType) -> &'static str {
    match db_type {
        DbType::Mssql => "SELECT name FROM sys.procedures ORDER BY name",
        DbType::Mysql => {
            "SELECT routine_name FROM information_schema.routines WHERE routine_schema = DATABASE() AND routine_type='PROCEDURE' ORDER BY routine_name"
        }
        DbType::Postgresql => {
            "SELECT routine_name FROM information_schema.routines WHERE specific_schema = current_schema() AND routine_type='PROCEDURE' ORDER BY routine_name"
        }
        DbType::Oracle => {
            "SELECT object_name FROM user_procedures WHERE object_type='PROCEDURE' ORDER BY object_name"
        }
        DbType::SqlLite => "SELECT '' WHERE 1=0",
    }
}

pub(crate) fn execute_sql_preview(conn: &mut DbConnection, sql: &str, max_rows: usize) -> Result<Vec<String>> {
    let result = conn.query(sql)?;
    let mut out = Vec::<String>::new();
    if result.columns.is_empty() {
        out.push("ok: SQL executed (no result set)".to_string());
        return Ok(out);
    }
    out.push(format!("cols: {}", result.columns.join(" | ")));
    for (idx, row) in result.rows.into_iter().enumerate() {
        if idx >= max_rows {
            out.push(format!("... truncated at {max_rows} rows"));
            break;
        }
        out.push(row.into_iter().map(|v| v.unwrap_or_else(|| "NULL".to_string())).collect::<Vec<_>>().join(" | "));
    }
    Ok(out)
}

pub(crate) fn tui_test_connect(cli: &Cli) -> Result<()> {
    DbConnection::connect(cli)?.test()
}

pub(crate) fn tui_list_tables(cli: &Cli) -> Result<Vec<String>> {
    let mut conn = DbConnection::connect(cli)?;
    query_first_column_as_vec(&mut conn, list_tables_only_sql(cli.db_type))
}

pub(crate) fn tui_list_views(cli: &Cli) -> Result<Vec<String>> {
    let mut conn = DbConnection::connect(cli)?;
    query_first_column_as_vec(&mut conn, list_views_only_sql(cli.db_type))
}

pub(crate) fn tui_list_stored_procedures(cli: &Cli) -> Result<Vec<String>> {
    let mut conn = DbConnection::connect(cli)?;
    query_first_column_as_vec(&mut conn, list_stored_procedures_sql(cli.db_type))
}

pub(crate) fn tui_exec_sql(cli: &Cli, sql: &str, max_rows: usize) -> Result<Vec<String>> {
    let mut conn = DbConnection::connect(cli)?;
    execute_sql_preview(&mut conn, sql, max_rows)
}

fn sql_quote_literal(raw: &str) -> String {
    format!("'{}'", raw.replace('"', "\"").replace('\'', "''"))
}

fn auto_increment_pk_columns_sql(db_type: DbType, table: &str) -> String {
    let table_lit = sql_quote_literal(table);
    match db_type {
        DbType::Mssql => format!(
            "SELECT c.name FROM sys.tables t JOIN sys.columns c ON c.object_id = t.object_id JOIN sys.index_columns ic ON ic.object_id = t.object_id AND ic.column_id = c.column_id JOIN sys.indexes i ON i.object_id = t.object_id AND i.index_id = ic.index_id WHERE t.name = {table_lit} AND i.is_primary_key = 1 AND c.is_identity = 1"
        ),
        DbType::Mysql => format!(
            "SELECT column_name FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = {table_lit} AND column_key = 'PRI' AND extra LIKE '%auto_increment%'"
        ),
        DbType::Postgresql => format!(
            "SELECT kcu.column_name FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema AND tc.table_name = kcu.table_name JOIN information_schema.columns c ON c.table_schema = kcu.table_schema AND c.table_name = kcu.table_name AND c.column_name = kcu.column_name WHERE tc.constraint_type = 'PRIMARY KEY' AND tc.table_schema = current_schema() AND tc.table_name = {table_lit} AND (c.is_identity = 'YES' OR c.column_default LIKE 'nextval(%')"
        ),
        DbType::Oracle => format!(
            "SELECT ucc.column_name FROM user_constraints uc JOIN user_cons_columns ucc ON uc.constraint_name = ucc.constraint_name JOIN user_tab_identity_cols uic ON uic.table_name = ucc.table_name AND uic.column_name = ucc.column_name WHERE uc.constraint_type = 'P' AND uc.table_name = UPPER({table_lit})"
        ),
        DbType::SqlLite => format!(
            "SELECT name FROM pragma_table_info({table_lit}) WHERE pk > 0 AND UPPER(type) LIKE '%INT%'"
        ),
    }
}

pub(crate) fn auto_increment_pk_column_set(conn: &mut DbConnection, db_type: DbType, table: &str) -> Result<HashSet<String>> {
    let sql = auto_increment_pk_columns_sql(db_type, table);
    let cols = query_first_column_as_vec(conn, &sql)?;
    Ok(cols.into_iter().map(|c| c.to_ascii_lowercase()).collect::<HashSet<_>>())
}

pub(crate) fn query_first_column_as_vec(conn: &mut DbConnection, sql: &str) -> Result<Vec<String>> {
    let result = conn.query(sql)?;
    Ok(result
        .rows
        .into_iter()
        .filter_map(|row| row.into_iter().next().flatten())
        .filter(|item| !item.trim().is_empty())
        .collect::<Vec<_>>())
}

fn query_table_stats_rows(conn: &mut DbConnection, sql: &str) -> Result<Vec<TableStat>> {
    let result = conn.query(sql)?;
    Ok(result
        .rows
        .into_iter()
        .filter_map(|row| {
            let name = row.first().and_then(|v| v.clone()).unwrap_or_default();
            if name.trim().is_empty() {
                return None;
            }
            let rows = row.get(1).and_then(|v| v.as_ref()).and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
            let size_bytes = row.get(2).and_then(|v| v.as_ref()).and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
            Some(TableStat { name, rows, size_bytes })
        })
        .collect::<Vec<_>>())
}

pub(crate) fn list_table_stats(cli: &Cli) -> Result<Vec<TableStat>> {
    let mut conn = DbConnection::connect(cli)?;
    match cli.db_type {
        DbType::Mssql => {
            let sql = "SELECT t.name, CAST(SUM(p.rows) AS BIGINT) AS row_count, CAST(SUM(a.total_pages) * 8192 AS BIGINT) AS size_bytes FROM sys.tables t JOIN sys.indexes i ON t.object_id=i.object_id JOIN sys.partitions p ON i.object_id=p.object_id AND i.index_id=p.index_id JOIN sys.allocation_units a ON p.partition_id=a.container_id WHERE i.index_id IN (0,1) GROUP BY t.name ORDER BY t.name";
            query_table_stats_rows(&mut conn, sql)
        }
        _ => {
            let names = query_first_column_as_vec(&mut conn, list_tables_only_sql(cli.db_type))?;
            Ok(names.into_iter().map(|name| TableStat { name, rows: 0, size_bytes: 0 }).collect::<Vec<_>>())
        }
    }
}

pub(crate) fn fetch_table_preview(cli: &Cli, table: &str, limit: usize) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let mut conn = DbConnection::connect(cli)?;
    let qt = crate::quote_ident(cli.db_type, table);
    let sql = match cli.db_type {
        DbType::Mssql => format!("SELECT TOP {limit} * FROM {qt}"),
        DbType::Mysql | DbType::Postgresql | DbType::SqlLite => format!("SELECT * FROM {qt} LIMIT {limit}"),
        DbType::Oracle => format!("SELECT * FROM {qt} FETCH FIRST {limit} ROWS ONLY"),
    };
    let result = conn.query(&sql)?;
    Ok((result.columns, result_rows_to_strings(result.rows)))
}

pub(crate) fn fetch_table_preview_page(cli: &Cli, table: &str, page_no: usize, page_size: usize) -> Result<(Vec<String>, Vec<Vec<String>>, String)> {
    let mut conn = DbConnection::connect(cli)?;
    let qt = crate::quote_ident(cli.db_type, table);
    let offset = page_no.saturating_sub(1) * page_size;
    let sql = match cli.db_type {
        DbType::Mssql => format!("SELECT * FROM {qt} ORDER BY 1 OFFSET {offset} ROWS FETCH NEXT {page_size} ROWS ONLY"),
        DbType::Mysql | DbType::Postgresql | DbType::SqlLite => format!("SELECT * FROM {qt} ORDER BY 1 LIMIT {page_size} OFFSET {offset}"),
        DbType::Oracle => format!("SELECT * FROM {qt} ORDER BY 1 OFFSET {offset} ROWS FETCH NEXT {page_size} ROWS ONLY"),
    };
    let result = conn.query(&sql)?;
    Ok((result.columns, result_rows_to_strings(result.rows), sql))
}

fn result_rows_to_strings(rows: Vec<Vec<Option<String>>>) -> Vec<Vec<String>> {
    rows.into_iter().map(|row| row.into_iter().map(|v| v.unwrap_or_default()).collect::<Vec<_>>()).collect::<Vec<_>>()
}

pub(crate) fn fetch_table_columns_info(cli: &Cli, table: &str) -> Result<Vec<(String, String, String)>> {
    let mut conn = DbConnection::connect(cli)?;
    let table_lit = format!("'{}'", crate::escape_sql_value(table));
    let sql = match cli.db_type {
        DbType::Mssql => format!("SELECT c.COLUMN_NAME, c.DATA_TYPE, ISNULL(ep.value, '') AS column_desc FROM INFORMATION_SCHEMA.COLUMNS c LEFT JOIN sys.columns sc ON sc.object_id = OBJECT_ID(c.TABLE_NAME) AND sc.name = c.COLUMN_NAME LEFT JOIN sys.extended_properties ep ON ep.major_id = sc.object_id AND ep.minor_id = sc.column_id AND ep.name='MS_Description' WHERE c.TABLE_NAME = {table_lit} ORDER BY c.ORDINAL_POSITION"),
        DbType::Mysql => format!("SELECT COLUMN_NAME, COLUMN_TYPE, IFNULL(COLUMN_COMMENT,'') FROM information_schema.columns WHERE table_schema = DATABASE() AND table_name = {table_lit} ORDER BY ORDINAL_POSITION"),
        DbType::Postgresql => format!("SELECT c.column_name, c.data_type, '' FROM information_schema.columns c WHERE c.table_schema = current_schema() AND c.table_name = {table_lit} ORDER BY c.ordinal_position"),
        DbType::Oracle => format!("SELECT utc.COLUMN_NAME, utc.DATA_TYPE, NVL(ucc.COMMENTS,'') FROM user_tab_columns utc LEFT JOIN user_col_comments ucc ON ucc.table_name = utc.table_name AND ucc.column_name = utc.column_name WHERE utc.table_name = UPPER({table_lit}) ORDER BY utc.COLUMN_ID"),
        DbType::SqlLite => format!("SELECT name, type, '' FROM pragma_table_info({table_lit})"),
    };
    let result = conn.query(&sql)?;
    Ok(result.rows.into_iter().filter_map(|row| {
        let name = row.first().and_then(|v| v.clone()).unwrap_or_default();
        if name.is_empty() {
            None
        } else {
            let ty = row.get(1).and_then(|v| v.clone()).unwrap_or_default();
            let desc = row.get(2).and_then(|v| v.clone()).unwrap_or_default();
            Some((name, ty, desc))
        }
    }).collect::<Vec<_>>())
}

pub(crate) fn fetch_db_object_definition(cli: &Cli, name: &str, is_sp: bool) -> Result<String> {
    let mut conn = DbConnection::connect(cli)?;
    let name_lit = format!("'{}'", crate::escape_sql_value(name));
    let sql = match cli.db_type {
        DbType::Mssql => format!("SELECT ISNULL(OBJECT_DEFINITION(OBJECT_ID({name_lit})), '-- no definition found')"),
        DbType::Mysql => if is_sp { format!("SHOW CREATE PROCEDURE {name}") } else { format!("SHOW CREATE VIEW {name}") },
        DbType::Postgresql => if is_sp { "SELECT '-- PostgreSQL stored procedure definition query not configured'".to_string() } else { format!("SELECT pg_get_viewdef({name_lit}::regclass, true)") },
        DbType::Oracle => if is_sp { format!("SELECT TEXT FROM USER_SOURCE WHERE TYPE='PROCEDURE' AND NAME=UPPER({name_lit}) ORDER BY LINE") } else { format!("SELECT TEXT FROM USER_VIEWS WHERE VIEW_NAME=UPPER({name_lit})") },
        DbType::SqlLite => if is_sp { "SELECT '-- sqlite does not support stored procedure'".to_string() } else { format!("SELECT IFNULL(sql, '-- no definition found') FROM sqlite_master WHERE type='view' AND name={name_lit}") },
    };
    let lines = execute_sql_preview(&mut conn, &sql, 400)?;
    if lines.is_empty() { Ok("-- empty definition".to_string()) } else { Ok(lines.join("\n")) }
}

pub(crate) fn update_table_cell(cli: &Cli, table: &str, key_col: &str, key_val: &str, target_col: &str, new_val: &str) -> Result<String> {
    let mut conn = DbConnection::connect(cli)?;
    let qt = crate::quote_ident(cli.db_type, table);
    let qk = crate::quote_ident(cli.db_type, key_col);
    let qc = crate::quote_ident(cli.db_type, target_col);
    let set_val = if new_val.eq_ignore_ascii_case("NULL") { "NULL".to_string() } else { sql_string_literal(cli.db_type, new_val) };
    let key_lit = sql_string_literal(cli.db_type, key_val);
    let lhs = mssql_collation_expr(cli, &qk);
    let rhs = mssql_collation_expr(cli, &key_lit);
    let sql = format!("UPDATE {qt} SET {qc} = {set_val} WHERE {lhs} = {rhs}");
    conn.execute(&sql)?;
    Ok(sql)
}

pub(crate) fn insert_table_row(cli: &Cli, table: &str, cols: &[String], values: &[String]) -> Result<String> {
    if cols.is_empty() || cols.len() != values.len() { return Err(anyhow!("invalid insert payload")); }
    let mut conn = DbConnection::connect(cli)?;
    let qt = crate::quote_ident(cli.db_type, table);
    let col_sql = cols.iter().map(|c| crate::quote_ident(cli.db_type, c)).collect::<Vec<_>>().join(", ");
    let val_sql = values.iter().map(|v| if v.trim().is_empty() || v.eq_ignore_ascii_case("NULL") { "NULL".to_string() } else { sql_string_literal(cli.db_type, v) }).collect::<Vec<_>>().join(", ");
    let sql = format!("INSERT INTO {qt} ({col_sql}) VALUES ({val_sql})");
    conn.execute(&sql)?;
    Ok(sql)
}

pub(crate) fn delete_table_row(cli: &Cli, table: &str, key_col: &str, key_val: &str) -> Result<String> {
    let mut conn = DbConnection::connect(cli)?;
    let qt = crate::quote_ident(cli.db_type, table);
    let qk = crate::quote_ident(cli.db_type, key_col);
    let key_lit = if key_val.trim().is_empty() || key_val.eq_ignore_ascii_case("NULL") { "NULL".to_string() } else { sql_string_literal(cli.db_type, key_val) };
    let sql = if key_lit == "NULL" {
        format!("DELETE FROM {qt} WHERE {} IS NULL", mssql_collation_expr(cli, &qk))
    } else {
        format!("DELETE FROM {qt} WHERE {} = {}", mssql_collation_expr(cli, &qk), mssql_collation_expr(cli, &key_lit))
    };
    conn.execute(&sql)?;
    Ok(sql)
}

pub(crate) fn fetch_rows_by_sql(cli: &Cli, sql: &str, limit: usize) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let mut conn = DbConnection::connect(cli)?;
    let limited_sql = match cli.db_type {
        DbType::Mssql => format!("SELECT TOP {limit} * FROM ({sql}) q"),
        DbType::Mysql | DbType::Postgresql | DbType::SqlLite => format!("SELECT * FROM ({sql}) q LIMIT {limit}"),
        DbType::Oracle => format!("SELECT * FROM ({sql}) q FETCH FIRST {limit} ROWS ONLY"),
    };
    let result = conn.query(&limited_sql)?;
    Ok((result.columns, result_rows_to_strings(result.rows)))
}
