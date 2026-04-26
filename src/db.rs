use anyhow::{anyhow, Result};
use odbc_api::{buffers::TextRowSet, ConnectionOptions, Cursor, Environment, ResultSetMetadata};
use std::collections::HashSet;
use std::env;

use crate::{Cli, DbType};

#[derive(Debug, Clone)]
pub(crate) struct TableStat {
    pub name: String,
    pub rows: usize,
    pub size_bytes: usize,
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

pub(crate) fn execute_sql_preview(
    conn: &odbc_api::Connection<'_>,
    sql: &str,
    max_rows: usize,
) -> Result<Vec<String>> {
    let mut out = Vec::<String>::new();
    if let Some(mut cursor) = conn.execute(sql, ())? {
        let col_count = cursor.num_result_cols()? as usize;
        if col_count == 0 {
            out.push("ok: SQL executed (no result set)".to_string());
            return Ok(out);
        }

        let column_names = (1..=col_count)
            .map(|idx| cursor.col_name(idx as u16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        out.push(format!("cols: {}", column_names.join(" | ")));

        let mut buffers = TextRowSet::for_cursor(200, &mut cursor, Some(8 * 1024))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
        let mut shown = 0usize;

        while let Some(batch) = row_set_cursor.fetch()? {
            for row_index in 0..batch.num_rows() {
                let mut row_vals = Vec::<String>::with_capacity(batch.num_cols());
                for col_index in 0..batch.num_cols() {
                    let v = batch
                        .at_as_str(col_index, row_index)
                        .map_err(|e| anyhow!("utf8 decode failed at row {row_index}, col {col_index}: {e}"))?
                        .unwrap_or("NULL")
                        .to_string();
                    row_vals.push(v);
                }
                out.push(row_vals.join(" | "));
                shown += 1;
                if shown >= max_rows {
                    out.push(format!("... truncated at {max_rows} rows"));
                    return Ok(out);
                }
            }
        }
    } else {
        out.push("ok: SQL executed (empty result)".to_string());
    }
    Ok(out)
}

pub(crate) fn tui_test_connect(cli: &Cli) -> Result<()> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let _conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    Ok(())
}

pub(crate) fn tui_list_tables(cli: &Cli) -> Result<Vec<String>> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    query_first_column_as_vec(&conn, list_tables_only_sql(cli.db_type))
}

pub(crate) fn tui_list_views(cli: &Cli) -> Result<Vec<String>> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    query_first_column_as_vec(&conn, list_views_only_sql(cli.db_type))
}

pub(crate) fn tui_list_stored_procedures(cli: &Cli) -> Result<Vec<String>> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    query_first_column_as_vec(&conn, list_stored_procedures_sql(cli.db_type))
}

pub(crate) fn tui_exec_sql(cli: &Cli, sql: &str, max_rows: usize) -> Result<Vec<String>> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    execute_sql_preview(&conn, sql, max_rows)
}

fn sql_quote_literal(raw: &str) -> String {
    format!("'{}'", raw.replace('"', "\"").replace('\'', "''"))
}

fn auto_increment_pk_columns_sql(db_type: DbType, table: &str) -> String {
    let table_lit = sql_quote_literal(table);
    match db_type {
        DbType::Mssql => format!(
            "SELECT c.name \
             FROM sys.tables t \
             JOIN sys.columns c ON c.object_id = t.object_id \
             JOIN sys.index_columns ic ON ic.object_id = t.object_id AND ic.column_id = c.column_id \
             JOIN sys.indexes i ON i.object_id = t.object_id AND i.index_id = ic.index_id \
             WHERE t.name = {table_lit} AND i.is_primary_key = 1 AND c.is_identity = 1"
        ),
        DbType::Mysql => format!(
            "SELECT column_name \
             FROM information_schema.columns \
             WHERE table_schema = DATABASE() \
               AND table_name = {table_lit} \
               AND column_key = 'PRI' \
               AND extra LIKE '%auto_increment%'"
        ),
        DbType::Postgresql => format!(
            "SELECT kcu.column_name \
             FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu \
               ON tc.constraint_name = kcu.constraint_name \
              AND tc.table_schema = kcu.table_schema \
              AND tc.table_name = kcu.table_name \
             JOIN information_schema.columns c \
               ON c.table_schema = kcu.table_schema \
              AND c.table_name = kcu.table_name \
              AND c.column_name = kcu.column_name \
             WHERE tc.constraint_type = 'PRIMARY KEY' \
               AND tc.table_schema = current_schema() \
               AND tc.table_name = {table_lit} \
               AND (c.is_identity = 'YES' OR c.column_default LIKE 'nextval(%')"
        ),
        DbType::Oracle => format!(
            "SELECT ucc.column_name \
             FROM user_constraints uc \
             JOIN user_cons_columns ucc \
               ON uc.constraint_name = ucc.constraint_name \
             JOIN user_tab_identity_cols uic \
               ON uic.table_name = ucc.table_name \
              AND uic.column_name = ucc.column_name \
             WHERE uc.constraint_type = 'P' \
               AND uc.table_name = UPPER({table_lit})"
        ),
        DbType::SqlLite => format!(
            "SELECT name FROM pragma_table_info({table_lit}) WHERE pk > 0 AND UPPER(type) LIKE '%INT%'"
        ),
    }
}

