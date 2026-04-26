fn execute_import_with_cancel(
    cli: &Cli,
    _silent: bool,
    cancel: &AtomicBool,
    progress: Option<&Arc<Mutex<RunProgressSnapshot>>>,
) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        return Err(anyhow!("import cancelled"));
    }

    let source_sqlite = cli.output_directory.join("export.sqlite");
    if !source_sqlite.exists() {
        return Err(anyhow!(
            "import source not found: {}",
            source_sqlite.display()
        ));
    }
    if cli.output_format != OutputFormat::SqlLite {
        return Err(anyhow!(
            "import currently supports output-format=sql-lite only"
        ));
    }

    let mut target_cli = cli.clone();
    target_cli.host = cli.import_host.clone();
    target_cli.port = cli.import_port;
    target_cli.db_type = cli.import_db_type;
    target_cli.user = cli.import_user.clone();
    target_cli.password = cli.import_password.clone();
    target_cli.db_name = cli.import_db_name.clone();

    if target_cli.db_type != DbType::SqlLite {
        if target_cli.host.trim().is_empty()
            || target_cli.user.trim().is_empty()
            || target_cli.password.trim().is_empty()
            || target_cli.db_name.trim().is_empty()
        {
            return Err(anyhow!(
                "missing import connection fields: import-host/import-user/import-password/import-db-name"
            ));
        }
    }

    let src = SqliteConnection::open(&source_sqlite)
        .with_context(|| format!("failed to open sqlite source: {}", source_sqlite.display()))?;
    let mut stmt = src.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )?;
    let table_names = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if let Some(p) = progress {
        if let Ok(mut s) = p.lock() {
            s.apply_event(RunProgressEvent {
                total_items: Some(table_names.len()),
                done_items: Some(0),
                current_item: Some(String::new()),
                current_item_rows_done: Some(0),
                current_item_rows_total: Some(0),
                status: Some("running".to_string()),
            });
        }
    }

    let env = Environment::new()?;
    let conn_str = build_connection_string(&target_cli);
    let target_conn = env.connect_with_connection_string(&conn_str, ConnectionOptions::default())?;

    for (table_idx, table) in table_names.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(anyhow!("import cancelled"));
        }

        let qt_src_count = quote_ident(DbType::SqlLite, &table);
        let table_total_rows = src
            .query_row(
                &format!("SELECT COUNT(*) FROM {qt_src_count}"),
                [],
                |row| row.get::<_, i64>(0),
            )
            .ok()
            .and_then(|v| usize::try_from(v).ok())
            .unwrap_or(0);
        let mut imported_rows = 0usize;
        if let Some(p) = progress {
            if let Ok(mut s) = p.lock() {
                s.apply_event(RunProgressEvent {
                    done_items: Some(table_idx),
                    current_item: Some(table.clone()),
                    current_item_rows_done: Some(0),
                    current_item_rows_total: Some(table_total_rows),
                    status: Some("running".to_string()),
                    ..RunProgressEvent::default()
                });
            }
        }

        let qt_target = quote_ident(target_cli.db_type, &table);
        let qt_src = quote_ident(DbType::SqlLite, &table);

        let cols = {
            let mut s = src.prepare(&format!("PRAGMA table_info({qt_src})"))?;
            let rows = s.query_map([], |row| row.get::<_, String>(1))?;
            rows.collect::<std::result::Result<Vec<_>, _>>()?
        };
        if cols.is_empty() {
            continue;
        }
        let quoted_cols_target = cols
            .iter()
            .map(|c| quote_ident(target_cli.db_type, c))
            .collect::<Vec<_>>();

        let insert_prefix = format!(
            "INSERT INTO {qt_target} ({}) VALUES ",
            quoted_cols_target.join(", ")
        );

        let mut sel = src.prepare(&format!("SELECT * FROM {qt_src}"))?;
        let mut rows = sel.query([])?;
        while let Some(row) = rows.next()? {
            if cancel.load(Ordering::Relaxed) {
                return Err(anyhow!("import cancelled"));
            }
            let mut values_sql = Vec::<String>::with_capacity(cols.len());
            for i in 0..cols.len() {
                let text = match row.get_ref(i)? {
                    ValueRef::Null => {
                        values_sql.push("NULL".to_string());
                        continue;
                    }
                    ValueRef::Integer(v) => v.to_string(),
                    ValueRef::Real(v) => v.to_string(),
                    ValueRef::Text(v) => String::from_utf8_lossy(v).to_string(),
                    ValueRef::Blob(v) => {
                        let mut out = String::with_capacity(v.len() * 2);
                        for b in v {
                            out.push_str(&format!("{:02x}", b));
                        }
                        out
                    }
                };
                values_sql.push(format!("'{}'", escape_sql_value(&text)));
            }
            let insert_sql = format!("{}({})", insert_prefix, values_sql.join(","));
            let _ = target_conn.execute(&insert_sql, ())?;
            imported_rows += 1;
            if imported_rows % 200 == 0 {
                if let Some(p) = progress {
                    if let Ok(mut s) = p.lock() {
                        s.apply_event(RunProgressEvent {
                            current_item_rows_done: Some(imported_rows),
                            ..RunProgressEvent::default()
                        });
                    }
                }
            }
        }

        if let Some(p) = progress {
            if let Ok(mut s) = p.lock() {
                s.apply_event(RunProgressEvent {
                    done_items: Some(table_idx + 1),
                    current_item: Some(table.clone()),
                    current_item_rows_done: Some(imported_rows),
                    current_item_rows_total: Some(table_total_rows.max(imported_rows)),
                    status: Some("running".to_string()),
                    ..RunProgressEvent::default()
                });
            }
        }
    }

    if let Some(p) = progress {
        if let Ok(mut s) = p.lock() {
            let total_items = s.total_items;
            s.apply_event(RunProgressEvent {
                done_items: Some(total_items),
                status: Some("done".to_string()),
                ..RunProgressEvent::default()
            });
        }
    }

    Ok(())
}

fn execute_import(cli: &Cli, silent: bool) -> Result<()> {
    let cancel = AtomicBool::new(false);
    execute_import_with_cancel(cli, silent, &cancel, None)
}
