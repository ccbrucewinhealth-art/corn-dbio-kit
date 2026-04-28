fn execute_export_with_cancel(
    cli: &Cli,
    silent: bool,
    cancel: &AtomicBool,
    progress: Option<&Arc<Mutex<RunProgressSnapshot>>>,
) -> Result<()> {
    let total_start = Instant::now();
    if let Err(e) = fs::create_dir_all(&cli.output_directory) {
        return Err(anyhow!(
            "failed to create output directory {}: {e}",
            cli.output_directory.display()
        ));
    }
    if cli.output_format == OutputFormat::SqlLite {
        let sqlite_path = cli.output_directory.join("export.sqlite");
        if sqlite_path.exists() {
            fs::remove_file(&sqlite_path).with_context(|| {
                format!(
                    "failed to remove existing sqlite export file: {}",
                    sqlite_path.display()
                )
            })?;
        }
    }

    let object_selector = parse_object_selector(&cli.objects);

    let mut conn = match DbConnection::connect(&cli) {
        Ok(v) => v,
        Err(e) => {
            return Err(anyhow!("DB connect failed: {e}"));
        }
    };

    let object_sql = list_objects_sql(cli.db_type);
    let all_objects = match query_first_column_as_vec(&mut conn, object_sql) {
        Ok(v) => v,
        Err(e) => {
            return Err(anyhow!("list objects failed: {e}"));
        }
    };

    let selected_objects = match &object_selector {
        ObjectSelector::Pattern(pattern) => {
            let object_regex = match Regex::new(pattern) {
                Ok(v) => v,
                Err(e) => {
                    return Err(anyhow!("invalid --objects regex: {e}"));
                }
            };
            all_objects
                .into_iter()
                .filter(|t| object_regex.is_match(t))
                .collect::<Vec<_>>()
        }
        ObjectSelector::ListFile(path) => {
            let name_set = match load_object_names_from_list_file(path) {
                Ok(v) => v,
                Err(e) => {
                    return Err(anyhow!("{e}"));
                }
            };
            all_objects
                .into_iter()
                .filter(|o| name_set.contains(o))
                .collect::<Vec<_>>()
        }
        ObjectSelector::SqlFile(_) => Vec::new(),
    };

    if let Some(p) = progress {
        if let Ok(mut s) = p.lock() {
            s.apply_event(RunProgressEvent {
                total_items: Some(selected_objects.len()),
                done_items: Some(0),
                current_item: Some(String::new()),
                current_item_rows_done: Some(0),
                current_item_rows_total: Some(0),
                status: Some("running".to_string()),
            });
        }
    }

    if !silent {
        println!("matched_objects={}", selected_objects.len());
        print_export_header();
    }
    let mut success_tables = 0usize;
    let mut failed_tables = 0usize;
    let mut exported_rows = 0usize;

    if let ObjectSelector::SqlFile(sql_file) = &object_selector {
        let started = Instant::now();
        match export_sql_query_result(&mut conn, &cli, sql_file) {
            Ok(rows) => {
                if !silent {
                    print_export_row(
                        "OBJECT",
                        &format!("@sql:{}", sql_file.display()),
                        None,
                        rows,
                        started.elapsed().as_millis(),
                        "OK",
                    );
                }
                success_tables += 1;
                exported_rows += rows;
            }
            Err(e) => {
                if !silent {
                    print_export_row(
                        "OBJECT",
                        &format!("@sql:{}", sql_file.display()),
                        None,
                        0,
                        started.elapsed().as_millis(),
                        "FAIL",
                    );
                    eprintln!("object=@sql:{} export_failed={e}", sql_file.display());
                }
                failed_tables += 1;
            }
        }
    }

    for (table_idx, table) in selected_objects.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(anyhow!("export cancelled"));
        }
        let estimated_total_rows = if let Ok(v) = query_first_column_as_vec(
            &mut conn,
            &format!("SELECT COUNT(*) FROM {}", quote_ident(cli.db_type, &table)),
        ) {
            v.first().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0)
        } else {
            0
        };
        if let Some(p) = progress {
            if let Ok(mut s) = p.lock() {
                s.apply_event(RunProgressEvent {
                    total_items: None,
                    done_items: Some(table_idx),
                    current_item: Some(table.clone()),
                    current_item_rows_done: Some(0),
                    current_item_rows_total: Some(estimated_total_rows),
                    status: Some("running".to_string()),
                });
            }
        }
        let table_start = Instant::now();
        match export_table(&mut conn, &cli, silent, &table) {
            Ok(rows) => {
                if !silent {
                    print_export_row(
                        "TABLE",
                        &table,
                        None,
                        rows,
                        table_start.elapsed().as_millis(),
                        "OK",
                    );
                }
                success_tables += 1;
                exported_rows += rows;
                if let Some(p) = progress {
                    if let Ok(mut s) = p.lock() {
                        s.apply_event(RunProgressEvent {
                            total_items: None,
                            done_items: Some(table_idx + 1),
                            current_item: Some(table.clone()),
                            current_item_rows_done: Some(rows),
                            current_item_rows_total: Some(estimated_total_rows.max(rows)),
                            status: Some("running".to_string()),
                        });
                    }
                }
            }
            Err(e) => {
                if !silent {
                    print_export_row(
                        "TABLE",
                        &table,
                        None,
                        0,
                        table_start.elapsed().as_millis(),
                        "FAIL",
                    );
                    eprintln!("table={table} export_failed={e}");
                }
                failed_tables += 1;
                if let Some(p) = progress {
                    if let Ok(mut s) = p.lock() {
                        s.apply_event(RunProgressEvent {
                            total_items: None,
                            done_items: Some(table_idx + 1),
                            current_item: Some(table.clone()),
                            current_item_rows_done: Some(0),
                            current_item_rows_total: Some(estimated_total_rows),
                            status: Some("running".to_string()),
                        });
                    }
                }
            }
        }
    }

    if !silent {
        print_export_line();
    }

    if !silent {
        println!("summary success_tables={success_tables} failed_tables={failed_tables} exported_rows={exported_rows} output_dir={}", cli.output_directory.display());
        println!("summary total_elapsed_ms={}", total_start.elapsed().as_millis());
    }

    let mut compress_failed = false;
    if let Some(compress) = cli.compress {
        match compress_output_directory(&cli.output_directory, compress) {
            Ok(archive_path) => {
                if !silent {
                    println!(
                        "compress=ok method={:?} archive={}",
                        compress,
                        archive_path.display()
                    );
                }
                if cli.delete_after_compress {
                    match fs::remove_dir_all(&cli.output_directory) {
                        Ok(_) => {
                            if !silent {
                                println!(
                                    "delete_after_compress=ok removed_dir={}",
                                    cli.output_directory.display()
                                )
                            }
                        }
                        Err(e) => {
                            if !silent {
                                eprintln!(
                                    "delete_after_compress=failed dir={} error={e}",
                                    cli.output_directory.display()
                                );
                            }
                            compress_failed = true;
                        }
                    }
                }
            }
            Err(e) => {
                if !silent {
                    eprintln!("compress=failed error={e}");
                }
                compress_failed = true;
            }
        }
    }

    if failed_tables > 0 || compress_failed {
        if let Some(p) = progress {
            if let Ok(mut s) = p.lock() {
                s.apply_event(RunProgressEvent {
                    status: Some("failed".to_string()),
                    ..RunProgressEvent::default()
                });
            }
        }
        return Err(anyhow!(
            "export finished with failure: failed_tables={failed_tables}, compress_failed={compress_failed}"
        ));
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

fn execute_export(cli: &Cli, silent: bool) -> Result<()> {
    let cancel = AtomicBool::new(false);
    execute_export_with_cancel(cli, silent, &cancel, None)
}
