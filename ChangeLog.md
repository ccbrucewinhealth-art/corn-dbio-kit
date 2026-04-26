# ChangeLog

## 2026-04-26

- 初始化 `corn-dbio-kit` Rust CLI 專案。
- 新增 CLI 參數與完整 help：`db-type`、`host`、`port`、`user`、`password`、`db-name`、`tables`、`condition`、`output-directory`、`recsize`、`page-size`、`page-no`。
- 實作 `.env` 載入規則：優先上層 `../.env`，否則使用目前目錄 `.env`。
- 實作支援資料庫型別：`mssql`、`oracle`、`mysql`、`postgresql`。
- 實作 tables regex 篩選與條件查詢分頁匯出。
- 實作 SQL 輸出檔名規則：`{table}_{page_index:06}.sql`。
- 新增工具腳本：`util_compile.sh`、`util_corn-dbio-kit.sh`、`util_corn-dbio-kit-loop-exec.sh`、`util_all-in-one-compile.sh`。
- 新增 `Makefile`。
- 更新離線套件下載腳本 `datahub-task/jobs/package/rust/modules/download_all_package-cron.sh` 指向 `corn-dbio-kit` 專案。
