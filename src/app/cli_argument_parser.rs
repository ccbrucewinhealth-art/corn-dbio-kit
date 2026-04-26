fn parse_db_type(raw: &str) -> Result<DbType> {
    match raw.to_ascii_lowercase().as_str() {
        "mssql" => Ok(DbType::Mssql),
        "oracle" => Ok(DbType::Oracle),
        "mysql" => Ok(DbType::Mysql),
        "postgresql" | "postgres" => Ok(DbType::Postgresql),
        "sql-lite" | "sqlite" => Ok(DbType::SqlLite),
        _ => Err(anyhow!(
            "unsupported db-type: {raw} (allowed: mssql, oracle, mysql, postgresql, sql-lite)"
        )),
    }
}

fn parse_output_format(raw: &str) -> Result<OutputFormat> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "sql" => Ok(OutputFormat::Sql),
        "sql-lite" | "sqlite" => Ok(OutputFormat::SqlLite),
        "csv" => Ok(OutputFormat::Csv),
        "excel" => Ok(OutputFormat::Excel),
        "json" => Ok(OutputFormat::Json),
        "xml" => Ok(OutputFormat::Xml),
        "md" => Ok(OutputFormat::Md),
        _ => Err(anyhow!(
            "unsupported output-format: {raw} (allowed: sql, sql-lite, csv, excel, json, xml, md)"
        )),
    }
}

fn parse_compress_type(raw: &str) -> Result<CompressType> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "zip" => Ok(CompressType::Zip),
        "bzip" => Ok(CompressType::Bzip),
        "tar" => Ok(CompressType::Tar),
        _ => Err(anyhow!(
            "unsupported compress value: {raw} (allowed: zip, bzip, tar)"
        )),
    }
}

fn parse_run_mode(raw: &str) -> Result<RunMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "cli" => Ok(RunMode::Cli),
        "tui" => Ok(RunMode::Tui),
        _ => Err(anyhow!("unsupported mode: {raw} (allowed: cli, tui)")),
    }
}

fn parse_flow_mode(raw: &str) -> Result<FlowMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "export" => Ok(FlowMode::Export),
        "import" => Ok(FlowMode::Import),
        _ => Err(anyhow!("unsupported flow: {raw} (allowed: export, import)")),
    }
}

fn parse_lang(raw: &str) -> Result<Lang> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "tw" => Ok(Lang::Tw),
        "en" => Ok(Lang::En),
        "ja" => Ok(Lang::Ja),
        _ => Err(anyhow!("unsupported lang: {raw} (allowed: tw, en, ja)")),
    }
}

fn default_tui_cli() -> Cli {
    Cli {
        flow: FlowMode::Export,
        mode: RunMode::Tui,
        lang: Lang::Tw,
        db_type: DbType::Mssql,
        host: String::new(),
        port: 1433,
        user: String::new(),
        password: String::new(),
        db_name: String::new(),
        objects: ".*".to_string(),
        condition: None,
        order_by: None,
        hide_primary_key: false,
        output_format: OutputFormat::Sql,
        compress: None,
        delete_after_compress: false,
        output_directory: PathBuf::from("./export"),
        recsize: None,
        page_size: 10_000,
        page_no: None,
        import_host: String::new(),
        import_port: 1433,
        import_db_type: DbType::Mssql,
        import_user: String::new(),
        import_password: String::new(),
        import_db_name: String::new(),
    }
}