pub(crate) fn auto_increment_pk_column_set(
    conn: &odbc_api::Connection<'_>,
    db_type: DbType,
    table: &str,
) -> Result<HashSet<String>> {
    let sql = auto_increment_pk_columns_sql(db_type, table);
    let cols = query_first_column_as_vec(conn, &sql)?;
    Ok(cols
        .into_iter()
        .map(|c| c.to_ascii_lowercase())
        .collect::<HashSet<_>>())
}

pub(crate) fn build_connection_string(cli: &Cli) -> String {
    let env_driver = env::var("DB2SQL_ODBC_DRIVER").ok();
    match cli.db_type {
        DbType::Mssql => {
            let driver = env_driver.unwrap_or_else(|| "ODBC Driver 18 for SQL Server".to_string());
            format!(
                "Driver={{{driver}}};Server={},{};Database={};Uid={};Pwd={};TrustServerCertificate=Yes;",
                cli.host, cli.port, cli.db_name, cli.user, cli.password
            )
        }
        DbType::Mysql => {
            let driver = env_driver.unwrap_or_else(|| "MySQL ODBC 8.0 Unicode Driver".to_string());
            format!(
                "Driver={{{driver}}};Server={};Port={};Database={};User={};Password={};",
                cli.host, cli.port, cli.db_name, cli.user, cli.password
            )
        }
        DbType::Postgresql => {
            let driver = env_driver.unwrap_or_else(|| "PostgreSQL Unicode".to_string());
            format!(
                "Driver={{{driver}}};Server={};Port={};Database={};Uid={};Pwd={};",
                cli.host, cli.port, cli.db_name, cli.user, cli.password
            )
        }
        DbType::Oracle => {
            let driver = env_driver.unwrap_or_else(|| "Oracle in OraDB19Home1".to_string());
            format!(
                "Driver={{{driver}}};Dbq=//{}:{}/{};Uid={};Pwd={};",
                cli.host, cli.port, cli.db_name, cli.user, cli.password
            )
        }
        DbType::SqlLite => {
            let driver = env_driver.unwrap_or_else(|| "SQLite3 ODBC Driver".to_string());
            format!("Driver={{{driver}}};Database={};", cli.db_name)
        }
    }
}

pub(crate) fn query_first_column_as_vec(conn: &odbc_api::Connection<'_>, sql: &str) -> Result<Vec<String>> {
    let mut output = Vec::<String>::new();
    if let Some(mut cursor) = conn.execute(sql, ())? {
        let mut buffers = TextRowSet::for_cursor(1_000, &mut cursor, Some(8 * 1024))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
        while let Some(batch) = row_set_cursor.fetch()? {
            for row_index in 0..batch.num_rows() {
                let item = batch
                    .at_as_str(0, row_index)
                    .map_err(|e| anyhow!("utf8 decode failed at row {row_index}: {e}"))?
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                if !item.trim().is_empty() {
                    output.push(item);
                }
            }
        }
    }
    Ok(output)
}

fn query_table_stats_rows(conn: &odbc_api::Connection<'_>, sql: &str) -> Result<Vec<TableStat>> {
    let mut out = Vec::<TableStat>::new();
    if let Some(mut cursor) = conn.execute(sql, ())? {
        let mut buffers = TextRowSet::for_cursor(1_000, &mut cursor, Some(8 * 1024))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
        while let Some(batch) = row_set_cursor.fetch()? {
            for row_index in 0..batch.num_rows() {
                let name = batch
                    .at_as_str(0, row_index)
                    .map_err(|e| anyhow!("utf8 decode failed at row {row_index}, col 0: {e}"))?
                    .unwrap_or("")
                    .to_string();
                if name.trim().is_empty() {
                    continue;
                }
                let rows = batch
                    .at_as_str(1, row_index)
                    .ok()
                    .flatten()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(0);
                let size_bytes = batch
                    .at_as_str(2, row_index)
                    .ok()
                    .flatten()
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(0);
                out.push(TableStat {
                    name,
                    rows,
                    size_bytes,
                });
            }
        }
    }
    Ok(out)
}

