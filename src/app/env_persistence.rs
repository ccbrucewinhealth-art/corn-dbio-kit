fn env_file_path() -> PathBuf {
    let parent_env = Path::new("..").join(".env");
    if parent_env.exists() {
        parent_env
    } else {
        PathBuf::from(".env")
    }
}

fn try_load_env() -> Result<PathBuf> {
    let env_path = env_file_path();
    if !env_path.exists() {
        File::create(&env_path)
            .with_context(|| format!("failed to create env file: {}", env_path.display()))?;
    }
    let _ = from_filename(&env_path);
    Ok(env_path)
}

fn mode_to_str(v: RunMode) -> &'static str {
    match v {
        RunMode::Cli => "cli",
        RunMode::Tui => "tui",
    }
}

fn flow_to_str(v: FlowMode) -> &'static str {
    match v {
        FlowMode::Export => "export",
        FlowMode::Import => "import",
    }
}

fn lang_to_str(v: Lang) -> &'static str {
    match v {
        Lang::Tw => "tw",
        Lang::En => "en",
        Lang::Ja => "ja",
    }
}

fn db_type_to_str(v: DbType) -> &'static str {
    match v {
        DbType::Mssql => "mssql",
        DbType::Oracle => "oracle",
        DbType::Mysql => "mysql",
        DbType::Postgresql => "postgresql",
        DbType::SqlLite => "sql-lite",
    }
}

fn output_format_to_str(v: OutputFormat) -> &'static str {
    match v {
        OutputFormat::Sql => "sql",
        OutputFormat::SqlLite => "sql-lite",
        OutputFormat::Csv => "csv",
        OutputFormat::Excel => "excel",
        OutputFormat::Json => "json",
        OutputFormat::Xml => "xml",
        OutputFormat::Md => "md",
    }
}

fn compress_to_str(v: CompressType) -> &'static str {
    match v {
        CompressType::Zip => "zip",
        CompressType::Bzip => "bzip",
        CompressType::Tar => "tar",
    }
}

fn env_quote(v: &str) -> String {
    format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""))
}

fn persist_env_from_cli(env_path: &Path, cli: &Cli) -> Result<()> {
    let managed = vec![
        ("DB2SQL_FLOW".to_string(), Some(flow_to_str(cli.flow).to_string())),
        ("DB2SQL_MODE".to_string(), Some(mode_to_str(cli.mode).to_string())),
        ("DB2SQL_LANG".to_string(), Some(lang_to_str(cli.lang).to_string())),
        (
            "DB2SQL_EXPORT_DB_TYPE".to_string(),
            Some(db_type_to_str(cli.db_type).to_string()),
        ),
        ("DB2SQL_EXPORT_HOST".to_string(), Some(cli.host.clone())),
        ("DB2SQL_EXPORT_PORT".to_string(), Some(cli.port.to_string())),
        ("DB2SQL_EXPORT_USER".to_string(), Some(cli.user.clone())),
        ("DB2SQL_EXPORT_PASSWORD".to_string(), Some(cli.password.clone())),
        ("DB2SQL_EXPORT_DB_NAME".to_string(), Some(cli.db_name.clone())),
        ("DB2SQL_EXPORT_OBJECTS".to_string(), Some(cli.objects.clone())),
        ("DB2SQL_EXPORT_CONDITION".to_string(), cli.condition.clone()),
        ("DB2SQL_EXPORT_ORDER_BY".to_string(), cli.order_by.clone()),
        (
            "DB2SQL_EXPORT_HIDE_PRIMARY_KEY".to_string(),
            Some(cli.hide_primary_key.to_string()),
        ),
        (
            "DB2SQL_EXPORT_OUTPUT_FORMAT".to_string(),
            Some(output_format_to_str(cli.output_format).to_string()),
        ),
        (
            "DB2SQL_EXPORT_COMPRESS".to_string(),
            cli.compress.map(|v| compress_to_str(v).to_string()),
        ),
        (
            "DB2SQL_EXPORT_DELETE_AFTER_COMPRESS".to_string(),
            Some(cli.delete_after_compress.to_string()),
        ),
        (
            "DB2SQL_EXPORT_OUTPUT_DIRECTORY".to_string(),
            Some(cli.output_directory.display().to_string()),
        ),
        (
            "DB2SQL_EXPORT_RECSIZE".to_string(),
            cli.recsize.map(|v| v.to_string()),
        ),
        (
            "DB2SQL_EXPORT_PAGE_SIZE".to_string(),
            Some(cli.page_size.to_string()),
        ),
        (
            "DB2SQL_EXPORT_PAGE_NO".to_string(),
            cli.page_no.map(|v| v.to_string()),
        ),
        ("DB2SQL_IMPORT_HOST".to_string(), Some(cli.import_host.clone())),
        (
            "DB2SQL_IMPORT_PORT".to_string(),
            Some(cli.import_port.to_string()),
        ),
        (
            "DB2SQL_IMPORT_DB_TYPE".to_string(),
            Some(db_type_to_str(cli.import_db_type).to_string()),
        ),
        ("DB2SQL_IMPORT_USER".to_string(), Some(cli.import_user.clone())),
        (
            "DB2SQL_IMPORT_PASSWORD".to_string(),
            Some(cli.import_password.clone()),
        ),
        (
            "DB2SQL_IMPORT_DB_NAME".to_string(),
            Some(cli.import_db_name.clone()),
        ),
    ];

    let existing = fs::read_to_string(env_path).unwrap_or_default();
    let mut seen = HashSet::<String>::new();
    let mut out_lines = Vec::<String>::new();

    for line in existing.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || !trimmed.contains('=') {
            out_lines.push(line.to_string());
            continue;
        }
        let key = trimmed
            .splitn(2, '=')
            .next()
            .map(|v| v.trim())
            .unwrap_or_default()
            .to_string();

        if let Some((_, val)) = managed.iter().find(|(k, _)| *k == key) {
            seen.insert(key.clone());
            if let Some(v) = val {
                out_lines.push(format!("{key}={}", env_quote(v)));
            }
        } else {
            out_lines.push(line.to_string());
        }
    }

    for (k, v) in managed {
        if seen.contains(&k) {
            continue;
        }
        if let Some(val) = v {
            out_lines.push(format!("{k}={}", env_quote(&val)));
        }
    }

    out_lines.push(String::new());
    fs::write(env_path, out_lines.join("\n"))
        .with_context(|| format!("failed to write env file: {}", env_path.display()))?;
    Ok(())
}
