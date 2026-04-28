fn export_sql_query_result(
    conn: &mut DbConnection,
    cli: &Cli,
    sql_file: &Path,
) -> Result<usize> {
    let sql_content = fs::read_to_string(sql_file)
        .with_context(|| format!("failed to read @sql file: {}", sql_file.display()))?;
    let object_name = sql_file
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "sql_result".to_string());

    let ext = output_file_extension(cli.output_format);
    let file_path = cli
        .output_directory
        .join(format!("{}_{}.{}", object_name, format!("{:06}", 1), ext));
    let sqlite_file_path = cli.output_directory.join("export.sqlite");
    let mut file: Option<File> = None;
    let mut sqlite_conn: Option<SqliteConnection> = None;
    let mut sqlite_insert_sql = String::new();
    let mut written_rows = 0usize;

    let result = conn.query(&sql_content)?;
    let col_count = result.columns.len();
    if col_count == 0 {
        return Ok(0);
    }
    let column_names = result.columns;

        if cli.output_format == OutputFormat::SqlLite {
            let sqlite = SqliteConnection::open(&sqlite_file_path).with_context(|| {
                format!("failed to open sqlite export db: {}", sqlite_file_path.display())
            })?;
            sqlite.execute_batch("BEGIN IMMEDIATE TRANSACTION;")?;
            let sqlite_table = quote_ident(DbType::SqlLite, &object_name);
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

        let mut json_first_row = true;

        for row in result.rows {
                if file.is_none() {
                    file = Some(File::create(&file_path).with_context(|| {
                        format!("failed to create output file: {}", file_path.display())
                    })?);
                }

                let mut values_raw = Vec::<Option<String>>::new();
                for col_index in 0..col_count {
                    values_raw.push(row.get(col_index).cloned().flatten());
                }

                match cli.output_format {
                    OutputFormat::Csv => {
                        if written_rows == 0 {
                            let header = column_names
                                .iter()
                                .map(|c| escape_csv_value(c))
                                .collect::<Vec<_>>()
                                .join(",");
                            file.as_mut().unwrap().write_all(format!("{}\n", header).as_bytes())?;
                        }
                        let line = values_raw
                            .iter()
                            .map(|v| v.as_deref().map(escape_csv_value).unwrap_or_default())
                            .collect::<Vec<_>>()
                            .join(",");
                        file.as_mut().unwrap().write_all(format!("{}\n", line).as_bytes())?;
                    }
                    OutputFormat::Excel => {
                        if written_rows == 0 {
                            let header = column_names
                                .iter()
                                .map(|c| escape_tsv_value(c))
                                .collect::<Vec<_>>()
                                .join("\t");
                            file.as_mut().unwrap().write_all(format!("{}\n", header).as_bytes())?;
                        }
                        let line = values_raw
                            .iter()
                            .map(|v| v.as_deref().map(escape_tsv_value).unwrap_or_default())
                            .collect::<Vec<_>>()
                            .join("\t");
                        file.as_mut().unwrap().write_all(format!("{}\n", line).as_bytes())?;
                    }
                    OutputFormat::Json => {
                        if written_rows == 0 {
                            file.as_mut().unwrap().write_all(b"[\n")?;
                        }
                        if !json_first_row {
                            file.as_mut().unwrap().write_all(b",\n")?;
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
                            .unwrap()
                            .write_all(format!("  {{{}}}", fields).as_bytes())?;
                    }
                    OutputFormat::Xml => {
                        if written_rows == 0 {
                            file.as_mut().unwrap().write_all(b"<rows>\n")?;
                        }
                        file.as_mut().unwrap().write_all(b"  <row>\n")?;
                        for (idx, col) in column_names.iter().enumerate() {
                            let v = values_raw
                                .get(idx)
                                .and_then(|v| v.as_deref())
                                .map(escape_xml_value)
                                .unwrap_or_default();
                            file.as_mut().unwrap().write_all(
                                format!("    <c name=\"{}\">{}</c>\n", escape_xml_value(col), v)
                                    .as_bytes(),
                            )?;
                        }
                        file.as_mut().unwrap().write_all(b"  </row>\n")?;
                    }
                    OutputFormat::Md => {
                        if written_rows == 0 {
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
                            file.as_mut().unwrap().write_all(header.as_bytes())?;
                            file.as_mut().unwrap().write_all(sep.as_bytes())?;
                        }
                        let mut row_cells = vec![
                            (written_rows + 1).to_string(),
                            escape_md_value(&object_name),
                        ];
                        row_cells.extend(
                            column_names
                            .iter()
                            .enumerate()
                            .map(|(idx, c)| {
                                let v = values_raw
                                    .get(idx)
                                    .and_then(|x| x.as_deref())
                                    .unwrap_or("");
                                let _ = c;
                                escape_md_value(v)
                            })
                            .collect::<Vec<_>>(),
                        );
                        let line = format!("| {} |\n", row_cells.join(" | "));
                        file.as_mut().unwrap().write_all(line.as_bytes())?;
                    }
                    OutputFormat::Sql => {
                        // @sql 匯入查詢結果無法可靠逆推 insert 目標，退化為 CSV 樣式
                        if written_rows == 0 {
                            let header = column_names.join(",");
                            file.as_mut().unwrap().write_all(format!("{}\n", header).as_bytes())?;
                        }
                        let line = values_raw
                            .iter()
                            .map(|v| v.clone().unwrap_or_default())
                            .collect::<Vec<_>>()
                            .join(",");
                        file.as_mut().unwrap().write_all(format!("{}\n", line).as_bytes())?;
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
            OutputFormat::SqlLite => {
                if let Some(sqlite) = sqlite_conn.as_ref() {
                    sqlite.execute_batch("COMMIT;")?;
                }
            }
            _ => {}
        }

    Ok(written_rows)
}

fn escape_csv_value(raw: &str) -> String {
    if raw.contains(',') || raw.contains('"') || raw.contains('\n') || raw.contains('\r') {
        format!("\"{}\"", raw.replace('"', "\"\""))
    } else {
        raw.to_string()
    }
}

fn escape_json_value(raw: &str) -> String {
    raw.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn escape_xml_value(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_tsv_value(raw: &str) -> String {
    raw.replace('\t', " ").replace('\n', " ").replace('\r', " ")
}

fn archive_extension(compress: CompressType) -> &'static str {
    match compress {
        CompressType::Zip => "zip",
        CompressType::Bzip => "bz2",
        CompressType::Tar => "tar",
    }
}

fn current_timestamp_yyyymmddhhmmss() -> Result<String> {
    let output = Command::new("date").arg("+%Y%m%d%H%M%S").output()?;
    if !output.status.success() {
        return Err(anyhow!("failed to get current time by `date` command"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn compress_output_directory(output_directory: &Path, compress: CompressType) -> Result<PathBuf> {
    let timestamp = current_timestamp_yyyymmddhhmmss()?;
    let ext = archive_extension(compress);
    let archive_name = format!("wh-export-{timestamp}.{ext}");

    let parent_dir = output_directory
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let output_dir_name = output_directory
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .ok_or_else(|| anyhow!("invalid output directory path: {}", output_directory.display()))?;
    let archive_path = parent_dir.join(&archive_name);

    let status = match compress {
        CompressType::Zip => Command::new("zip")
            .arg("-r")
            .arg("-q")
            .arg(&archive_name)
            .arg(&output_dir_name)
            .current_dir(&parent_dir)
            .status()?,
        CompressType::Bzip => Command::new("tar")
            .arg("-cjf")
            .arg(&archive_name)
            .arg(&output_dir_name)
            .current_dir(&parent_dir)
            .status()?,
        CompressType::Tar => Command::new("tar")
            .arg("-cf")
            .arg(&archive_name)
            .arg(&output_dir_name)
            .current_dir(&parent_dir)
            .status()?,
    };

    if !status.success() {
        return Err(anyhow!(
            "compress command failed, method={:?}, output_dir={}",
            compress,
            output_directory.display()
        ));
    }

    Ok(archive_path)
}