fn env_non_empty(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn apply_env_to_cli(cli: &mut Cli) -> Result<()> {
    if let Some(v) = env_non_empty("DB2SQL_FLOW") {
        cli.flow = parse_flow_mode(&v)?;
    }
    if let Some(v) = env_non_empty("DB2SQL_MODE") {
        cli.mode = parse_run_mode(&v)?;
    }
    if let Some(v) = env_non_empty("DB2SQL_LANG") {
        cli.lang = parse_lang(&v)?;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_DB_TYPE").or_else(|| env_non_empty("DB2SQL_DB_TYPE")) {
        cli.db_type = parse_db_type(&v)?;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_HOST").or_else(|| env_non_empty("DB2SQL_HOST")) {
        cli.host = v;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_PORT").or_else(|| env_non_empty("DB2SQL_PORT")) {
        let p = v
            .parse::<u16>()
            .with_context(|| format!("invalid DB2SQL_EXPORT_PORT value: {v}"))?;
        if p == 0 {
            return Err(anyhow!("invalid DB2SQL_EXPORT_PORT value: 0"));
        }
        cli.port = p;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_USER").or_else(|| env_non_empty("DB2SQL_USER")) {
        cli.user = v;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_PASSWORD").or_else(|| env_non_empty("DB2SQL_PASSWORD")) {
        cli.password = v;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_DB_NAME").or_else(|| env_non_empty("DB2SQL_DB_NAME")) {
        cli.db_name = v;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_OBJECTS").or_else(|| env_non_empty("DB2SQL_OBJECTS")) {
        cli.objects = v;
    }
    cli.condition = env_non_empty("DB2SQL_EXPORT_CONDITION").or_else(|| env_non_empty("DB2SQL_CONDITION"));
    cli.order_by = env_non_empty("DB2SQL_EXPORT_ORDER_BY").or_else(|| env_non_empty("DB2SQL_ORDER_BY"));
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_HIDE_PRIMARY_KEY").or_else(|| env_non_empty("DB2SQL_HIDE_PRIMARY_KEY")) {
        cli.hide_primary_key = parse_bool_arg("DB2SQL_EXPORT_HIDE_PRIMARY_KEY", &v)?;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_OUTPUT_FORMAT").or_else(|| env_non_empty("DB2SQL_OUTPUT_FORMAT")) {
        cli.output_format = parse_output_format(&v)?;
    }
    cli.compress = match env_non_empty("DB2SQL_EXPORT_COMPRESS").or_else(|| env_non_empty("DB2SQL_COMPRESS")) {
        Some(v) => Some(parse_compress_type(&v)?),
        None => None,
    };
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_DELETE_AFTER_COMPRESS").or_else(|| env_non_empty("DB2SQL_DELETE_AFTER_COMPRESS")) {
        cli.delete_after_compress = parse_bool_arg("DB2SQL_EXPORT_DELETE_AFTER_COMPRESS", &v)?;
    }
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_OUTPUT_DIRECTORY").or_else(|| env_non_empty("DB2SQL_OUTPUT_DIRECTORY")) {
        cli.output_directory = PathBuf::from(v);
    }
    cli.recsize = match env_non_empty("DB2SQL_EXPORT_RECSIZE").or_else(|| env_non_empty("DB2SQL_RECSIZE")) {
        Some(v) => Some(parse_positive_usize("DB2SQL_EXPORT_RECSIZE", &v)?),
        None => None,
    };
    if let Some(v) = env_non_empty("DB2SQL_EXPORT_PAGE_SIZE").or_else(|| env_non_empty("DB2SQL_PAGE_SIZE")) {
        cli.page_size = parse_positive_usize("DB2SQL_EXPORT_PAGE_SIZE", &v)?;
    }
    cli.page_no = match env_non_empty("DB2SQL_EXPORT_PAGE_NO").or_else(|| env_non_empty("DB2SQL_PAGE_NO")) {
        Some(v) => Some(parse_positive_usize("DB2SQL_EXPORT_PAGE_NO", &v)?),
        None => None,
    };
    if let Some(v) = env_non_empty("DB2SQL_IMPORT_HOST") {
        cli.import_host = v;
    }
    if let Some(v) = env_non_empty("DB2SQL_IMPORT_PORT") {
        let p = v
            .parse::<u16>()
            .with_context(|| format!("invalid DB2SQL_IMPORT_PORT value: {v}"))?;
        if p == 0 {
            return Err(anyhow!("invalid DB2SQL_IMPORT_PORT value: 0"));
        }
        cli.import_port = p;
    }
    if let Some(v) = env_non_empty("DB2SQL_IMPORT_DB_TYPE") {
        cli.import_db_type = parse_db_type(&v)?;
    }
    if let Some(v) = env_non_empty("DB2SQL_IMPORT_USER") {
        cli.import_user = v;
    }
    if let Some(v) = env_non_empty("DB2SQL_IMPORT_PASSWORD") {
        cli.import_password = v;
    }
    if let Some(v) = env_non_empty("DB2SQL_IMPORT_DB_NAME") {
        cli.import_db_name = v;
    }
    Ok(())
}

fn parse_positive_usize(name: &str, value: &str) -> Result<usize> {
    let v = value
        .parse::<usize>()
        .with_context(|| format!("invalid {name} value: {value}"))?;
    if v == 0 {
        return Err(anyhow!("invalid {name} value: 0"));
    }
    Ok(v)
}