pub(crate) fn list_table_stats(cli: &Cli) -> Result<Vec<TableStat>> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;

    match cli.db_type {
        DbType::Mssql => {
            let sql = "SELECT t.name, CAST(SUM(p.rows) AS BIGINT) AS row_count, CAST(SUM(a.total_pages) * 8192 AS BIGINT) AS size_bytes \
                       FROM sys.tables t \
                       JOIN sys.indexes i ON t.object_id=i.object_id \
                       JOIN sys.partitions p ON i.object_id=p.object_id AND i.index_id=p.index_id \
                       JOIN sys.allocation_units a ON p.partition_id=a.container_id \
                       WHERE i.index_id IN (0,1) \
                       GROUP BY t.name \
                       ORDER BY t.name";
            query_table_stats_rows(&conn, sql)
        }
        _ => {
            let names = query_first_column_as_vec(&conn, list_tables_only_sql(cli.db_type))?;
            Ok(names
                .into_iter()
                .map(|name| TableStat {
                    name,
                    rows: 0,
                    size_bytes: 0,
                })
                .collect::<Vec<_>>())
        }
    }
}

pub(crate) fn fetch_table_preview(
    cli: &Cli,
    table: &str,
    limit: usize,
) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    let qt = crate::quote_ident(cli.db_type, table);
    let sql = match cli.db_type {
        DbType::Mssql => format!("SELECT TOP {limit} * FROM {qt}"),
        DbType::Mysql | DbType::Postgresql | DbType::SqlLite => {
            format!("SELECT * FROM {qt} LIMIT {limit}")
        }
        DbType::Oracle => format!("SELECT * FROM {qt} FETCH FIRST {limit} ROWS ONLY"),
    };

    let mut cols = Vec::<String>::new();
    let mut rows = Vec::<Vec<String>>::new();
    if let Some(mut cursor) = conn.execute(&sql, ())? {
        let col_count = cursor.num_result_cols()? as usize;
        cols = (1..=col_count)
            .map(|idx| cursor.col_name(idx as u16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut buffers = TextRowSet::for_cursor(200, &mut cursor, Some(8 * 1024))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
        while let Some(batch) = row_set_cursor.fetch()? {
            for row_index in 0..batch.num_rows() {
                let mut row = Vec::<String>::with_capacity(batch.num_cols());
                for col_index in 0..batch.num_cols() {
                    let v = batch
                        .at_as_str(col_index, row_index)
                        .map_err(|e| anyhow!("utf8 decode failed at row {row_index}, col {col_index}: {e}"))?
                        .unwrap_or("")
                        .to_string();
                    row.push(v);
                }
                rows.push(row);
            }
        }
    }
    Ok((cols, rows))
}

pub(crate) fn fetch_table_preview_page(
    cli: &Cli,
    table: &str,
    page_no: usize,
    page_size: usize,
) -> Result<(Vec<String>, Vec<Vec<String>>, String)> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    let qt = crate::quote_ident(cli.db_type, table);
    let offset = page_no.saturating_sub(1) * page_size;
    let sql = match cli.db_type {
        DbType::Mssql => format!(
            "SELECT * FROM {qt} ORDER BY 1 OFFSET {offset} ROWS FETCH NEXT {page_size} ROWS ONLY"
        ),
        DbType::Mysql | DbType::Postgresql | DbType::SqlLite => {
            format!("SELECT * FROM {qt} ORDER BY 1 LIMIT {page_size} OFFSET {offset}")
        }
        DbType::Oracle => {
            format!("SELECT * FROM {qt} ORDER BY 1 OFFSET {offset} ROWS FETCH NEXT {page_size} ROWS ONLY")
        }
    };

    let mut cols = Vec::<String>::new();
    let mut rows = Vec::<Vec<String>>::new();
    if let Some(mut cursor) = conn.execute(&sql, ())? {
        let col_count = cursor.num_result_cols()? as usize;
        cols = (1..=col_count)
            .map(|idx| cursor.col_name(idx as u16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut buffers = TextRowSet::for_cursor(200, &mut cursor, Some(8 * 1024))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
        while let Some(batch) = row_set_cursor.fetch()? {
            for row_index in 0..batch.num_rows() {
                let mut row = Vec::<String>::with_capacity(batch.num_cols());
                for col_index in 0..batch.num_cols() {
                    let v = batch
                        .at_as_str(col_index, row_index)
                        .map_err(|e| anyhow!("utf8 decode failed at row {row_index}, col {col_index}: {e}"))?
                        .unwrap_or("")
                        .to_string();
                    row.push(v);
                }
                rows.push(row);
            }
        }
    }
    Ok((cols, rows, sql))
}

pub(crate) fn fetch_table_columns_info(cli: &Cli, table: &str) -> Result<Vec<(String, String, String)>> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    let table_lit = format!("'{}'", crate::escape_sql_value(table));
    let sql = match cli.db_type {
        DbType::Mssql => format!(
            "SELECT c.COLUMN_NAME, c.DATA_TYPE, ISNULL(ep.value, '') AS column_desc \
             FROM INFORMATION_SCHEMA.COLUMNS c \
             LEFT JOIN sys.columns sc ON sc.object_id = OBJECT_ID(c.TABLE_NAME) AND sc.name = c.COLUMN_NAME \
             LEFT JOIN sys.extended_properties ep ON ep.major_id = sc.object_id AND ep.minor_id = sc.column_id AND ep.name='MS_Description' \
             WHERE c.TABLE_NAME = {table_lit} ORDER BY c.ORDINAL_POSITION"
        ),
        DbType::Mysql => format!(
            "SELECT COLUMN_NAME, COLUMN_TYPE, IFNULL(COLUMN_COMMENT,'') \
             FROM information_schema.columns \
             WHERE table_schema = DATABASE() AND table_name = {table_lit} ORDER BY ORDINAL_POSITION"
        ),
        DbType::Postgresql => format!(
            "SELECT c.column_name, c.data_type, '' \
             FROM information_schema.columns c \
             WHERE c.table_schema = current_schema() AND c.table_name = {table_lit} ORDER BY c.ordinal_position"
        ),
        DbType::Oracle => format!(
            "SELECT utc.COLUMN_NAME, utc.DATA_TYPE, NVL(ucc.COMMENTS,'') \
             FROM user_tab_columns utc \
             LEFT JOIN user_col_comments ucc ON ucc.table_name = utc.table_name AND ucc.column_name = utc.column_name \
             WHERE utc.table_name = UPPER({table_lit}) ORDER BY utc.COLUMN_ID"
        ),
        DbType::SqlLite => format!("SELECT name, type, '' FROM pragma_table_info({table_lit})"),
    };

    let mut out = Vec::<(String, String, String)>::new();
    if let Some(mut cursor) = conn.execute(&sql, ())? {
        let mut buffers = TextRowSet::for_cursor(500, &mut cursor, Some(8 * 1024))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
        while let Some(batch) = row_set_cursor.fetch()? {
            for row_index in 0..batch.num_rows() {
                let name = batch.at_as_str(0, row_index).ok().flatten().unwrap_or("").to_string();
                let ty = batch.at_as_str(1, row_index).ok().flatten().unwrap_or("").to_string();
                let desc = batch.at_as_str(2, row_index).ok().flatten().unwrap_or("").to_string();
                if !name.is_empty() {
                    out.push((name, ty, desc));
                }
            }
        }
    }
    Ok(out)
}

pub(crate) fn fetch_db_object_definition(cli: &Cli, name: &str, is_sp: bool) -> Result<String> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    let name_lit = format!("'{}'", crate::escape_sql_value(name));
    let sql = match cli.db_type {
        DbType::Mssql => {
            if is_sp {
                format!(
                    "SELECT ISNULL(OBJECT_DEFINITION(OBJECT_ID({name_lit})), '-- no definition found')"
                )
            } else {
                format!(
                    "SELECT ISNULL(OBJECT_DEFINITION(OBJECT_ID({name_lit})), '-- no definition found')"
                )
            }
        }
        DbType::Mysql => {
            if is_sp {
                format!("SHOW CREATE PROCEDURE {name}")
            } else {
                format!("SHOW CREATE VIEW {name}")
            }
        }
        DbType::Postgresql => {
            if is_sp {
                "SELECT '-- PostgreSQL stored procedure definition query not configured'".to_string()
            } else {
                format!("SELECT pg_get_viewdef({name_lit}::regclass, true)")
            }
        }
        DbType::Oracle => {
            if is_sp {
                format!(
                    "SELECT TEXT FROM USER_SOURCE WHERE TYPE='PROCEDURE' AND NAME=UPPER({name_lit}) ORDER BY LINE"
                )
            } else {
                format!("SELECT TEXT FROM USER_VIEWS WHERE VIEW_NAME=UPPER({name_lit})")
            }
        }
        DbType::SqlLite => {
            if is_sp {
                "SELECT '-- sqlite does not support stored procedure'".to_string()
            } else {
                format!(
                    "SELECT IFNULL(sql, '-- no definition found') FROM sqlite_master WHERE type='view' AND name={name_lit}"
                )
            }
        }
    };

    let lines = execute_sql_preview(&conn, &sql, 400)?;
    if lines.is_empty() {
        return Ok("-- empty definition".to_string());
    }
    if lines.len() == 1 {
        return Ok(lines[0].clone());
    }
    Ok(lines.join("\n"))
}

