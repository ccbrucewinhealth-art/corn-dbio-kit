fn export_single_page(
    conn: &mut DbConnection,
    cli: &Cli,
    silent: bool,
    table: &str,
    page_index: usize,
    offset: usize,
    limit: usize,
    output_directory: &Path,
) -> Result<usize> {
    let page_start = Instant::now();
    let quoted_table = quote_ident(cli.db_type, table);
    let sqlite_table = quote_ident(DbType::SqlLite, table);
    let auto_inc_pk_set = if cli.hide_primary_key {
        auto_increment_pk_column_set(conn, cli.db_type, table)?
    } else {
        HashSet::new()
    };
    let mut sql = format!("SELECT * FROM {quoted_table}");
    if let Some(condition) = &cli.condition {
        let trimmed = condition.trim();
        if !trimmed.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(trimmed);
        }
    }
    sql.push_str(&order_by_clause(cli.db_type, cli.order_by.as_deref()));
    sql.push_str(&pagination_clause(cli.db_type, offset, limit));

    let ext = output_file_extension(cli.output_format);
    let file_name = format!("{}_{}.{}", table, format!("{:06}", page_index), ext);
    let file_path = output_directory.join(file_name);
    let sqlite_file_path = output_directory.join("export.sqlite");
    let mut file: Option<File> = None;
    let mut sqlite_conn: Option<SqliteConnection> = None;
    let mut sqlite_insert_sql = String::new();

    let mut written_rows = 0usize;
    let result = conn.query(&sql)?;
    if !result.columns.is_empty() {
        let col_count = result.columns.len();
        if col_count == 0 {
            if !silent {
                print_export_row(
                    "PAGE",
                    table,
                    Some(page_index),
                    0,
                    page_start.elapsed().as_millis(),
                    "OK",
                );
            }
            return Ok(0);
        }

        let column_names = result.columns;
        let quoted_columns = column_names
            .iter()
            .map(|col_name| quote_ident(cli.db_type, col_name))
            .collect::<Vec<_>>();
        let insert_prefix = format!(
            "INSERT INTO {quoted_table} ({})\nVALUES\n    ",
            quoted_columns.join(", ")
        );

        if cli.output_format == OutputFormat::SqlLite {
            let sqlite = SqliteConnection::open(&sqlite_file_path).with_context(|| {
                format!("failed to open sqlite export db: {}", sqlite_file_path.display())
            })?;
            sqlite.execute_batch("BEGIN IMMEDIATE TRANSACTION;")?;
            let sqlite_cols = column_names
                .iter()
                .map(|c| format!("{} TEXT", quote_ident(DbType::SqlLite, c)))
                .collect::<Vec<_>>()
                .join(", ");
            let create_sql = format!("CREATE TABLE IF NOT EXISTS {sqlite_table} ({sqlite_cols});");
            sqlite.execute_batch(&create_sql)?;
            let sqlite_quoted_columns = column_names
                .iter()
                .map(|c| quote_ident(DbType::SqlLite, c))
                .collect::<Vec<_>>()
                .join(", ");
            let placeholders = std::iter::repeat_n("?", column_names.len())
                .collect::<Vec<_>>()
                .join(", ");
            sqlite_insert_sql = format!(
                "INSERT INTO {sqlite_table} ({sqlite_quoted_columns}) VALUES ({placeholders})"
            );
            sqlite_conn = Some(sqlite);
        }

        let mut pending_rows = Vec::<String>::with_capacity(100);
        let mut json_first_row = true;

        for row in result.rows {
                let mut values_raw = Vec::<Option<String>>::new();
                for col_index in 0..col_count {
                    let col_name_lc = column_names
                        .get(col_index)
                        .map(|s| s.to_ascii_lowercase())
                        .unwrap_or_default();
                    if cli.hide_primary_key && auto_inc_pk_set.contains(&col_name_lc) {
                        values_raw.push(None);
                        continue;
                    }

                    let value = to_insert_value(row.get(col_index).and_then(|v| v.as_deref()));
                    if value == "NULL" {
                        values_raw.push(None);
                    } else {
                        values_raw.push(Some(value.trim_matches('"').trim_matches('\'').to_string()));
                    }
                }

                match cli.output_format {
                    OutputFormat::Sql => {
                        let sql_values = values_raw
                            .iter()
                            .map(|v| match v {
                                Some(s) => format!("'{}'", escape_sql_value(s)),
                                None => "NULL".to_string(),
                            })
                            .collect::<Vec<_>>();

                        pending_rows.push(format!("({})", sql_values.join(", ")));
                        if pending_rows.len() >= 100 {
                            if file.is_none() {
                                file = Some(File::create(&file_path).with_context(|| {
                                    format!("failed to create output file: {}", file_path.display())
                                })?);
                            }
                            let line = format!("{}{};\n", insert_prefix, pending_rows.join(",\n    "));
                            file
                                .as_mut()
                                .expect("file must be initialized")
                                .write_all(line.as_bytes())?;
                            pending_rows.clear();
                        }
                    }
                    OutputFormat::Csv => {
                        if file.is_none() {
                            let mut f = File::create(&file_path).with_context(|| {
                                format!("failed to create output file: {}", file_path.display())
                            })?;
                            let header = column_names
                                .iter()
                                .map(|c| escape_csv_value(c))
                                .collect::<Vec<_>>()
                                .join(",");
                            f.write_all(format!("{}\n", header).as_bytes())?;
                            file = Some(f);
                        }
                        let line = values_raw
                            .iter()
                            .map(|v| match v {
                                Some(s) => escape_csv_value(s),
                                None => "".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        file.as_mut()
                            .expect("file must be initialized")
                            .write_all(format!("{}\n", line).as_bytes())?;
                    }
                    OutputFormat::Excel => {
                        if file.is_none() {
                            let mut f = File::create(&file_path).with_context(|| {
                                format!("failed to create output file: {}", file_path.display())
                            })?;
                            let header = column_names
                                .iter()
                                .map(|c| escape_tsv_value(c))
                                .collect::<Vec<_>>()
                                .join("\t");
                            f.write_all(format!("{}\n", header).as_bytes())?;
                            file = Some(f);
                        }
                        let line = values_raw
                            .iter()
                            .map(|v| match v {
                                Some(s) => escape_tsv_value(s),
                                None => "".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join("\t");
                        file.as_mut()
                            .expect("file must be initialized")
                            .write_all(format!("{}\n", line).as_bytes())?;
                    }
                    OutputFormat::Json => {
                        if file.is_none() {
                            let mut f = File::create(&file_path).with_context(|| {
                                format!("failed to create output file: {}", file_path.display())
                            })?;
                            f.write_all(b"[\n")?;
                            file = Some(f);
                        }

                        if !json_first_row {
                            file.as_mut()
                                .expect("file must be initialized")
                                .write_all(b",\n")?;
                        }
                        json_first_row = false;

                        let fields = column_names
                            .iter()
                            .enumerate()
                            .map(|(idx, col)| match values_raw.get(idx) {
                                Some(Some(v)) => {
                                    format!("\"{}\":\"{}\"", escape_json_value(col), escape_json_value(v))
                                }
                                _ => format!("\"{}\":null", escape_json_value(col)),
                            })
                            .collect::<Vec<_>>()
                            .join(",");
                        file.as_mut()
                            .expect("file must be initialized")
                            .write_all(format!("  {{{}}}", fields).as_bytes())?;
                    }
                    OutputFormat::Xml => {
                        if file.is_none() {
                            let mut f = File::create(&file_path).with_context(|| {
                                format!("failed to create output file: {}", file_path.display())
                            })?;
                            f.write_all(b"<rows>\n")?;
                            file = Some(f);
                        }
                        file.as_mut()
                            .expect("file must be initialized")
                            .write_all(b"  <row>\n")?;
                        for (idx, col) in column_names.iter().enumerate() {
                            let val = values_raw.get(idx).cloned().flatten();
                            let v = val
                                .as_deref()
                                .map(escape_xml_value)
                                .unwrap_or_else(String::new);
                            let line = format!(
                                "    <c name=\"{}\">{}</c>\n",
                                escape_xml_value(col),
                                v
                            );
                            file.as_mut()
                                .expect("file must be initialized")
                                .write_all(line.as_bytes())?;
                        }
                        file.as_mut()
                            .expect("file must be initialized")
                            .write_all(b"  </row>\n")?;
                    }
                    OutputFormat::Md => {
                        if file.is_none() {
                            let mut f = File::create(&file_path).with_context(|| {
                                format!("failed to create output file: {}", file_path.display())
                            })?;
                            let mut header_cols = vec!["no".to_string(), "表格名稱".to_string()];
                            header_cols.extend(column_names.iter().cloned());
                            let header = format!(
                                "| {} |\n",
                                header_cols
                                    .iter()
                                    .map(|c| escape_md_value(c))
                                    .collect::<Vec<_>>()
                                    .join(" | ")
                            );
                            let mut sep_cols = vec!["---:".to_string(), "---".to_string()];
                            sep_cols.extend(std::iter::repeat_n("---".to_string(), column_names.len()));
                            let sep = format!("| {} |\n", sep_cols.join(" | "));
                            f.write_all(header.as_bytes())?;
                            f.write_all(sep.as_bytes())?;
                            file = Some(f);
                        }
                        let mut row_cells = vec![
                            (written_rows + 1).to_string(),
                            escape_md_value(table),
                        ];
                        row_cells.extend(
                            column_names
                            .iter()
                            .enumerate()
                            .map(|(idx, col)| {
                                let val = values_raw
                                    .get(idx)
                                    .and_then(|v| v.as_deref())
                                    .unwrap_or("");
                                let _ = col;
                                escape_md_value(val)
                            })
                            .collect::<Vec<_>>(),
                        );
                        let line = format!("| {} |\n", row_cells.join(" | "));
                        file.as_mut()
                            .expect("file must be initialized")
                            .write_all(line.as_bytes())?;
                    }
                    OutputFormat::SqlLite => {
                        let vals = values_raw
                            .iter()
                            .map(|v| match v {
                                Some(s) => rusqlite::types::Value::Text(s.clone()),
                                None => rusqlite::types::Value::Null,
                            })
                            .collect::<Vec<_>>();
                        sqlite_conn
                            .as_ref()
                            .expect("sqlite connection must be initialized")
                            .execute(&sqlite_insert_sql, params_from_iter(vals.iter()))?;
                    }
                }
                written_rows += 1;
        }

        match cli.output_format {
            OutputFormat::Sql => {
                if !pending_rows.is_empty() {
                    if file.is_none() {
                        file = Some(File::create(&file_path).with_context(|| {
                            format!("failed to create output file: {}", file_path.display())
                        })?);
                    }
                    let line = format!("{}{};\n", insert_prefix, pending_rows.join(",\n    "));
                    file
                        .as_mut()
                        .expect("file must be initialized")
                        .write_all(line.as_bytes())?;
                }
            }
            OutputFormat::Json => {
                if let Some(f) = file.as_mut() {
                    f.write_all(b"\n]\n")?;
                }
            }
            OutputFormat::Xml => {
                if let Some(f) = file.as_mut() {
                    f.write_all(b"</rows>\n")?;
                }
            }
            OutputFormat::Csv | OutputFormat::Excel | OutputFormat::Md => {}
            OutputFormat::SqlLite => {
                if let Some(sqlite) = sqlite_conn.as_ref() {
                    sqlite.execute_batch("COMMIT;")?;
                }
            }
        }
    }

    if !silent {
        print_export_row(
            "PAGE",
            table,
            Some(page_index),
            written_rows,
            page_start.elapsed().as_millis(),
            "OK",
        );
    }

    Ok(written_rows)
}

fn export_table(conn: &mut DbConnection, cli: &Cli, silent: bool, table: &str) -> Result<usize> {
    let mut total_rows = 0usize;
    let mut page_index = cli.page_no.unwrap_or(1);

    loop {
        let offset = (page_index - 1) * cli.page_size;
        let remaining = cli
            .recsize
            .map(|max| max.saturating_sub(total_rows))
            .unwrap_or(cli.page_size);
        if remaining == 0 {
            break;
        }
        let limit = cli.page_size.min(remaining);

        let rows = export_single_page(
            conn,
            cli,
            silent,
            table,
            page_index,
            offset,
            limit,
            &cli.output_directory,
        )?;

        if rows == 0 {
            break;
        }

        total_rows += rows;
        if cli.page_no.is_some() {
            break;
        }
        if rows < limit {
            break;
        }
        page_index += 1;
    }

    Ok(total_rows)
}