fn parse_bool_arg(name: &str, value: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(anyhow!("invalid {name} value: {value} (allowed: true|false)")),
    }
}

fn print_help(program: &str) {
    println!("corn-dbio-kit - export table data to SQL INSERT scripts");
    println!();
    println!("Usage:");
    println!(
        "  {program} --db-type <mssql|oracle|mysql|postgresql|sql-lite> --host <host> --port <port> --user <user> --password <password> --db-name <db-name> [--objects <regex|@sql:file|@list:file>] [--condition <where-condition>] [--output-directory <dir>] [--recsize <n>] [--page-size <n>] [--page-no <n>]"
    );
    println!();
    println!("Arguments:");
    println!("  --flow              流程：export|import，預設 export");
    println!("  --export-db-type    匯出資料庫類型（必填，含 sql-lite）");
    println!("  --mode              執行模式：cli|tui，預設 tui");
    println!("  --lang              語系：tw|en|ja，預設 tw（tw:繁中, en:美式英文, ja:日文）");
    println!("  --export-host       匯出 DB host（必填）");
    println!("  --export-port       匯出 DB port（必填）");
    println!("  --export-user       匯出 DB 帳號（必填）");
    println!("  --export-password   匯出 DB 密碼（必填）");
    println!("  --export-db-name    匯出 DB 名稱（必填；export-db-type=sql-lite 時為 sqlite 檔名）");
    println!("  --export-objects    選擇物件(table/view)；可為 regex，預設 .* ");
    println!("                      @sql:<filename> 讀取 SQL 檔內容執行後匯出");
    println!("                      @list:<filename> 讀取 table/view 名稱清單");
    println!("  --export-condition  WHERE 條件，不需寫 WHERE 關鍵字");
    println!("  --export-order-by   排序條件，例如 \"SnId desc\" 或 \"SnId\"");
    println!("  --export-hide-primary-key  true|false；true 時自增主鍵欄位輸出為 NULL");
    println!("  --export-output-format     輸出格式：sql|sql-lite|csv|excel|json|xml|md，預設 sql");
    println!("  --export-compress          壓縮格式：zip|bzip|tar");
    println!("  --export-delete-after-compress true|false；壓縮成功後是否刪除輸出目錄");
    println!("  --export-output-directory  輸出目錄，預設 ./export");
    println!("  --export-recsize           每個 table 最多輸出筆數");
    println!("  --export-page-size         每頁筆數，預設 10000");
    println!("  --export-page-no           指定輸出第幾頁；未指定則輸出全部頁");
    println!("  --import-host       匯入目標 host（flow=import 時使用）");
    println!("  --import-port       匯入目標 port（flow=import 時使用）");
    println!("  --import-db-type    匯入目標資料庫類型：mssql|oracle|mysql|postgresql|sql-lite");
    println!("  --import-user       匯入目標使用者（flow=import 時使用）");
    println!("  --import-password   匯入目標密碼（flow=import 時使用）");
    println!("  --import-db-name    匯入目標資料庫（flow=import 時使用）");
    println!();
    println!("Ratatui Commands (--mode tui):");
    println!("  /set <field> <value>     設定參數，例如 /set output-format md");
    println!("  /unset <field>           清除可選參數（condition/order-by/compress/recsize/page-no）");
    println!("  /show                    顯示目前設定");
    println!("  /run                     執行匯出");
    println!("  /help                    顯示指令說明");
    println!("  /quit                    離開 TUI");
    println!();
    println!("Example:");
    println!(
        "  {program} --mode cli --flow export --export-db-type mssql --export-host 192.168.0.25 --export-port 1433 --export-user TRC201 --export-password 'syscom#1' --export-db-name TRC_RData --export-objects '^TRM_.*' --export-condition \"SnId>0\" --export-order-by \"SnId desc\" --export-hide-primary-key true --export-output-format sql --export-compress zip --export-delete-after-compress false --export-page-size 10000"
    );
}

