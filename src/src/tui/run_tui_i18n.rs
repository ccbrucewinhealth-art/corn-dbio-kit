    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum FocusArea {
        Form,
        Input,
        DbPane,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum DbTab {
        Table,
        View,
        StoredProcedure,
    }

    let tr = |lang: Lang, key: &str| -> String {
        match (lang, key) {
            (Lang::Tw, "started") => "Ratatui 模式已啟動，輸入 /help 查看指令".to_string(),
            (Lang::En, "started") => "Ratatui mode started. type /help for commands".to_string(),
            (Lang::Ja, "started") => "Ratatui モードを開始しました。/help でコマンド一覧を表示".to_string(),
            (Lang::Tw, "commands_short") => "指令: /set /unset /show /show-hot-key /run /quit".to_string(),
            (Lang::En, "commands_short") => "commands: /set /unset /show /show-hot-key /run /quit".to_string(),
            (Lang::Ja, "commands_short") => "コマンド: /set /unset /show /show-hot-key /run /quit".to_string(),
            (Lang::Tw, "cfg_title") => "目前設定 (部分)".to_string(),
            (Lang::En, "cfg_title") => "Current Config (partial)".to_string(),
            (Lang::Ja, "cfg_title") => "現在の設定 (一部)".to_string(),
            (Lang::Tw, "cmd_title") => "指令".to_string(),
            (Lang::En, "cmd_title") => "Commands".to_string(),
            (Lang::Ja, "cmd_title") => "コマンド".to_string(),
            (Lang::Tw, "top_title") => " Corn DB I/O".to_string(),
            (Lang::En, "top_title") => " Corn DB I/O".to_string(),
            (Lang::Ja, "top_title") => " Corn DB I/O".to_string(),
            (Lang::Tw, "log_title") => "日誌".to_string(),
            (Lang::En, "log_title") => "Logs".to_string(),
            (Lang::Ja, "log_title") => "ログ".to_string(),
            (Lang::Tw, "input_title") => "輸入".to_string(),
            (Lang::En, "input_title") => "Input".to_string(),
            (Lang::Ja, "input_title") => "入力".to_string(),
            (Lang::Tw, "help_line") => "說明: /set <field> <value>, /unset <field>, /show, /run, /quit".to_string(),
            (Lang::En, "help_line") => "help: /set <field> <value>, /unset <field>, /show, /run, /quit".to_string(),
            (Lang::Ja, "help_line") => "ヘルプ: /set <field> <value>, /unset <field>, /show, /run, /quit".to_string(),
            (Lang::Tw, "set_usage") => "錯誤: 用法 /set <field> <value>".to_string(),
            (Lang::En, "set_usage") => "error: usage /set <field> <value>".to_string(),
            (Lang::Ja, "set_usage") => "エラー: 使い方 /set <field> <value>".to_string(),
            (Lang::Tw, "form_title") => "參數輸入表單 (↑↓:選欄位, Enter:編輯, ←→:下拉選項)".to_string(),
            (Lang::En, "form_title") => "Input Form (↑↓ select, Enter edit, ←→ dropdown)".to_string(),
            (Lang::Ja, "form_title") => "入力フォーム (↑↓選択, Enter編集, ←→ドロップダウン)".to_string(),
            (Lang::Tw, "editing") => "編輯中".to_string(),
            (Lang::En, "editing") => "editing".to_string(),
            (Lang::Ja, "editing") => "編集中".to_string(),
            (Lang::Tw, "dropdown") => "下拉".to_string(),
            (Lang::En, "dropdown") => "dropdown".to_string(),
            (Lang::Ja, "dropdown") => "ドロップダウン".to_string(),
            (Lang::Tw, "run_start") => "[tui] 以目前設定執行匯出...".to_string(),
            (Lang::En, "run_start") => "[tui] run export with current config...".to_string(),
            (Lang::Ja, "run_start") => "[tui] 現在の設定でエクスポートを実行...".to_string(),
            (Lang::Tw, "status_default") => "/run 進行執行".to_string(),
            (Lang::En, "status_default") => "/run to execute".to_string(),
            (Lang::Ja, "status_default") => "/run で実行".to_string(),
            (Lang::Tw, "status_tab_table") => "TAB=table".to_string(),
            (Lang::En, "status_tab_table") => "TAB=table".to_string(),
            (Lang::Ja, "status_tab_table") => "TAB=table".to_string(),
            (Lang::Tw, "status_tab_view") => "TAB=view".to_string(),
            (Lang::En, "status_tab_view") => "TAB=view".to_string(),
            (Lang::Ja, "status_tab_view") => "TAB=view".to_string(),
            (Lang::Tw, "status_tab_sp") => "TAB=stored procedure".to_string(),
            (Lang::En, "status_tab_sp") => "TAB=stored procedure".to_string(),
            (Lang::Ja, "status_tab_sp") => "TAB=stored procedure".to_string(),
            (Lang::Tw, "run_progress") => "執行中".to_string(),
            (Lang::En, "run_progress") => "Running".to_string(),
            (Lang::Ja, "run_progress") => "実行中".to_string(),
            (Lang::Tw, "press_enter") => "[tui] 按 Enter 返回 TUI".to_string(),
            (Lang::En, "press_enter") => "[tui] press Enter to return TUI".to_string(),
            (Lang::Ja, "press_enter") => "[tui] Enter キーで TUI に戻る".to_string(),
            (Lang::Tw, "run_ok") => "執行完成: 成功".to_string(),
            (Lang::En, "run_ok") => "run finished: success".to_string(),
            (Lang::Ja, "run_ok") => "実行完了: 成功".to_string(),
            (Lang::Tw, "run_fail") => "執行完成: 失敗".to_string(),
            (Lang::En, "run_fail") => "run finished: failed".to_string(),
            (Lang::Ja, "run_fail") => "実行完了: 失敗".to_string(),
            (Lang::Tw, "unknown_cmd") => "未知指令".to_string(),
            (Lang::En, "unknown_cmd") => "unknown command".to_string(),
            (Lang::Ja, "unknown_cmd") => "不明なコマンド".to_string(),
            (Lang::Tw, "ok_set") => "完成: 已設定".to_string(),
            (Lang::En, "ok_set") => "ok: set".to_string(),
            (Lang::Ja, "ok_set") => "完了: 設定".to_string(),
            (Lang::Tw, "ok_unset") => "完成: 已清除".to_string(),
            (Lang::En, "ok_unset") => "ok: unset".to_string(),
            (Lang::Ja, "ok_unset") => "完了: 解除".to_string(),
            (Lang::Tw, "err") => "錯誤".to_string(),
            (Lang::En, "err") => "error".to_string(),
            (Lang::Ja, "err") => "エラー".to_string(),
            (Lang::Tw, "focus_form") => "焦點: 表單".to_string(),
            (Lang::En, "focus_form") => "focus: form".to_string(),
            (Lang::Ja, "focus_form") => "フォーカス: フォーム".to_string(),
            (Lang::Tw, "focus_input") => "焦點: 輸入區".to_string(),
            (Lang::En, "focus_input") => "focus: input".to_string(),
            (Lang::Ja, "focus_input") => "フォーカス: 入力".to_string(),
            (Lang::Tw, "focus_db") => "焦點: DB 區".to_string(),
            (Lang::En, "focus_db") => "focus: db pane".to_string(),
            (Lang::Ja, "focus_db") => "フォーカス: DB ペイン".to_string(),
            (Lang::Tw, "input_title_focus") => "輸入 (TAB 已切至此區，可用 ↑↓ 歷史)".to_string(),
            (Lang::En, "input_title_focus") => "Input (TAB focus, ↑↓ history)".to_string(),
            (Lang::Ja, "input_title_focus") => "入力 (TABフォーカス, ↑↓履歴)".to_string(),
            (Lang::Tw, "run_cancel_hint") => "[Ctrl+Q] 取消匯出，返回主畫面".to_string(),
            (Lang::En, "run_cancel_hint") => "[Ctrl+Q] cancel export and return".to_string(),
            (Lang::Ja, "run_cancel_hint") => "[Ctrl+Q] エクスポート中止して戻る".to_string(),
            _ => key.to_string(),
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Length(ascii_art_ver1().len() as u16 + 4),
                Constraint::Percentage(20),
            ])
            .split(f.size());

        let splash_lines = ascii_art_ver1()
            .iter()
            .map(|l| Line::from(*l))
            .collect::<Vec<_>>();

        let splash = Paragraph::new(splash_lines)
            .block(Block::default().title("ratatui-splash-screen").borders(Borders::ALL));
        f.render_widget(splash, chunks[1]);
    })?;
    thread::sleep(Duration::from_millis(1200));

    let mut input = String::new();
    let mut selected_idx = 0usize;
    let mut editing_form = false;
    let mut form_input = String::new();
    let mut focus_area = FocusArea::Form;
    let mut cmd_history = Vec::<String>::new();
    let mut cmd_history_idx: Option<usize> = None;
    let mut db_tab = DbTab::Table;
    let mut table_stats = Vec::<TableStat>::new();
    let mut views = Vec::<String>::new();
    let mut sps = Vec::<String>::new();
    let mut db_selected_idx = 0usize;
    let mut table_edit_popup = false;
    let mut table_edit_cell = String::new();
    let mut table_edit_sql = String::new();
    let mut table_preview_cols = Vec::<String>::new();
    let mut table_preview_rows = Vec::<Vec<String>>::new();
    let mut table_edit_row_idx = 0usize;
    let mut table_edit_col_idx = 1usize;
    let mut sql_popup = false;
    let mut sql_input = String::new();
    let mut logs = vec![
        tr(cli.lang, "started").to_string(),
        tr(cli.lang, "commands_short").to_string(),
        "commands+: /test-connect /list-table /list-view /list-stored-procedure /exec-sql /show-hot-key".to_string(),
    ];

    let can_auto_refresh_db_tabs = !cli.host.trim().is_empty()
        && cli.port > 0
        && !cli.user.trim().is_empty()
        && !cli.password.trim().is_empty()
        && !cli.db_name.trim().is_empty();
    if can_auto_refresh_db_tabs {
        match list_table_stats(&cli) {
            Ok(v) => table_stats = v,
            Err(e) => logs.push(format!("auto refresh table stats failed: {e}")),
        }
        match tui_list_views(&cli) {
            Ok(v) => views = v,
            Err(e) => logs.push(format!("auto refresh views failed: {e}")),
        }
        match tui_list_stored_procedures(&cli) {
            Ok(v) => sps = v,
            Err(e) => logs.push(format!("auto refresh stored procedures failed: {e}")),
        }
    }
