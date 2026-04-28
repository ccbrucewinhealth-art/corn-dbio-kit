fn main() {
    let env_path = match try_load_env() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[error] {e}");
            std::process::exit(1);
        }
    };

    let program = env::args().next().unwrap_or_else(|| "corn-dbio-kit".to_string());
    let cli = match parse_args() {
        Ok(ParseOutcome::Cli(v)) => v,
        Ok(ParseOutcome::Help) => {
            print_help(&program);
            return;
        }
        Err(e) => {
            eprintln!("[error] {e}");
            eprintln!();
            print_help(&program);
            std::process::exit(2);
        }
    };

    let (result, cli_to_persist) = match cli.mode {
        RunMode::Cli => (
            match cli.flow {
                FlowMode::Export | FlowMode::ExportSchema => execute_export(&cli, false),
                FlowMode::Import | FlowMode::ImportSchema => execute_import(&cli, false),
            },
            cli.clone(),
        ),
        RunMode::Tui => match tui::run_tui(cli) {
            Ok(updated_cli) => (Ok(()), updated_cli),
            Err(e) => (Err(e), default_tui_cli()),
        },
    };

    if let Err(e) = persist_env_from_cli(&env_path, &cli_to_persist) {
        eprintln!("[error] persist env failed: {e}");
        std::process::exit(1);
    }

    if let Err(e) = result {
        eprintln!("[error] {e}");
        std::process::exit(1);
    }
}