fn parse_args() -> Result<ParseOutcome> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() <= 1 {
        let mut cli = default_tui_cli();
        apply_env_to_cli(&mut cli)?;
        return Ok(ParseOutcome::Cli(cli));
    }

    let mut base_cli = default_tui_cli();
    apply_env_to_cli(&mut base_cli)?;

    let mut mode = base_cli.mode;
    let mut flow = base_cli.flow;
    let mut lang = base_cli.lang;
    let mut db_type: Option<DbType> = Some(base_cli.db_type);
    let mut host: Option<String> = if base_cli.host.is_empty() {
        None
    } else {
        Some(base_cli.host)
    };
    let mut port: Option<u16> = Some(base_cli.port);
    let mut user: Option<String> = if base_cli.user.is_empty() {
        None
    } else {
        Some(base_cli.user)
    };
    let mut password: Option<String> = if base_cli.password.is_empty() {
        None
    } else {
        Some(base_cli.password)
    };
    let mut db_name: Option<String> = if base_cli.db_name.is_empty() {
        None
    } else {
        Some(base_cli.db_name)
    };
    let mut objects: String = base_cli.objects;
    let mut condition: Option<String> = base_cli.condition;
    let mut order_by: Option<String> = base_cli.order_by;
    let mut hide_primary_key = base_cli.hide_primary_key;
    let mut output_format = base_cli.output_format;
    let mut compress: Option<CompressType> = base_cli.compress;
    let mut delete_after_compress = base_cli.delete_after_compress;
    let mut output_directory = base_cli.output_directory;
    let mut recsize: Option<usize> = base_cli.recsize;
    let mut page_size: usize = base_cli.page_size;
    let mut page_no: Option<usize> = base_cli.page_no;
    let mut import_host: Option<String> = if base_cli.import_host.is_empty() {
        None
    } else {
        Some(base_cli.import_host)
    };
    let mut import_port: Option<u16> = Some(base_cli.import_port);
    let mut import_db_type: Option<DbType> = Some(base_cli.import_db_type);
    let mut import_user: Option<String> = if base_cli.import_user.is_empty() {
        None
    } else {
        Some(base_cli.import_user)
    };
    let mut import_password: Option<String> = if base_cli.import_password.is_empty() {
        None
    } else {
        Some(base_cli.import_password)
    };
    let mut import_db_name: Option<String> = if base_cli.import_db_name.is_empty() {
        None
    } else {
        Some(base_cli.import_db_name)
    };

    let mut i = 1usize;
    while i < args.len() {
        let key = &args[i];
        if key == "--help" || key == "-h" {
            return Ok(ParseOutcome::Help);
        }

        let need_value = |idx: usize, k: &str, all: &[String]| -> Result<String> {
            if idx + 1 >= all.len() {
                return Err(anyhow!("missing value for {k}"));
            }
            Ok(all[idx + 1].clone())
        };

        match key.as_str() {
            "--flow" => {
                flow = parse_flow_mode(&need_value(i, key, &args)?)?;
                i += 2;
            }
            "--mode" => {
                mode = parse_run_mode(&need_value(i, key, &args)?)?;
                i += 2;
            }
            "--lang" => {
                lang = parse_lang(&need_value(i, key, &args)?)?;
                i += 2;
            }
            "--export-db-type" | "--db-type" => {
                db_type = Some(parse_db_type(&need_value(i, key, &args)?)?);
                i += 2;
            }
            "--export-host" | "--host" => {
                host = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--export-port" | "--port" => {
                let p = need_value(i, key, &args)?
                    .parse::<u16>()
                    .with_context(|| "invalid --export-port value".to_string())?;
                if p == 0 {
                    return Err(anyhow!("invalid --export-port value: 0"));
                }
                port = Some(p);
                i += 2;
            }
            "--export-user" | "--user" => {
                user = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--export-password" | "--password" => {
                password = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--export-db-name" | "--db-name" => {
                db_name = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--export-objects" | "--export-tables" | "--objects" | "--tables" => {
                objects = need_value(i, key, &args)?;
                i += 2;
            }
            "--export-condition" | "--condition" => {
                condition = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--export-order-by" | "--order-by" => {
                order_by = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--export-hide-primary-key" | "--hide-primary-key" => {
                hide_primary_key = parse_bool_arg("--export-hide-primary-key", &need_value(i, key, &args)?)?;
                i += 2;
            }
            "--export-output-format" | "--output-format" => {
                output_format = parse_output_format(&need_value(i, key, &args)?)?;
                i += 2;
            }
            "--export-compress" | "--compress" => {
                compress = Some(parse_compress_type(&need_value(i, key, &args)?)?);
                i += 2;
            }
            "--export-delete-after-compress" | "--delete-after-compress" => {
                delete_after_compress =
                    parse_bool_arg("--export-delete-after-compress", &need_value(i, key, &args)?)?;
                i += 2;
            }
            "--export-output-directory" | "--output-directory" => {
                output_directory = PathBuf::from(need_value(i, key, &args)?);
                i += 2;
            }
            "--export-recsize" | "--recsize" => {
                recsize = Some(parse_positive_usize("--export-recsize", &need_value(i, key, &args)?)?);
                i += 2;
            }
            "--export-page-size" | "--page-size" => {
                page_size = parse_positive_usize("--export-page-size", &need_value(i, key, &args)?)?;
                i += 2;
            }
            "--export-page-no" | "--page-no" => {
                page_no = Some(parse_positive_usize("--export-page-no", &need_value(i, key, &args)?)?);
                i += 2;
            }
            "--import-host" => {
                import_host = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--import-port" => {
                let p = need_value(i, key, &args)?
                    .parse::<u16>()
                    .with_context(|| "invalid --import-port value".to_string())?;
                if p == 0 {
                    return Err(anyhow!("invalid --import-port value: 0"));
                }
                import_port = Some(p);
                i += 2;
            }
            "--import-db-type" => {
                import_db_type = Some(parse_db_type(&need_value(i, key, &args)?)?);
                i += 2;
            }
            "--import-user" => {
                import_user = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--import-password" => {
                import_password = Some(need_value(i, key, &args)?);
                i += 2;
            }
            "--import-db-name" => {
                import_db_name = Some(need_value(i, key, &args)?);
                i += 2;
            }
            _ => {
                return Err(anyhow!("unknown argument: {key}"));
            }
        }
    }

    let cli = Cli {
        flow,
        mode,
        lang,
        db_type: db_type.unwrap_or(DbType::Mssql),
        host: host.unwrap_or_default(),
        port: port.unwrap_or(1433),
        user: user.unwrap_or_default(),
        password: password.unwrap_or_default(),
        db_name: db_name.unwrap_or_default(),
        objects,
        condition,
        order_by,
        hide_primary_key,
        output_format,
        compress,
        delete_after_compress,
        output_directory,
        recsize,
        page_size,
        page_no,
        import_host: import_host.unwrap_or_default(),
        import_port: import_port.unwrap_or(1433),
        import_db_type: import_db_type.unwrap_or(DbType::Mssql),
        import_user: import_user.unwrap_or_default(),
        import_password: import_password.unwrap_or_default(),
        import_db_name: import_db_name.unwrap_or_default(),
    };

    if cli.mode == RunMode::Cli {
        match cli.flow {
            FlowMode::Export => {
                if cli.db_name.trim().is_empty() {
                    return Err(anyhow!("missing required --export-db-name (or DB2SQL_EXPORT_DB_NAME)"));
                }
                if cli.db_type != DbType::SqlLite {
                    if cli.host.trim().is_empty() {
                        return Err(anyhow!("missing required --export-host (or DB2SQL_EXPORT_HOST)"));
                    }
                    if cli.user.trim().is_empty() {
                        return Err(anyhow!("missing required --export-user (or DB2SQL_EXPORT_USER)"));
                    }
                    if cli.password.trim().is_empty() {
                        return Err(anyhow!("missing required --export-password (or DB2SQL_EXPORT_PASSWORD)"));
                    }
                }
            }
            FlowMode::Import => {
                if cli.import_db_name.trim().is_empty() {
                    return Err(anyhow!(
                        "missing required --import-db-name (or DB2SQL_IMPORT_DB_NAME)"
                    ));
                }
                if cli.import_db_type != DbType::SqlLite {
                    if cli.import_host.trim().is_empty() {
                        return Err(anyhow!(
                            "missing required --import-host (or DB2SQL_IMPORT_HOST)"
                        ));
                    }
                    if cli.import_user.trim().is_empty() {
                        return Err(anyhow!(
                            "missing required --import-user (or DB2SQL_IMPORT_USER)"
                        ));
                    }
                    if cli.import_password.trim().is_empty() {
                        return Err(anyhow!(
                            "missing required --import-password (or DB2SQL_IMPORT_PASSWORD)"
                        ));
                    }
                }
            }
        }
    }

    Ok(ParseOutcome::Cli(cli))
}
