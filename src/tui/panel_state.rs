fn cli_snapshot_lines(cli: &Cli) -> Vec<String> {
    vec![
        format!("flow={}", flow_to_str(cli.flow)),
        format!("mode={:?}", cli.mode),
        format!("lang={:?}", cli.lang),
        format!("export-db-type={:?}", cli.db_type),
        format!("export-host={}", cli.host),
        format!("export-port={}", cli.port),
        format!("export-user={}", cli.user),
        format!("export-db-name={}", cli.db_name),
        format!("export-objects={}", cli.objects),
        format!("export-condition={}", cli.condition.clone().unwrap_or_default()),
        format!("export-order-by={}", cli.order_by.clone().unwrap_or_default()),
        format!("export-hide-primary-key={}", cli.hide_primary_key),
        format!("export-output-format={:?}", cli.output_format),
        format!(
            "export-compress={}",
            cli.compress
                .map(|v| format!("{:?}", v))
                .unwrap_or_else(|| "none".to_string())
        ),
        format!("export-delete-after-compress={}", cli.delete_after_compress),
        format!("export-output-directory={}", cli.output_directory.display()),
        format!("export-recsize={}", cli.recsize.map(|v| v.to_string()).unwrap_or_default()),
        format!("export-page-size={}", cli.page_size),
        format!("export-page-no={}", cli.page_no.map(|v| v.to_string()).unwrap_or_default()),
        format!("import-host={}", cli.import_host),
        format!("import-port={}", cli.import_port),
        format!("import-db-type={:?}", cli.import_db_type),
        format!("import-user={}", cli.import_user),
        format!("import-password={}", cli.import_password),
        format!("import-db-name={}", cli.import_db_name),
    ]
}
