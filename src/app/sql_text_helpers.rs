fn quote_ident(db_type: DbType, ident: &str) -> String {
    match db_type {
        DbType::Mssql => format!("[{}]", ident.replace(']', "]]")),
        DbType::Mysql => format!("`{}`", ident.replace('`', "``")),
        DbType::Postgresql | DbType::Oracle | DbType::SqlLite => {
            format!("\"{}\"", ident.replace('"', "\"\""))
        }
    }
}

fn escape_sql_value(raw: &str) -> String {
    raw.replace('\\', "\\\\").replace('\'', "''")
}

fn to_insert_value(value: Option<&str>) -> String {
    match value {
        Some(v) => format!("'{}'", escape_sql_value(v)),
        None => "NULL".to_string(),
    }
}

fn clip_text(text: &str, width: usize) -> String {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    if width == 1 {
        return "…".to_string();
    }
    let mut out = chars[..(width - 1)].iter().collect::<String>();
    out.push('…');
    out
}

fn print_export_line() {
    println!(
        "+-{:-<COL_KIND$}-+-{:-<COL_TABLE$}-+-{:-<COL_PAGE$}-+-{:-<COL_ROWS$}-+-{:-<COL_ELAPSED$}-+-{:-<COL_STATUS$}-+",
        "", "", "", "", "", ""
    );
}

fn print_export_header() {
    print_export_line();
    println!(
        "| {:<COL_KIND$} | {:<COL_TABLE$} | {:>COL_PAGE$} | {:>COL_ROWS$} | {:>COL_ELAPSED$} | {:<COL_STATUS$} |",
        "TYPE", "TABLE", "PAGE", "ROWS", "ELAPSED_MS", "STATUS"
    );
    print_export_line();
}

fn print_export_row(
    kind: &str,
    table: &str,
    page: Option<usize>,
    rows: usize,
    elapsed_ms: u128,
    status: &str,
) {
    let table_cell = clip_text(table, COL_TABLE);
    let page_cell = page
        .map(|v| v.to_string())
        .unwrap_or_else(|| "-".to_string());
    println!(
        "| {:<COL_KIND$} | {:<COL_TABLE$} | {:>COL_PAGE$} | {:>COL_ROWS$} | {:>COL_ELAPSED$} | {:<COL_STATUS$} |",
        clip_text(kind, COL_KIND),
        table_cell,
        page_cell,
        rows,
        elapsed_ms,
        clip_text(status, COL_STATUS)
    );
}


fn pagination_clause(db_type: DbType, offset: usize, limit: usize) -> String {
    match db_type {
        DbType::Mssql => format!(" OFFSET {offset} ROWS FETCH NEXT {limit} ROWS ONLY"),
        DbType::Mysql | DbType::Postgresql | DbType::SqlLite => {
            format!(" LIMIT {limit} OFFSET {offset}")
        }
        DbType::Oracle => format!(" OFFSET {offset} ROWS FETCH NEXT {limit} ROWS ONLY"),
    }
}

fn order_by_clause(db_type: DbType, order_by: Option<&str>) -> String {
    let order_by_trimmed = order_by.map(|v| v.trim()).unwrap_or("");
    if !order_by_trimmed.is_empty() {
        return format!(" ORDER BY {order_by_trimmed}");
    }

    match db_type {
        DbType::Mssql => " ORDER BY 1".to_string(),
        _ => String::new(),
    }
}

fn output_file_extension(format: OutputFormat) -> &'static str {
    match format {
        OutputFormat::Sql => "sql",
        OutputFormat::SqlLite => "sqlite",
        OutputFormat::Csv => "csv",
        OutputFormat::Excel => "xls",
        OutputFormat::Json => "json",
        OutputFormat::Xml => "xml",
        OutputFormat::Md => "md",
    }
}

fn escape_md_value(raw: &str) -> String {
    raw.replace('|', "\\|").replace('\n', " ").replace('\r', " ")
}

enum ObjectSelector {
    Pattern(String),
    ListFile(PathBuf),
    SqlFile(PathBuf),
}

fn parse_object_selector(raw: &str) -> ObjectSelector {
    if let Some(file) = raw.strip_prefix("@sql:") {
        ObjectSelector::SqlFile(PathBuf::from(file.trim()))
    } else if let Some(file) = raw.strip_prefix("@list:") {
        ObjectSelector::ListFile(PathBuf::from(file.trim()))
    } else {
        ObjectSelector::Pattern(raw.to_string())
    }
}

fn load_object_names_from_list_file(path: &Path) -> Result<HashSet<String>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read @list file: {}", path.display()))?;
    Ok(content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|s| s.to_string())
        .collect::<HashSet<_>>())
}