pub(crate) fn update_table_cell(
    cli: &Cli,
    table: &str,
    key_col: &str,
    key_val: &str,
    target_col: &str,
    new_val: &str,
) -> Result<String> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;

    let qt = crate::quote_ident(cli.db_type, table);
    let qk = crate::quote_ident(cli.db_type, key_col);
    let qc = crate::quote_ident(cli.db_type, target_col);
    let set_val = if new_val.eq_ignore_ascii_case("NULL") {
        "NULL".to_string()
    } else {
        format!("'{}'", crate::escape_sql_value(new_val))
    };
    let key_lit = format!("'{}'", crate::escape_sql_value(key_val));
    let sql = format!("UPDATE {qt} SET {qc} = {set_val} WHERE {qk} = {key_lit}");
    let _ = conn.execute(&sql, ())?;
    Ok(sql)
}

pub(crate) fn insert_table_row(cli: &Cli, table: &str, cols: &[String], values: &[String]) -> Result<String> {
    if cols.is_empty() || cols.len() != values.len() {
        return Err(anyhow!("invalid insert payload"));
    }
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;

    let qt = crate::quote_ident(cli.db_type, table);
    let col_sql = cols
        .iter()
        .map(|c| crate::quote_ident(cli.db_type, c))
        .collect::<Vec<_>>()
        .join(", ");
    let val_sql = values
        .iter()
        .map(|v| {
            if v.trim().is_empty() || v.eq_ignore_ascii_case("NULL") {
                "NULL".to_string()
            } else {
                format!("'{}'", crate::escape_sql_value(v))
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("INSERT INTO {qt} ({col_sql}) VALUES ({val_sql})");
    let _ = conn.execute(&sql, ())?;
    Ok(sql)
}

pub(crate) fn delete_table_row(cli: &Cli, table: &str, key_col: &str, key_val: &str) -> Result<String> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;

    let qt = crate::quote_ident(cli.db_type, table);
    let qk = crate::quote_ident(cli.db_type, key_col);
    let key_lit = if key_val.trim().is_empty() || key_val.eq_ignore_ascii_case("NULL") {
        "NULL".to_string()
    } else {
        format!("'{}'", crate::escape_sql_value(key_val))
    };
    let sql = if key_lit == "NULL" {
        format!("DELETE FROM {qt} WHERE {qk} IS NULL")
    } else {
        format!("DELETE FROM {qt} WHERE {qk} = {key_lit}")
    };
    let _ = conn.execute(&sql, ())?;
    Ok(sql)
}

pub(crate) fn fetch_rows_by_sql(cli: &Cli, sql: &str, limit: usize) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let env = Environment::new()?;
    let conn_str = build_connection_string(cli);
    let conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;
    let limited_sql = match cli.db_type {
        DbType::Mssql => format!("SELECT TOP {limit} * FROM ({sql}) q"),
        DbType::Mysql | DbType::Postgresql | DbType::SqlLite => format!("SELECT * FROM ({sql}) q LIMIT {limit}"),
        DbType::Oracle => format!("SELECT * FROM ({sql}) q FETCH FIRST {limit} ROWS ONLY"),
    };
    let mut cols = Vec::<String>::new();
    let mut rows = Vec::<Vec<String>>::new();
    if let Some(mut cursor) = conn.execute(&limited_sql, ())? {
        let col_count = cursor.num_result_cols()? as usize;
        cols = (1..=col_count)
            .map(|idx| cursor.col_name(idx as u16))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut buffers = TextRowSet::for_cursor(200, &mut cursor, Some(8 * 1024))?;
        let mut row_set_cursor = cursor.bind_buffer(&mut buffers)?;
        while let Some(batch) = row_set_cursor.fetch()? {
            for row_index in 0..batch.num_rows() {
                let mut row = Vec::<String>::with_capacity(batch.num_cols());
                for col_index in 0..batch.num_cols() {
                    let v = batch
                        .at_as_str(col_index, row_index)
                        .map_err(|e| anyhow!("utf8 decode failed at row {row_index}, col {col_index}: {e}"))?
                        .unwrap_or("")
                        .to_string();
                    row.push(v);
                }
                rows.push(row);
            }
        }
    }
    Ok((cols, rows))
}
