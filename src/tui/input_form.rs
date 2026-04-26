fn apply_set_field(cli: &mut Cli, field: &str, value: &str) -> Result<()> {
    match field {
        "flow" => cli.flow = parse_flow_mode(value)?,
        "lang" => cli.lang = parse_lang(value)?,
        "export-db-type" => cli.db_type = parse_db_type(value)?,
        "export-host" => cli.host = value.to_string(),
        "export-port" => {
            let p = value
                .parse::<u16>()
                .with_context(|| format!("invalid export-port: {value}"))?;
            if p == 0 {
                return Err(anyhow!("invalid export-port: 0"));
            }
            cli.port = p;
        }
        "export-user" => cli.user = value.to_string(),
        "export-password" => cli.password = value.to_string(),
        "export-db-name" => cli.db_name = value.to_string(),
        "export-objects" => cli.objects = value.to_string(),
        "export-condition" => cli.condition = Some(value.to_string()),
        "export-order-by" => cli.order_by = Some(value.to_string()),
        "export-hide-primary-key" => {
            cli.hide_primary_key = parse_bool_arg("export-hide-primary-key", value)?
        }
        "export-output-format" => cli.output_format = parse_output_format(value)?,
        "export-compress" => cli.compress = Some(parse_compress_type(value)?),
        "export-delete-after-compress" => {
            cli.delete_after_compress = parse_bool_arg("export-delete-after-compress", value)?
        }
        "export-output-directory" => cli.output_directory = PathBuf::from(value),
        "export-recsize" => cli.recsize = Some(parse_positive_usize("export-recsize", value)?),
        "export-page-size" => cli.page_size = parse_positive_usize("export-page-size", value)?,
        "export-page-no" => cli.page_no = Some(parse_positive_usize("export-page-no", value)?),
        "import-host" => cli.import_host = value.to_string(),
        "import-port" => {
            let p = value
                .parse::<u16>()
                .with_context(|| format!("invalid import-port: {value}"))?;
            if p == 0 {
                return Err(anyhow!("invalid import-port: 0"));
            }
            cli.import_port = p;
        }
        "import-db-type" => cli.import_db_type = parse_db_type(value)?,
        "import-user" => cli.import_user = value.to_string(),
        "import-password" => cli.import_password = value.to_string(),
        "import-db-name" => cli.import_db_name = value.to_string(),
        _ => {
            return Err(anyhow!(
                "unsupported field: {field} (allowed: flow,db-type,host,port,user,password,db-name,objects,condition,order-by,hide-primary-key,output-format,compress,delete-after-compress,output-directory,recsize,page-size,page-no,import-host,import-port,import-db-type,import-user,import-password,import-db-name)"
            ))
        }
    }
    Ok(())
}

fn apply_unset_field(cli: &mut Cli, field: &str) -> Result<()> {
    match field {
        "export-condition" => cli.condition = None,
        "export-order-by" => cli.order_by = None,
        "export-compress" => cli.compress = None,
        "export-recsize" => cli.recsize = None,
        "export-page-no" => cli.page_no = None,
        _ => {
            return Err(anyhow!(
                "unsupported unset field: {field} (allowed: export-condition,export-order-by,export-compress,export-recsize,export-page-no)"
            ))
        }
    }
    Ok(())
}

const FORM_FIELDS: [&str; 26] = [
    "flow",
    "mode",
    "lang",
    "export-db-type",
    "export-host",
    "export-port",
    "export-user",
    "export-password",
    "export-db-name",
    "export-objects",
    "export-condition",
    "export-order-by",
    "export-hide-primary-key",
    "export-output-format",
    "export-compress",
    "export-delete-after-compress",
    "export-output-directory",
    "export-recsize",
    "export-page-size",
    "export-page-no",
    "import-host",
    "import-port",
    "import-db-type",
    "import-user",
    "import-password",
    "import-db-name",
];

fn field_options(field: &str) -> Option<Vec<&'static str>> {
    match field {
        "flow" => Some(vec!["export", "import"]),
        "mode" => Some(vec!["tui", "cli"]),
        "lang" => Some(vec!["tw", "en", "ja"]),
        "export-db-type" => Some(vec!["mssql", "oracle", "mysql", "postgresql", "sql-lite"]),
        "export-hide-primary-key" => Some(vec!["false", "true"]),
        "export-output-format" => Some(vec!["sql", "sql-lite", "csv", "excel", "json", "xml", "md"]),
        "export-compress" => Some(vec!["none", "zip", "bzip", "tar"]),
        "export-delete-after-compress" => Some(vec!["false", "true"]),
        "import-db-type" => Some(vec!["mssql", "oracle", "mysql", "postgresql", "sql-lite"]),
        _ => None,
    }
}

fn get_field_value(cli: &Cli, field: &str) -> String {
    match field {
        "flow" => flow_to_str(cli.flow).to_string(),
        "mode" => mode_to_str(cli.mode).to_string(),
        "lang" => lang_to_str(cli.lang).to_string(),
        "export-db-type" => db_type_to_str(cli.db_type).to_string(),
        "export-host" => cli.host.clone(),
        "export-port" => cli.port.to_string(),
        "export-user" => cli.user.clone(),
        "export-password" => cli.password.clone(),
        "export-db-name" => cli.db_name.clone(),
        "export-objects" => cli.objects.clone(),
        "export-condition" => cli.condition.clone().unwrap_or_default(),
        "export-order-by" => cli.order_by.clone().unwrap_or_default(),
        "export-hide-primary-key" => cli.hide_primary_key.to_string(),
        "export-output-format" => output_format_to_str(cli.output_format).to_string(),
        "export-compress" => cli
            .compress
            .map(|v| compress_to_str(v).to_string())
            .unwrap_or_else(|| "none".to_string()),
        "export-delete-after-compress" => cli.delete_after_compress.to_string(),
        "export-output-directory" => cli.output_directory.display().to_string(),
        "export-recsize" => cli.recsize.map(|v| v.to_string()).unwrap_or_default(),
        "export-page-size" => cli.page_size.to_string(),
        "export-page-no" => cli.page_no.map(|v| v.to_string()).unwrap_or_default(),
        "import-host" => cli.import_host.clone(),
        "import-port" => cli.import_port.to_string(),
        "import-db-type" => db_type_to_str(cli.import_db_type).to_string(),
        "import-user" => cli.import_user.clone(),
        "import-password" => cli.import_password.clone(),
        "import-db-name" => cli.import_db_name.clone(),
        _ => String::new(),
    }
}

fn set_field_value(cli: &mut Cli, field: &str, value: &str) -> Result<()> {
    match field {
        "export-compress" if value == "none" || value.trim().is_empty() => {
            cli.compress = None;
            Ok(())
        }
        "export-condition" | "export-order-by" | "export-recsize" | "export-page-no"
            if value.trim().is_empty() =>
        {
            apply_unset_field(cli, field)
        }
        _ => apply_set_field(cli, field, value),
    }
}

fn cycle_field_option(cli: &mut Cli, field: &str, dir: i32) -> Result<bool> {
    let opts = match field_options(field) {
        Some(v) => v,
        None => return Ok(false),
    };
    let cur = get_field_value(cli, field);
    let mut idx = opts
        .iter()
        .position(|v| *v == cur)
        .unwrap_or(0) as i32;
    idx = (idx + dir).rem_euclid(opts.len() as i32);
    set_field_value(cli, field, opts[idx as usize])?;
    Ok(true)
}
