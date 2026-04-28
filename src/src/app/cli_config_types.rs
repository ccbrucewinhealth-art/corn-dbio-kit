const COL_KIND: usize = 8;
const COL_TABLE: usize = 30;
const COL_PAGE: usize = 8;
const COL_ROWS: usize = 12;
const COL_ELAPSED: usize = 12;
const COL_STATUS: usize = 8;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DbType {
    Mssql,
    Oracle,
    Mysql,
    Postgresql,
    SqlLite,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum OutputFormat {
    Sql,
    SqlLite,
    Csv,
    Excel,
    Json,
    Xml,
    Md,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ImportContentFormat {
    Sql,
    Csv,
    Md,
    Excel,
    SqlLite,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum CompressType {
    Zip,
    Bzip,
    Tar,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum RunMode {
    Cli,
    Tui,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum FlowMode {
    Export,
    Import,
    ExportSchema,
    ImportSchema,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Lang {
    Tw,
    En,
    Ja,
}

#[derive(Debug, Clone)]
struct Cli {
    flow: FlowMode,
    mode: RunMode,
    lang: Lang,
    db_type: DbType,
    host: String,
    port: u16,
    user: String,
    password: String,
    db_name: String,
    objects: String,
    condition: Option<String>,
    order_by: Option<String>,
    hide_primary_key: bool,
    output_format: OutputFormat,
    compress: Option<CompressType>,
    delete_after_compress: bool,
    output_directory: PathBuf,
    recsize: Option<usize>,
    page_size: usize,
    page_no: Option<usize>,
    export_collation: String,
    import_host: String,
    import_port: u16,
    import_db_type: DbType,
    import_user: String,
    import_password: String,
    import_db_name: String,
    import_collation: String,
    import_content_format: ImportContentFormat,
    import_content_src: PathBuf,
}

enum ParseOutcome {
    Cli(Cli),
    Help,
}
