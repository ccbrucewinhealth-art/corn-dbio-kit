# corn-dbio-kit

一個跨平台的資料庫匯入匯出工具，支持多種資料庫類型和多種匯出格式。

## 功能特點

- **資料庫支持**：MSSQL、Oracle、MySQL、PostgreSQL、SQLite
- **匯出格式**：SQL、SQLite、CSV、Excel、JSON、XML、Markdown
- **壓縮功能**：ZIP、BZIP2、TAR
- **介面模式**：CLI（命令列）和 TUI（互動式終端介面）
- **語言支持**：英文、繁體中文、日文
- **匯出功能**：單一資料表、透過pattern匯出多個資料表、清單檔案或自訂SQL
- **匯入功能**：SQLite 到其他資料庫

## 安裝

```bash
cd datahub-task/tools/corn-dbio-kit
cargo build --release
```

編譯後的二元檔案位於 `target/release/corn-dbio-kit`。

## 使用方式

### CLI 模式

```bash
./corn-dbio-kit --host localhost --port 1433 --user sa --password <password> --db-name mydb --objects "table_*" --output-format sql --output-dir ./export
```

### TUI 模式

```bash
./corn-dbio-kit --tui
```

## 命令列參數

| 參數 | 說明 | 預設值 |
|--------|------------|---------|
| `--host` | 資料庫伺服器主機 | - |
| `--port` | 資料庫伺服器連接埠 | - |
| `--user` | 資料庫使用者 | - |
| `--password` | 資料庫密碼 | - |
| `--db-name` | 資料庫名稱 | - |
| `--db-type` | 資料庫類型：mssql/oracle/mysql/postgresql/sqlite | mssql |
| `--objects` | 要匯出的資料表（pattern、@file 或 @sql:file） | - |
| `--condition` | WHERE 子句條件 | - |
| `--order-by` | ORDER BY 子句 | - |
| `--hide-primary-key` | 排除主鍵欄位 | false |
| `--output-format` | 輸出格式：sql/sqlite/csv/excel/json/xml/md | sql |
| `--output-dir` | 輸出目錄 | ./export |
| `--compress` | 壓縮輸出：zip/bzip/tar | - |
| `--delete-after-compress` | 壓縮後刪除檔案 | false |
| `--page-size` | 匯出每頁資料列數 | 10000 |
| `--page-no` | 要匯出的頁碼 | - |
| `--lang` | 語言：en/tw/ja | en |
| `--tui` | 使用互動式 TUI 模式 | false |
| `--import` | 執行匯入模式 | false |
| `--import-host` | 匯入目標主機 | - |
| `--import-port` | 匯入目標連接埠 | - |
| `--import-db-type` | 匯入目標類型 | - |
| `--import-user` | 匯入目標使用者 | - |
| `--import-password` | 匯入目標密碼 | - |
| `--import-db-name` | 匯入目標資料庫 | - |

## 物件選擇器語法

- **Pattern**：`table*` 或 `schema.table`
- **檔案清單**：`@/path/to/tables.txt`
- **SQL查詢**：`@sql:/path/to/query.sql`

## 環境變數

工具會從 `.env` 檔案載入配置：

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

## 使用範例

### 匯出符合pattern的所有資料表

```bash
./corn-dbio-kit --host localhost --user sa --password pass --db-name mydb --objects "tbl_*" --output-format csv --output-dir ./export
```

### 使用WHERE條件匯出

```bash
./corn-dbio-kit --host localhost --user sa --password pass --db-name mydb --objects "customers" --condition "status=1" --order-by "id DESC"
```

### 從檔案匯出特定資料表

```bash
./corn-dbio-kit --host localhost --user sa --password pass --db-name mydb --objects @tables.txt --output-format sqlite --output-dir ./export
```

### 壓縮後匯出

```bash
./corn-dbio-kit --host localhost --user sa --password pass --db-name mydb --objects "orders*" --compress zip --delete-after-compress
```

### 從SQLite匯入到資料庫

```bash
./corn-dbio-kit --import --import-host targetserver --import-port 1433 --import-user sa --import-password targetpass --import-db-name targetdb
```

### TUI 互動模式

```bash
./corn-dbio-kit --tui
```

## 輸出格式

- **sql**：SQL INSERT 語句
- **sqlite**：SQLite 資料庫檔案
- **csv**：CSV 檔案（每個資料表一個）
- **excel**：Excel 檔案（.xlsx）
- **json**：JSON 檔案
- **xml**：XML 檔案
- **md**：Markdown 表格格式

## 專案結構

```
src/
├── main.rs           # 進入點
├── db.rs            # 資料庫操作
├── tui.rs           # TUI 模組
├── app/
│   ├── cli_config_types.rs    # 設定類型
│   ├── cli_argument_parser.rs # CLI 參數解析
│   ├── export_runner.rs      # 匯出執行
│   ├── import_runner.rs     # 匯入執行
│   ├── table_page_export.rs # 資料表/分頁匯出
│   ├── object_query_export.rs # 物件查詢匯出
│   └── env_persistence.rs  # 環境變數儲存
```

## 相依套件

- Rust 2021 edition
- odbc-api
- rusqlite
- ratatui (TUI)
- crossterm

## 授權

僅供內部使用。