fn current_unix_ts() -> Result<String> {
    let output = Command::new("date").arg("+%s").output()?;
    if !output.status.success() {
        return Err(anyhow!("failed to get unix timestamp by `date` command"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

struct TempImportDir {
    path: PathBuf,
}

impl TempImportDir {
    fn create() -> Result<Self> {
        let ts = current_unix_ts()?;
        let pid = std::process::id();
        let path = PathBuf::from(format!("/tmp/corn-dbio-kit-import-{ts}-{pid}"));
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create temp import dir: {}", path.display()))?;
        Ok(Self { path })
    }
}

impl Drop for TempImportDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn detect_archive_kind(path: &Path) -> Option<&'static str> {
    let name = path.file_name()?.to_string_lossy().to_ascii_lowercase();
    if name.ends_with(".zip") {
        Some("zip")
    } else if name.ends_with(".tar") {
        Some("tar")
    } else if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") || name.ends_with(".bz2") {
        Some("bzip")
    } else {
        None
    }
}

fn find_first_sqlite_file(dir: &Path) -> Result<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(cur) = stack.pop() {
        for ent in fs::read_dir(&cur)
            .with_context(|| format!("failed to scan dir: {}", cur.display()))?
        {
            let ent = ent?;
            let p = ent.path();
            if p.is_dir() {
                stack.push(p);
                continue;
            }
            let name = p
                .file_name()
                .map(|v| v.to_string_lossy().to_ascii_lowercase())
                .unwrap_or_default();
            if name == "export.sqlite" || name.ends_with(".sqlite") || name.ends_with(".db") {
                return Ok(p);
            }
        }
    }
    Err(anyhow!(
        "no sqlite source found under extracted directory: {}",
        dir.display()
    ))
}

fn resolve_import_source(cli: &Cli, silent: bool) -> Result<(PathBuf, Option<TempImportDir>)> {
    let src = cli.import_content_src.clone();
    if !src.exists() {
        return Err(anyhow!("import source not found: {}", src.display()));
    }

    if let Some(kind) = detect_archive_kind(&src) {
        let tmp = TempImportDir::create()?;
        if !silent {
            println!("[import] extracting archive to {}", tmp.path.display());
        }
        let status = match kind {
            "zip" => Command::new("unzip")
                .arg("-q")
                .arg(&src)
                .arg("-d")
                .arg(&tmp.path)
                .status()?,
            "tar" => Command::new("tar")
                .arg("-xf")
                .arg(&src)
                .arg("-C")
                .arg(&tmp.path)
                .status()?,
            "bzip" => Command::new("tar")
                .arg("-xjf")
                .arg(&src)
                .arg("-C")
                .arg(&tmp.path)
                .status()?,
            _ => return Err(anyhow!("unsupported archive kind")),
        };
        if !status.success() {
            return Err(anyhow!("extract archive failed: {}", src.display()));
        }
        let sqlite = find_first_sqlite_file(&tmp.path)?;
        return Ok((sqlite, Some(tmp)));
    }

    Ok((src, None))
}

fn execute_import_with_cancel(
    cli: &Cli,
    silent: bool,
    cancel: &AtomicBool,
    progress: Option<&Arc<Mutex<RunProgressSnapshot>>>,
) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        return Err(anyhow!("import cancelled"));
    }

    if cli.import_content_format != ImportContentFormat::SqlLite {
        return Err(anyhow!(
            "import currently supports import-content-format=sqlite only"
        ));
    }

    let (source_sqlite, _temp_guard) = resolve_import_source(cli, silent)?;
    if !silent {
        println!("[import] using source {}", source_sqlite.display());
    }

    let mut target_cli = cli.clone();
    target_cli.host = cli.import_host.clone();
    target_cli.port = cli.import_port;
    target_cli.db_type = cli.import_db_type;
    target_cli.user = cli.import_user.clone();
    target_cli.password = cli.import_password.clone();
    target_cli.db_name = cli.import_db_name.clone();
    target_cli.export_collation = cli.import_collation.clone();

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

    let mut target_conn = DbConnection::connect(&target_cli)?;

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

        let mut pending_values_sql = Vec::<String>::with_capacity(100);

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
            pending_values_sql.push(format!("({})", values_sql.join(",")));
            if pending_values_sql.len() >= 100 {
                let insert_sql = build_insert_sql(
                    target_cli.db_type,
                    &qt_target,
                    &quoted_cols_target,
                    &pending_values_sql,
                );
                target_conn.execute(&insert_sql)?;
                pending_values_sql.clear();
            }
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

        if !pending_values_sql.is_empty() {
            let insert_sql = build_insert_sql(
                target_cli.db_type,
                &qt_target,
                &quoted_cols_target,
                &pending_values_sql,
            );
            target_conn.execute(&insert_sql)?;
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

fn build_insert_sql(
    db_type: DbType,
    table: &str,
    quoted_cols: &[String],
    values_rows: &[String],
) -> String {
    let col_sql = quoted_cols.join(", ");
    match db_type {
        DbType::Oracle => values_rows
            .iter()
            .map(|values| format!("INSERT INTO {table} ({col_sql}) VALUES {values}"))
            .collect::<Vec<_>>()
            .join("; "),
        _ => format!(
            "INSERT INTO {table} ({col_sql}) VALUES {}",
            values_rows.join(",")
        ),
    }
}
