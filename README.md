# corn-dbio-kit

A cross-platform CLI/TUI tool for database import/export operations. Supports multiple database types and various export formats.

## Features

- **Database Support**: MSSQL, Oracle, MySQL, PostgreSQL, SQLite
- **Export Formats**: SQL, SQLite, CSV, Excel, JSON, XML, Markdown
- **Compression**: ZIP, BZIP2, TAR
- **Interface Modes**: CLI (command-line) and TUI (interactive terminal UI)
- **Languages**: English, Traditional Chinese, Japanese
- **Export**: Single table, multiple tables by pattern, list file, or custom SQL
- **Import**: SQLite to other databases

## Installation

```bash
cd datahub-task/tools/corn-dbio-kit
cargo build --release
```

The compiled binary will be at `target/release/corn-dbio-kit`.

## Usage

### CLI Mode

```bash
./corn-dbio-kit --host localhost --port 1433 --user sa --password <password> --db-name mydb --objects "table_*" --output-format sql --output-dir ./export
```

### TUI Mode

```bash
./corn-dbio-kit --tui
```

## Command-Line Options

| Option                    | Description                                         | Default  |
| ------------------------- | --------------------------------------------------- | -------- |
| `--host`                  | Database server host                                | -        |
| `--port`                  | Database server port                                | -        |
| `--user`                  | Database user                                       | -        |
| `--password`              | Database password                                   | -        |
| `--db-name`               | Database name                                       | -        |
| `--db-type`               | Database type: mssql/oracle/mysql/postgresql/sqlite | mssql    |
| `--objects`               | Tables to export (pattern, @file, or @sql:file)     | -        |
| `--condition`             | WHERE clause condition                              | -        |
| `--order-by`              | ORDER BY clause                                     | -        |
| `--hide-primary-key`      | Exclude primary key columns                         | false    |
| `--output-format`         | Output format: sql/sqlite/csv/excel/json/xml/md     | sql      |
| `--output-dir`            | Output directory                                    | ./export |
| `--compress`              | Compress output: zip/bzip/tar                       | -        |
| `--delete-after-compress` | Delete files after compression                      | false    |
| `--page-size`             | Rows per page for export                            | 10000    |
| `--page-no`               | Page number to export                               | -        |
| `--lang`                  | Language: en/tw/ja                                  | en       |
| `--tui`                   | Use interactive TUI mode                            | false    |
| `--import`                | Run import mode                                     | false    |
| `--import-host`           | Import target host                                  | -        |
| `--import-port`           | Import target port                                  | -        |
| `--import-db-type`        | Import target type                                  | -        |
| `--import-user`           | Import target user                                  | -        |
| `--import-password`       | Import target password                              | -        |
| `--import-db-name`        | Import target database                              | -        |

## Object Selector Syntax

- **Pattern**: `table*` or `schema.table`
- **File list**: `@/path/to/tables.txt`
- **SQL query**: `@sql:/path/to/query.sql`

## Environment Variables

The tool loads configuration from `.env` file:

```
DB_HOST=localhost
DB_PORT=1433
DB_USER=sa
DB_PASSWORD=yourpassword
DB_NAME=mydb
DB_TYPE=mssql
OUTPUT_FORMAT=sql
OUTPUT_DIR=./export
LANG=tw
```

## Examples

### Export all tables matching pattern

```bash
./corn-dbio-kit --host localhost --user sa --password pass --db-name mydb --objects "tbl_*" --output-format csv --output-dir ./export
```

### Export with WHERE condition

```bash
./corn-dbio-kit --host localhost --user sa --password pass --db-name mydb --objects "customers" --condition "status=1" --order-by "id DESC"
```

### Export specific tables from file

```bash
./corn-dbio-kit --host localhost --user sa --password pass --db-name mydb --objects @tables.txt --output-format sqlite --output-dir ./export
```

### Export with compression

```bash
./corn-dbio-kit --host localhost --user sa --password pass --db-name mydb --objects "orders*" --compress zip --delete-after-compress
```

### Import from SQLite to database

```bash
./corn-dbio-kit --import --import-host targetserver --import-port 1433 --import-user sa --import-password targetpass --import-db-name targetdb
```

### TUI Interactive Mode

```bash
./corn-dbio-kit --tui
```

## Output Formats

- **sql**: SQL INSERT statements
- **sqlite**: SQLite database file
- **csv**: CSV files (one per table)
- **excel**: Excel files (.xlsx)
- **json**: JSON files
- **xml**: XML files
- **md**: Markdown table format

## Project Structure

```
src/
├── main.rs           # Entry point
├── db.rs            # Database operations
├── tui.rs           # TUI module
├── app/
│   ├── cli_config_types.rs    # Configuration types
│   ├── cli_argument_parser.rs # CLI argument parsing
│   ├── export_runner.rs      # Export execution
│   ├── import_runner.rs     # Import execution
│   ├── table_page_export.rs # Table/page export
│   ├── object_query_export.rs # Object query export
│   └── env_persistence.rs  # Environment persistence
```

## Dependencies

- Rust 2021 edition
- Native DB connectors: tiberius (SQL Server), mysql, postgres, oracle-rs
- rusqlite
- ratatui (TUI)
- crossterm

## License

Internal use only.
