pub(crate) fn run_tui(mut cli: Cli) -> Result<Cli> {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum FocusArea {
        Form,
        Input,
        DbPane,
        CmdPane,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum DbTab {
        Table,
        View,
        StoredProcedure,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum DbConnSource {
        Export,
        Import,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum ScreenPage {
        Main,
        DbOps,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    enum TableEditFocus {
        SqlEditor,
        Grid,
        CellInput,
        SaveBtn,
        InsertBtn,
        DeleteBtn,
        CancelBtn,
        RowForm,
        RowFormSaveBtn,
        RowFormCancelBtn,
    }

    let tr = |lang: Lang, key: &str| -> String {
        match (lang, key) {
            (Lang::Tw, "started") => "Ratatui 模式已啟動，輸入 /help 查看指令".to_string(),
            (Lang::En, "started") => "Ratatui mode started. type /help for commands".to_string(),
            (Lang::Ja, "started") => "Ratatui モードを開始しました。/help でコマンド一覧を表示".to_string(),
            (Lang::Tw, "commands_short") => "指令: /set /unset /show /show-hot-key /persist-to-sqlite /run-export-data /run-import-data /quit".to_string(),
            (Lang::En, "commands_short") => "commands: /set /unset /show /show-hot-key /persist-to-sqlite /run-export-data /run-import-data /quit".to_string(),
            (Lang::Ja, "commands_short") => "コマンド: /set /unset /show /show-hot-key /persist-to-sqlite /run-export-data /run-import-data /quit".to_string(),
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
            (Lang::Tw, "help_line") => "說明: /set <field> <value>, /unset <field>, /show, /persist-to-sqlite, /run-export-data, /run-import-data, /quit".to_string(),
            (Lang::En, "help_line") => "help: /set <field> <value>, /unset <field>, /show, /persist-to-sqlite, /run-export-data, /run-import-data, /quit".to_string(),
            (Lang::Ja, "help_line") => "ヘルプ: /set <field> <value>, /unset <field>, /show, /persist-to-sqlite, /run-export-data, /run-import-data, /quit".to_string(),
            (Lang::Tw, "set_usage") => "錯誤: 用法 /set <field> <value>".to_string(),
            (Lang::En, "set_usage") => "error: usage /set <field> <value>".to_string(),
            (Lang::Ja, "set_usage") => "エラー: 使い方 /set <field> <value>".to_string(),
            (Lang::Tw, "form_title") => "參數輸入表單 (↑↓:選欄位, Shift-Enter:編輯, ←→:下拉選項)".to_string(),
            (Lang::En, "form_title") => "Input Form (↑↓ select, Shift-Enter edit, ←→ dropdown)".to_string(),
            (Lang::Ja, "form_title") => "入力フォーム (↑↓選択, Shift-Enter編集, ←→ドロップダウン)".to_string(),
            (Lang::Tw, "disk_title") => "磁碟空間圓餅圖 (MB)".to_string(),
            (Lang::En, "disk_title") => "Disk Space Pie (MB)".to_string(),
            (Lang::Ja, "disk_title") => "ディスク使用量 円グラフ (MB)".to_string(),
            (Lang::Tw, "editing") => "編輯中".to_string(),
            (Lang::En, "editing") => "editing".to_string(),
            (Lang::Ja, "editing") => "編集中".to_string(),
            (Lang::Tw, "dropdown") => "下拉".to_string(),
            (Lang::En, "dropdown") => "dropdown".to_string(),
            (Lang::Ja, "dropdown") => "ドロップダウン".to_string(),
            (Lang::Tw, "run_start") => "[tui] 以目前設定執行匯出...".to_string(),
            (Lang::En, "run_start") => "[tui] run export with current config...".to_string(),
            (Lang::Ja, "run_start") => "[tui] 現在の設定でエクスポートを実行...".to_string(),
            (Lang::Tw, "run_start_import") => "[tui] 以目前設定執行匯入...".to_string(),
            (Lang::En, "run_start_import") => "[tui] run import with current config...".to_string(),
            (Lang::Ja, "run_start_import") => "[tui] 現在の設定でインポートを実行...".to_string(),
            (Lang::Tw, "status_default") => "/run-export-data 匯出，/run-import-data 匯入".to_string(),
            (Lang::En, "status_default") => "/run-export-data to export, /run-import-data to import".to_string(),
            (Lang::Ja, "status_default") => "/run-export-data でエクスポート、/run-import-data でインポート".to_string(),
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
    stdout.execute(EnableMouseCapture)?;
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
    let mut db_conn_source = DbConnSource::Export;
    let menu_items = vec![
        ("匯出資料", "__menu_export_data__"),
        ("匯入資料", "__menu_import_data__"),
        ("匯出結構", "__menu_export_schema__"),
        ("匯入結構", "__menu_import_schema__"),
        ("DB 操作", "__menu_db_ops__"),
        ("Help", "__menu_help__"),
    ];
    let mut menu_selected_idx = 0usize;
    let mut current_page = ScreenPage::Main;
    let cmd_items: Vec<(&str, &str)> = vec![
        ("/set <field> <value>", "設定欄位值"),
        ("/unset <field>", "清除可選欄位"),
        ("/show", "顯示目前設定"),
        ("/show-hot-key", "顯示快捷鍵"),
        ("/persist-to-sqlite", "建立 cort-dbio-kit.sqlite 並持久化"),
        ("/refresh-db", "重新載入 DB 清單"),
        ("/test-connect", "測試連線"),
        ("/list-table", "列出資料表"),
        ("/list-view", "列出檢視"),
        ("/list-stored-procedure", "列出預存程序"),
        ("/exec-sql", "開啟 SQL 執行視窗"),
        ("/run-export-data", "執行匯出"),
        ("/run-import-data", "執行匯入"),
        ("/quit", "離開 TUI"),
    ];
    let mut cmd_selected_idx = 0usize;
    let mut table_stats = Vec::<TableStat>::new();
    let mut views = Vec::<String>::new();
    let mut sps = Vec::<String>::new();
    let mut db_selected_idx = 0usize;
    let mut db_search_query = String::new();
    let mut db_search_last_input: Option<Instant> = None;
    let mut table_edit_popup = false;
    let mut table_edit_table_name = String::new();
    let mut table_edit_cell = String::new();
    let mut table_edit_focus = TableEditFocus::SqlEditor;
    let mut table_edit_sql = String::new();
    let mut table_page_no = 1usize;
    let table_page_size = 20usize;
    let mut table_preview_cols = Vec::<String>::new();
    let mut table_preview_rows = Vec::<Vec<String>>::new();
    let mut table_columns_info = Vec::<(String, String, String)>::new();
    let mut table_edit_row_idx = 0usize;
    let mut table_edit_col_idx = 1usize;
    let mut row_form_popup = false;
    let mut row_form_values = Vec::<String>::new();
    let mut row_form_original = Vec::<String>::new();
    let mut row_form_idx = 1usize;
    let mut row_form_insert_mode = false;
    let mut row_form_pk_auto_inc = std::collections::HashSet::<String>::new();
    let mut object_def_popup = false;
    let mut object_def_name = String::new();
    let mut object_def_sql = String::new();
    let mut object_def_is_sp = false;
    let mut dlg_save_btn: Option<Rect> = None;
    let mut dlg_insert_btn: Option<Rect> = None;
    let mut dlg_delete_btn: Option<Rect> = None;
    let mut dlg_cancel_btn: Option<Rect> = None;
    let mut sql_popup = false;
    let mut sql_input = String::new();
    let mut help_popup = false;
    let mut logs = vec![
        tr(cli.lang, "started").to_string(),
        tr(cli.lang, "commands_short").to_string(),
        "commands+: /test-connect /list-table /list-view /list-stored-procedure /exec-sql /show-hot-key".to_string(),
    ];
    let mut disk_total_mb = 0u64;
    let mut disk_used_mb = 0u64;
    let mut disk_filesystem = String::new();
    let mut disk_mount_on = String::new();
    let mut disk_size_h = String::new();
    let mut disk_used_h = String::new();
    let mut disk_avail_h = String::new();
    let mut disk_use_pct_h = String::new();
    let mut disk_last_update: Option<Instant> = None;
    let theme_bg = Color::Rgb(165, 210, 255);
    let theme_bar = Color::Rgb(62, 130, 215);
    let theme_panel = Color::Rgb(32, 92, 176);
    let theme_panel_active = Color::Rgb(18, 64, 138);
    let theme_text = Color::Rgb(225, 232, 248);
    let theme_dim = Color::Rgb(150, 162, 192);
    let theme_border = Color::Rgb(88, 108, 165);
    let theme_focus = Color::Rgb(255, 205, 64);

    let query_disk_usage = |path: &Path| -> Option<(u64, u64, String, String, String, String, String, String)> {
        let parse_first_data_line = |raw: &str| -> Option<Vec<String>> {
            let line = raw.lines().nth(1)?;
            Some(
                line.split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>(),
            )
        };

        let out_num = Command::new("df").arg("-B1").arg(path).output().ok()?;
        if !out_num.status.success() {
            return None;
        }
        let num_text = String::from_utf8_lossy(&out_num.stdout);
        let num_cols = parse_first_data_line(&num_text)?;
        if num_cols.len() < 6 {
            return None;
        }
        let total_mb = num_cols[1].parse::<u64>().ok()? / (1024 * 1024);
        let used_mb = num_cols[2].parse::<u64>().ok()? / (1024 * 1024);

        let out_h = Command::new("df").arg("-h").arg(path).output().ok()?;
        if !out_h.status.success() {
            return None;
        }
        let h_text = String::from_utf8_lossy(&out_h.stdout);
        let h_cols = parse_first_data_line(&h_text)?;
        if h_cols.len() < 6 {
            return None;
        }

        Some((
            total_mb,
            used_mb,
            h_cols[0].clone(), // filesystem
            h_cols[1].clone(), // size
            h_cols[2].clone(), // used
            h_cols[3].clone(), // avail
            h_cols[4].clone(), // use%
            h_cols[5].clone(), // mounted on
        ))
    };

    let build_pie_lines = |used_pct: f64| -> Vec<Line<'static>> {
        let mut grid = vec![vec![' '; 9]; 5];
        let slots = [(4usize, 0usize), (6, 1), (7, 2), (6, 3), (4, 4), (2, 3), (1, 2), (2, 1)];
        let filled = ((used_pct * 8.0).round() as usize).min(8);
        for (idx, (x, y)) in slots.iter().enumerate() {
            grid[*y][*x] = if idx < filled { '●' } else { '○' };
        }
        grid[2][4] = '◉';
        grid.into_iter()
            .map(|row| Line::from(row.into_iter().collect::<String>()))
            .collect::<Vec<_>>()
    };

    let build_db_panel_cli = |base: &Cli, source: DbConnSource| -> Cli {
        let mut x = base.clone();
        if source == DbConnSource::Import {
            x.db_type = base.import_db_type;
            x.host = base.import_host.clone();
            x.port = base.import_port;
            x.user = base.import_user.clone();
            x.password = base.import_password.clone();
            x.db_name = base.import_db_name.clone();
        }
        x
    };
    let refresh_db_panel = |base: &Cli,
                            source: DbConnSource,
                            table_stats: &mut Vec<TableStat>,
                            views: &mut Vec<String>,
                            sps: &mut Vec<String>,
                            logs: &mut Vec<String>| {
        let db_cli = build_db_panel_cli(base, source);
        let can_auto_refresh = !db_cli.host.trim().is_empty()
            && db_cli.port > 0
            && !db_cli.user.trim().is_empty()
            && !db_cli.password.trim().is_empty()
            && !db_cli.db_name.trim().is_empty();
        if !can_auto_refresh {
            table_stats.clear();
            views.clear();
            sps.clear();
            logs.push(format!(
                "db panel ({}) missing connection fields",
                if source == DbConnSource::Export {
                    "export"
                } else {
                    "import"
                }
            ));
            return;
        }
        match list_table_stats(&db_cli) {
            Ok(v) => *table_stats = v,
            Err(e) => logs.push(format!("auto refresh table stats failed: {e}")),
        }
        match tui_list_views(&db_cli) {
            Ok(v) => *views = v,
            Err(e) => logs.push(format!("auto refresh views failed: {e}")),
        }
        match tui_list_stored_procedures(&db_cli) {
            Ok(v) => *sps = v,
            Err(e) => logs.push(format!("auto refresh stored procedures failed: {e}")),
        }
    };
    refresh_db_panel(
        &cli,
        db_conn_source,
        &mut table_stats,
        &mut views,
        &mut sps,
        &mut logs,
    );
    let run_result = (|| -> Result<()> {
        loop {
            let need_refresh_disk = disk_last_update
                .map(|t| t.elapsed() >= Duration::from_secs(3))
                .unwrap_or(true);
            if need_refresh_disk {
                if let Some((total, used, fs_name, size_h, used_h, avail_h, use_pct_h, mount_on)) =
                    query_disk_usage(Path::new("."))
                {
                    disk_total_mb = total;
                    disk_used_mb = used.min(total);
                    disk_filesystem = fs_name;
                    disk_size_h = size_h;
                    disk_used_h = used_h;
                    disk_avail_h = avail_h;
                    disk_use_pct_h = use_pct_h;
                    disk_mount_on = mount_on;
                }
                disk_last_update = Some(Instant::now());
            }

            terminal.draw(|f| {
                let bg = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(30),
                        Constraint::Percentage(35),
                        Constraint::Percentage(35),
                    ])
                    .split(f.size());
                f.render_widget(
                    Block::default().style(Style::default().bg(Color::Rgb(185, 225, 255))),
                    bg[0],
                );
                f.render_widget(
                    Block::default().style(Style::default().bg(Color::Rgb(95, 165, 235))),
                    bg[1],
                );
                f.render_widget(
                    Block::default().style(Style::default().bg(Color::Rgb(28, 78, 156))),
                    bg[2],
                );

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(12),
                        Constraint::Min(6),
                        Constraint::Length(3),
                        Constraint::Length(1),
                    ])
                    .split(f.size());

                let mut top_spans = vec![Span::styled(
                    format!(" {} ", tr(cli.lang, "top_title")),
                    Style::default().fg(theme_text).bg(theme_bar),
                )];
                for (idx, (label, _cmd)) in menu_items.iter().enumerate() {
                    top_spans.push(Span::raw(" "));
                    let is_sel = idx == menu_selected_idx;
                    top_spans.push(Span::styled(
                        format!(" {label} "),
                        if is_sel {
                            Style::default().fg(Color::Black).bg(theme_focus)
                        } else {
                            Style::default().fg(theme_dim).bg(theme_bar)
                        },
                    ));
                }
                top_spans.push(Span::raw("  "));
                top_spans.push(Span::styled(
                    "F8/F9/F12",
                    Style::default().fg(theme_focus).bg(theme_bar),
                ));
                top_spans.push(Span::styled(
                    "    Help!!",
                    Style::default().fg(theme_dim).bg(theme_bar),
                ));
                let top_title = Paragraph::new(Line::from(top_spans)).style(Style::default().bg(theme_bar));
                f.render_widget(top_title, chunks[0]);

                let top = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(55),
                        Constraint::Percentage(10),
                        Constraint::Percentage(35),
                    ])
                    .split(chunks[1]);

                let form_visible = top[0].height.saturating_sub(2) as usize;
                let form_visible = form_visible.max(1);
                let mut form_start = 0usize;
                if selected_idx >= form_visible {
                    form_start = selected_idx + 1 - form_visible;
                }

                let form_lines = FORM_FIELDS
                    .iter()
                    .enumerate()
                    .skip(form_start)
                    .take(form_visible)
                    .map(|(idx, field)| {
                        let value = if editing_form && idx == selected_idx {
                            form_input.clone()
                        } else {
                            get_field_value(&cli, field)
                        };
                        let prefix = if idx == selected_idx { ">" } else { " " };
                        let dd = field_options(field)
                            .map(|opts| format!("  [{}:{}]", tr(cli.lang, "dropdown"), opts.join("|")))
                            .unwrap_or_default();
                        let edit_mark = if editing_form && idx == selected_idx {
                            format!("  <{}>", tr(cli.lang, "editing"))
                        } else {
                            String::new()
                        };
                        Line::from(format!("{prefix} {field:<20} = {value}{dd}{edit_mark}"))
                    })
                    .collect::<Vec<_>>();

                let form_is_active = focus_area == FocusArea::Form;
                let form_block = {
                    let b = Block::default()
                        .title(tr(cli.lang, "form_title"))
                        .borders(Borders::ALL);
                    if form_is_active {
                        b.border_style(Style::default().fg(theme_focus))
                            .style(Style::default().fg(theme_text).bg(theme_panel_active))
                    } else {
                        b.border_style(Style::default().fg(theme_border))
                            .style(Style::default().fg(theme_text).bg(theme_panel))
                    }
                };
                let state_widget = Paragraph::new(form_lines)
                    .style(if form_is_active {
                        Style::default().fg(theme_text).bg(theme_panel_active)
                    } else {
                        Style::default().fg(theme_text).bg(theme_panel)
                    })
                    .block(form_block);
                f.render_widget(state_widget, top[0]);

                let used_pct = if disk_total_mb == 0 {
                    0.0
                } else {
                    (disk_used_mb as f64) / (disk_total_mb as f64)
                };
                let unused_mb = disk_total_mb.saturating_sub(disk_used_mb);
                let mut pie_lines = build_pie_lines(used_pct);
                pie_lines.push(Line::from(""));
                pie_lines.push(Line::from(format!("used   : {} MB", disk_used_mb)));
                pie_lines.push(Line::from(format!("avail  : {} MB", unused_mb)));
                pie_lines.push(Line::from(format!("total  : {} MB", disk_total_mb)));
                pie_lines.push(Line::from(format!("ratio  : {:.1}%", used_pct * 100.0)));
                pie_lines.push(Line::from("-- df -h --"));
                pie_lines.push(Line::from(format!("fs     : {}", disk_filesystem)));
                pie_lines.push(Line::from(format!("size   : {}", disk_size_h)));
                pie_lines.push(Line::from(format!("used   : {}", disk_used_h)));
                pie_lines.push(Line::from(format!("avail  : {}", disk_avail_h)));
                pie_lines.push(Line::from(format!("use%   : {}", disk_use_pct_h)));
                pie_lines.push(Line::from(format!("mount  : {}", disk_mount_on)));
                let disk_widget = Paragraph::new(pie_lines).style(Style::default().fg(theme_text).bg(theme_panel)).block(
                    Block::default()
                        .title(tr(cli.lang, "disk_title"))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme_border))
                        .style(Style::default().bg(theme_panel)),
                );
                f.render_widget(disk_widget, top[1]);

                let cmd_visible = top[2].height.saturating_sub(2) as usize;
                let cmd_visible = cmd_visible.max(1);
                let cmd_selected = cmd_selected_idx.min(cmd_items.len().saturating_sub(1));
                let cmd_start = if cmd_selected >= cmd_visible {
                    cmd_selected + 1 - cmd_visible
                } else {
                    0
                };
                let cmd_lines = cmd_items
                    .iter()
                    .enumerate()
                    .skip(cmd_start)
                    .take(cmd_visible)
                    .map(|(idx, (cmd, desc))| {
                        let prefix = if idx == cmd_selected { ">" } else { " " };
                        Line::from(format!("{prefix} {cmd:<28} {desc}"))
                    })
                    .collect::<Vec<_>>();
                let cmd_is_active = focus_area == FocusArea::CmdPane;
                let cmd_block = {
                    let b = Block::default()
                        .title(format!(
                            "{} ({}/{})",
                            tr(cli.lang, "cmd_title"),
                            cmd_selected + 1,
                            cmd_items.len()
                        ))
                        .borders(Borders::ALL);
                    if cmd_is_active {
                        b.border_style(Style::default().fg(theme_focus))
                            .style(Style::default().fg(theme_text).bg(theme_panel_active))
                    } else {
                        b.border_style(Style::default().fg(theme_border))
                            .style(Style::default().fg(theme_text).bg(theme_panel))
                    }
                };
                let cmd_widget = Paragraph::new(cmd_lines)
                    .style(if cmd_is_active {
                        Style::default().fg(theme_text).bg(theme_panel_active)
                    } else {
                        Style::default().fg(theme_text).bg(theme_panel)
                    })
                    .block(cmd_block);
                f.render_widget(cmd_widget, top[2]);

                let log_lines = logs
                    .iter()
                    .rev()
                    .take((chunks[1].height as usize).saturating_sub(2))
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .map(Line::from)
                    .collect::<Vec<_>>();
                let log_widget = Paragraph::new(log_lines).style(Style::default().fg(theme_text).bg(theme_panel)).block(
                    Block::default()
                        .title(tr(cli.lang, "log_title"))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(theme_border))
                        .style(Style::default().bg(theme_panel)),
                );
                if current_page == ScreenPage::DbOps {
                    let logs_area = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
                        .split(chunks[2]);
                    f.render_widget(log_widget, logs_area[0]);

                    let db_visible = (logs_area[1].height as usize).saturating_sub(2).max(1);
                    let db_len = match db_tab {
                        DbTab::Table => table_stats.len(),
                        DbTab::View => views.len(),
                        DbTab::StoredProcedure => sps.len(),
                    };
                    let db_selected = if db_len == 0 {
                        0
                    } else {
                        db_selected_idx.min(db_len.saturating_sub(1))
                    };
                    let db_start = if db_selected >= db_visible {
                        db_selected + 1 - db_visible
                    } else {
                        0
                    };
                    let db_source_text = if db_conn_source == DbConnSource::Export {
                        "export"
                    } else {
                        "import"
                    };
                    let mut db_title = match db_tab {
                        DbTab::Table => format!(
                            "DB[{db_source_text}]: table ({}/{}) | view ({}) | stored procedure ({})",
                            if db_len == 0 { 0 } else { db_selected + 1 },
                            table_stats.len(),
                            views.len(),
                            sps.len()
                        ),
                        DbTab::View => format!(
                            "DB[{db_source_text}]: table ({}) | view ({}/{}) | stored procedure ({})",
                            table_stats.len(),
                            if db_len == 0 { 0 } else { db_selected + 1 },
                            views.len(),
                            sps.len()
                        ),
                        DbTab::StoredProcedure => format!(
                            "DB[{db_source_text}]: table ({}) | view ({}) | stored procedure ({}/{})",
                            table_stats.len(),
                            views.len(),
                            if db_len == 0 { 0 } else { db_selected + 1 },
                            sps.len()
                        ),
                    };
                    if !db_search_query.is_empty() {
                        db_title.push_str(&format!(" | find:{}", db_search_query));
                    }
                    let db_lines = match db_tab {
                        DbTab::Table => table_stats
                            .iter()
                            .enumerate()
                            .skip(db_start)
                            .take(db_visible)
                            .map(|(idx, t)| {
                                let prefix = if idx == db_selected { ">" } else { " " };
                                Line::from(format!(
                                    "{prefix} {} | rows={} | size={}",
                                    t.name, t.rows, t.size_bytes
                                ))
                            })
                            .collect::<Vec<_>>(),
                        DbTab::View => views
                            .iter()
                            .enumerate()
                            .skip(db_start)
                            .take(db_visible)
                            .map(|(idx, v)| {
                                let prefix = if idx == db_selected { ">" } else { " " };
                                Line::from(format!("{prefix} {v}"))
                            })
                            .collect::<Vec<_>>(),
                        DbTab::StoredProcedure => sps
                            .iter()
                            .enumerate()
                            .skip(db_start)
                            .take(db_visible)
                            .map(|(idx, v)| {
                                let prefix = if idx == db_selected { ">" } else { " " };
                                Line::from(format!("{prefix} {v}"))
                            })
                            .collect::<Vec<_>>(),
                    };
                    let db_is_active = focus_area == FocusArea::DbPane;
                    let db_block = {
                        let b = Block::default().title(db_title).borders(Borders::ALL);
                        if db_is_active {
                            b.border_style(Style::default().fg(theme_focus))
                                .style(Style::default().fg(theme_text).bg(theme_panel_active))
                        } else {
                            b.border_style(Style::default().fg(theme_border))
                                .style(Style::default().fg(theme_text).bg(theme_panel))
                        }
                    };
                    let db_widget = Paragraph::new(db_lines)
                        .style(if db_is_active {
                            Style::default().fg(theme_text).bg(theme_panel_active)
                        } else {
                            Style::default().fg(theme_text).bg(theme_panel)
                        })
                        .block(db_block);
                    f.render_widget(db_widget, logs_area[1]);
                } else {
                    f.render_widget(log_widget, chunks[2]);
                }

                let input_is_active = focus_area == FocusArea::Input;
                let input_block = {
                    let b = Block::default()
                        .title(if input_is_active {
                            tr(cli.lang, "input_title_focus")
                        } else {
                            tr(cli.lang, "input_title")
                        })
                        .borders(Borders::ALL);
                    if input_is_active {
                        b.border_style(Style::default().fg(theme_focus))
                            .style(Style::default().fg(theme_text).bg(theme_panel_active))
                    } else {
                        b.border_style(Style::default().fg(theme_border))
                            .style(Style::default().fg(theme_text).bg(theme_panel))
                    }
                };
                let input_widget = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "> ",
                        if input_is_active {
                            Style::default().fg(theme_focus).bg(theme_panel_active)
                        } else {
                            Style::default().fg(theme_focus).bg(theme_panel)
                        },
                    ),
                    Span::styled(
                        input.as_str(),
                        if input_is_active {
                            Style::default().fg(theme_text).bg(theme_panel_active)
                        } else {
                            Style::default().fg(theme_text).bg(theme_panel)
                        },
                    ),
                ]))
                .style(if input_is_active {
                    Style::default().fg(theme_text).bg(theme_panel_active)
                } else {
                    Style::default().fg(theme_text).bg(theme_panel)
                })
                .block(input_block);
                f.render_widget(input_widget, chunks[3]);

                let status_widget = Paragraph::new(Line::from(vec![
                    Span::styled(" Q/Esc ", Style::default().fg(Color::Black).bg(theme_focus)),
                    Span::styled(" Quit  ", Style::default().fg(theme_dim).bg(theme_bg)),
                    Span::styled(" Tab ", Style::default().fg(Color::Black).bg(theme_focus)),
                    Span::styled(" Next Pane  ", Style::default().fg(theme_dim).bg(theme_bg)),
                    Span::styled(" ↑/k ", Style::default().fg(Color::Black).bg(theme_focus)),
                    Span::styled(" Up  ", Style::default().fg(theme_dim).bg(theme_bg)),
                    Span::styled(" ↓/j ", Style::default().fg(Color::Black).bg(theme_focus)),
                    Span::styled(" Down  ", Style::default().fg(theme_dim).bg(theme_bg)),
                    Span::styled(" F8 ", Style::default().fg(Color::Black).bg(theme_focus)),
                    Span::styled(" Next  ", Style::default().fg(theme_dim).bg(theme_bg)),
                    Span::styled(" F9 ", Style::default().fg(Color::Black).bg(theme_focus)),
                    Span::styled(" Apply", Style::default().fg(theme_dim).bg(theme_bg)),
                    Span::styled("  F12 ", Style::default().fg(Color::Black).bg(theme_focus)),
                    Span::styled(" Help", Style::default().fg(theme_dim).bg(theme_bg)),
                ]))
                .style(Style::default().bg(theme_bg));
                f.render_widget(status_widget, chunks[4]);

                if help_popup {
                    let area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(15),
                            Constraint::Length(16),
                            Constraint::Percentage(15),
                        ])
                        .split(f.size())[1];
                    f.render_widget(Clear, area);
                    let help_lines = vec![
                        Line::from("操作手冊"),
                        Line::from("----------------------------------------------"),
                        Line::from("選單"),
                        Line::from("1) 匯出   : 設定 flow=export-data 並執行 /run-export-data"),
                        Line::from("2) 匯入   : 設定 flow=import-data 並執行 /run-import-data"),
                        Line::from("3) DB 操作: 進入 DB 操作頁 (含 Table/View/SP/SQL)"),
                        Line::from("4) Help   : 開啟本說明視窗"),
                        Line::from(""),
                        Line::from("快捷鍵"),
                        Line::from("F8: 選下一個選單   F9: 套用選單   F12: Help"),
                        Line::from("Tab: 切換焦點       Shift-Tab: 切換 DB 分頁(僅 DB 操作頁)"),
                        Line::from("Esc: 關閉視窗/離開  Ctrl+Q: 取消執行中工作"),
                        Line::from(""),
                        Line::from("[Esc/F12] Close"),
                    ];
                    let help_widget = Paragraph::new(help_lines).block(
                        Block::default()
                            .title("Help / 操作手冊")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme_focus))
                            .style(Style::default().fg(theme_text).bg(theme_panel_active)),
                    );
                    f.render_widget(help_widget, area);
                }

                if sql_popup {
                    let area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(25),
                            Constraint::Length(8),
                            Constraint::Percentage(25),
                        ])
                        .split(f.size())[1];
                    f.render_widget(Clear, area);
                    let sql_widget = Paragraph::new(vec![
                        Line::from("/exec-sql"),
                        Line::from("--------------------------------"),
                        Line::from(sql_input.as_str()),
                        Line::from(""),
                        Line::from("[Enter] Execute   [Esc] Close"),
                    ])
                    .block(Block::default().title("SQL Popup").borders(Borders::ALL));
                    f.render_widget(sql_widget, area);
                }

                if table_edit_popup {
                    let area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(7),
                            Constraint::Percentage(86),
                            Constraint::Percentage(7),
                        ])
                        .split(f.size())[1];
                    f.render_widget(Clear, area);
                    let outer = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
                        .split(area);
                    let left = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(25),
                            Constraint::Percentage(75),
                            Constraint::Length(3),
                        ])
                        .split(outer[0]);

                    let sql_widget = Paragraph::new(vec![
                        Line::from("table edit sql"),
                        Line::from("------------------------------------------------"),
                        Line::from(table_edit_sql.as_str()),
                        Line::from(if table_edit_focus == TableEditFocus::SqlEditor {
                            "<SQL 編輯中，可按 Tab 切焦點；Enter 套用 SQL 到 GRID>"
                        } else {
                            "[Tab] 切到 SQL 編輯"
                        }),
                    ])
                    .block(
                        Block::default()
                            .title("Row Data Selected SQL")
                            .borders(Borders::ALL)
                            .border_style(if table_edit_focus == TableEditFocus::SqlEditor {
                                Style::default().fg(theme_focus)
                            } else {
                                Style::default()
                            }),
                    );
                    f.render_widget(sql_widget, left[0]);

                    let mut grid_lines = vec![];
                    if !table_preview_cols.is_empty() {
                        grid_lines.push(Line::from(table_preview_cols.join(" | ")));
                        grid_lines.push(Line::from("------------------------------------------------"));
                    }
                    for (ri, row) in table_preview_rows.iter().enumerate() {
                        let marker = if ri == table_edit_row_idx { ">" } else { " " };
                        grid_lines.push(Line::from(format!("{marker} {}", row.join(" | "))));
                    }
                    grid_lines.push(Line::from(format!(
                        "selected row={} col={} (PK col=0 不可編輯)",
                        table_edit_row_idx, table_edit_col_idx
                    )));
                    grid_lines.push(Line::from(format!(
                        "page={} size={} | edit cell: {} | PgUp/PgDn: page",
                        table_page_no, table_page_size, table_edit_cell
                    )));
                    let grid_widget = Paragraph::new(grid_lines)
                        .block(
                            Block::default()
                                .title("Data Editor")
                                .borders(Borders::ALL)
                                .border_style(if table_edit_focus == TableEditFocus::Grid {
                                    Style::default().fg(theme_focus)
                                } else {
                                    Style::default()
                                }),
                        );
                    f.render_widget(grid_widget, left[1]);

                    let btns = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(20),
                            Constraint::Length(20),
                            Constraint::Length(20),
                            Constraint::Length(20),
                            Constraint::Min(1),
                        ])
                        .split(left[2]);
                    dlg_save_btn = Some(btns[0]);
                    dlg_insert_btn = Some(btns[1]);
                    dlg_delete_btn = Some(btns[2]);
                    dlg_cancel_btn = Some(btns[3]);
                    let save_btn = Paragraph::new("[ 儲存 Ctrl+F2 ]").block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(if table_edit_focus == TableEditFocus::SaveBtn {
                                Style::default().fg(theme_focus)
                            } else {
                                Style::default()
                            }),
                    );
                    let insert_btn = Paragraph::new("[ 新增 Ctrl+F4 ]").block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(if table_edit_focus == TableEditFocus::InsertBtn {
                                Style::default().fg(theme_focus)
                            } else {
                                Style::default()
                            }),
                    );
                    let delete_btn = Paragraph::new("[ 刪除 Ctrl+F5 ]").block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(if table_edit_focus == TableEditFocus::DeleteBtn {
                                Style::default().fg(theme_focus)
                            } else {
                                Style::default()
                            }),
                    );
                    let cancel_btn = Paragraph::new("[ 取消 Ctrl+F3 ]").block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(if table_edit_focus == TableEditFocus::CancelBtn {
                                Style::default().fg(theme_focus)
                            } else {
                                Style::default()
                            }),
                    );
                    f.render_widget(save_btn, btns[0]);
                    f.render_widget(insert_btn, btns[1]);
                    f.render_widget(delete_btn, btns[2]);
                    f.render_widget(cancel_btn, btns[3]);

                    let col_lines = table_columns_info
                        .iter()
                        .map(|(n, t, d)| Line::from(format!("{n} | {t} | {d}")))
                        .collect::<Vec<_>>();
                    let col_widget = Paragraph::new(col_lines)
                        .block(Block::default().title("columns").borders(Borders::ALL));
                    f.render_widget(col_widget, outer[1]);

                    if row_form_popup {
                        let form_area = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Percentage(12),
                                Constraint::Percentage(76),
                                Constraint::Percentage(12),
                            ])
                            .split(area)[1];
                        let form_area = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([
                                Constraint::Percentage(12),
                                Constraint::Percentage(76),
                                Constraint::Percentage(12),
                            ])
                            .split(form_area)[1];
                        f.render_widget(Clear, form_area);
                        let block = Block::default()
                            .title(if row_form_insert_mode { "新增資料列" } else { "編輯資料列" })
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme_focus));
                        f.render_widget(block, form_area);

                        let inner = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Min(5), Constraint::Length(3)])
                            .margin(1)
                            .split(form_area);

                        let mut lines = Vec::<Line>::new();
                        for (idx, col) in table_preview_cols.iter().enumerate() {
                            let mark = if idx == row_form_idx { ">" } else { " " };
                            let val = row_form_values.get(idx).cloned().unwrap_or_default();
                            lines.push(Line::from(format!("{mark} {col:<24} = {val}")));
                        }
                        lines.push(Line::from("Tab:欄位/按鈕切換  Enter:儲存  Esc:取消"));
                        let form_widget = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));
                        f.render_widget(form_widget, inner[0]);

                        let btns = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([Constraint::Length(20), Constraint::Length(20), Constraint::Min(1)])
                            .split(inner[1]);
                        dlg_save_btn = Some(btns[0]);
                        dlg_cancel_btn = Some(btns[1]);
                        let save_style = if table_edit_focus == TableEditFocus::RowFormSaveBtn {
                            Style::default().fg(theme_focus)
                        } else {
                            Style::default()
                        };
                        let cancel_style = if table_edit_focus == TableEditFocus::RowFormCancelBtn {
                            Style::default().fg(theme_focus)
                        } else {
                            Style::default()
                        };
                        f.render_widget(
                            Paragraph::new("[ 儲存 ]").block(Block::default().borders(Borders::ALL).border_style(save_style)),
                            btns[0],
                        );
                        f.render_widget(
                            Paragraph::new("[ 取消 ]").block(Block::default().borders(Borders::ALL).border_style(cancel_style)),
                            btns[1],
                        );
                    }
                }

                if object_def_popup {
                    let area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(20),
                            Constraint::Length(14),
                            Constraint::Percentage(20),
                        ])
                        .split(f.size())[1];
                    f.render_widget(Clear, area);
                    let parts = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Min(8), Constraint::Length(3)])
                        .split(area);
                    let sql_widget = Paragraph::new(vec![
                        Line::from(format!("object: {}", object_def_name)),
                        Line::from("----------------------------------------"),
                        Line::from(object_def_sql.as_str()),
                    ])
                    .block(Block::default().title("Definition SQL").borders(Borders::ALL));
                    f.render_widget(sql_widget, parts[0]);
                    let btns = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Length(20), Constraint::Length(20), Constraint::Min(1)])
                        .split(parts[1]);
                    dlg_save_btn = Some(btns[0]);
                    dlg_cancel_btn = Some(btns[1]);
                    f.render_widget(
                        Paragraph::new("[ 儲存 Ctrl+F2 ]").block(Block::default().borders(Borders::ALL)),
                        btns[0],
                    );
                    f.render_widget(
                        Paragraph::new("[ 取消 Ctrl+F3 ]").block(Block::default().borders(Borders::ALL)),
                        btns[1],
                    );
                }
            })?;

            if !event::poll(Duration::from_millis(200))? {
                continue;
            }
            match event::read()? {
                Event::Mouse(m) => {
                    if m.kind == MouseEventKind::Down(MouseButton::Left) {
                        let x = m.column;
                        let y = m.row;
                        let in_rect = |r: Rect| -> bool {
                            x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
                        };
                        if let Some(r) = dlg_save_btn {
                            if in_rect(r) {
                                if table_edit_popup
                                    && !table_preview_rows.is_empty()
                                    && !table_preview_cols.is_empty()
                                    && table_edit_col_idx < table_preview_cols.len()
                                {
                                    let table_name = table_stats
                                        .get(db_selected_idx)
                                        .map(|t| t.name.clone())
                                        .unwrap_or_default();
                                    let key_col = table_preview_cols[0].clone();
                                    let key_val = table_preview_rows
                                        .get(table_edit_row_idx)
                                        .and_then(|r| r.first())
                                        .cloned()
                                        .unwrap_or_default();
                                    let target_col = table_preview_cols[table_edit_col_idx].clone();
                                    match update_table_cell(
                                        &cli,
                                        &table_name,
                                        &key_col,
                                        &key_val,
                                        &target_col,
                                        table_edit_cell.trim(),
                                    ) {
                                        Ok(sql) => {
                                            table_edit_sql = sql;
                                            logs.push("table cell updated".to_string());
                                        }
                                        Err(e) => logs.push(format!("table update failed: {e}")),
                                    }
                                }
                            }
                        }
                        if let Some(r) = dlg_cancel_btn {
                            if in_rect(r) {
                                table_edit_popup = false;
                                object_def_popup = false;
                            }
                        }
                    }
                    continue;
                }
                Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Tab if table_edit_popup => {
                        if row_form_popup {
                            table_edit_focus = match table_edit_focus {
                                TableEditFocus::RowForm => TableEditFocus::RowFormSaveBtn,
                                TableEditFocus::RowFormSaveBtn => TableEditFocus::RowFormCancelBtn,
                                TableEditFocus::RowFormCancelBtn => TableEditFocus::RowForm,
                                _ => TableEditFocus::RowForm,
                            };
                            continue;
                        }
                        table_edit_focus = match table_edit_focus {
                            TableEditFocus::SqlEditor => TableEditFocus::Grid,
                            TableEditFocus::Grid => TableEditFocus::CellInput,
                            TableEditFocus::CellInput => TableEditFocus::SaveBtn,
                            TableEditFocus::SaveBtn => TableEditFocus::InsertBtn,
                            TableEditFocus::InsertBtn => TableEditFocus::DeleteBtn,
                            TableEditFocus::DeleteBtn => TableEditFocus::CancelBtn,
                            TableEditFocus::CancelBtn => TableEditFocus::SqlEditor,
                        };
                    }
                    KeyCode::Tab => {
                        if !sql_popup && !table_edit_popup && !help_popup {
                            if editing_form {
                                editing_form = false;
                                form_input.clear();
                            }
                            focus_area = match (current_page, focus_area) {
                                (ScreenPage::Main, FocusArea::Form) => FocusArea::Input,
                                (ScreenPage::Main, FocusArea::Input) => FocusArea::CmdPane,
                                (ScreenPage::Main, FocusArea::CmdPane) => FocusArea::Form,
                                (ScreenPage::Main, FocusArea::DbPane) => FocusArea::Form,
                                (_, FocusArea::Form) => FocusArea::Input,
                                (_, FocusArea::Input) => FocusArea::DbPane,
                                (_, FocusArea::DbPane) => FocusArea::CmdPane,
                                (_, FocusArea::CmdPane) => FocusArea::Form,
                            };
                            logs.push(match focus_area {
                                FocusArea::Form => tr(cli.lang, "focus_form"),
                                FocusArea::Input => tr(cli.lang, "focus_input"),
                                FocusArea::DbPane => tr(cli.lang, "focus_db"),
                                FocusArea::CmdPane => "focus: command pane".to_string(),
                            });
                        }
                    }
                    KeyCode::BackTab if table_edit_popup => {
                        if row_form_popup {
                            table_edit_focus = match table_edit_focus {
                                TableEditFocus::RowForm => TableEditFocus::RowFormCancelBtn,
                                TableEditFocus::RowFormSaveBtn => TableEditFocus::RowForm,
                                TableEditFocus::RowFormCancelBtn => TableEditFocus::RowFormSaveBtn,
                                _ => TableEditFocus::RowForm,
                            };
                            continue;
                        }
                        table_edit_focus = match table_edit_focus {
                            TableEditFocus::SqlEditor => TableEditFocus::CancelBtn,
                            TableEditFocus::Grid => TableEditFocus::SqlEditor,
                            TableEditFocus::CellInput => TableEditFocus::Grid,
                            TableEditFocus::SaveBtn => TableEditFocus::CellInput,
                            TableEditFocus::InsertBtn => TableEditFocus::SaveBtn,
                            TableEditFocus::DeleteBtn => TableEditFocus::InsertBtn,
                            TableEditFocus::CancelBtn => TableEditFocus::DeleteBtn,
                        };
                    }
                    KeyCode::BackTab => {
                        if !sql_popup && !table_edit_popup && !help_popup && current_page == ScreenPage::DbOps {
                            db_tab = match db_tab {
                                DbTab::Table => DbTab::StoredProcedure,
                                DbTab::View => DbTab::Table,
                                DbTab::StoredProcedure => DbTab::View,
                            };
                            db_selected_idx = 0;
                        }
                    }
                    KeyCode::F(10) => {
                        if !sql_popup
                            && !table_edit_popup
                            && !help_popup
                            && current_page == ScreenPage::DbOps
                        {
                            db_conn_source = match db_conn_source {
                                DbConnSource::Export => DbConnSource::Import,
                                DbConnSource::Import => DbConnSource::Export,
                            };
                            db_selected_idx = 0;
                            refresh_db_panel(
                                &cli,
                                db_conn_source,
                                &mut table_stats,
                                &mut views,
                                &mut sps,
                                &mut logs,
                            );
                        }
                    }
                    KeyCode::F(8) => {
                        if !sql_popup && !table_edit_popup {
                            menu_selected_idx = (menu_selected_idx + 1) % menu_items.len();
                        }
                    }
                    KeyCode::F(9) => {
                        if !sql_popup && !table_edit_popup && !help_popup {
                            if let Some((_, cmd)) = menu_items.get(menu_selected_idx) {
                                match *cmd {
                                    "__menu_export_data__" => {
                                        cli.flow = FlowMode::Export;
                                        input = "/run-export-data".to_string();
                                        focus_area = FocusArea::Input;
                                        current_page = ScreenPage::Main;
                                        logs.push("menu: 匯出 (flow=export-data)".to_string());
                                        logs.push("請按下 Enter 確認執行".to_string());
                                    }
                                    "__menu_import_data__" => {
                                        cli.flow = FlowMode::Import;
                                        input = "/run-import-data".to_string();
                                        focus_area = FocusArea::Input;
                                        current_page = ScreenPage::Main;
                                        logs.push("menu: 匯入 (flow=import-data)".to_string());
                                        logs.push("請按下 Enter 確認執行".to_string());
                                    }
                                    "__menu_export_schema__" => {
                                        cli.flow = FlowMode::ExportSchema;
                                        input = "/run-export-schema".to_string();
                                        focus_area = FocusArea::Input;
                                        current_page = ScreenPage::Main;
                                        logs.push("menu: 匯出結構 (flow=export-schema)".to_string());
                                        logs.push("請按下 Enter 確認執行".to_string());
                                    }
                                    "__menu_import_schema__" => {
                                        cli.flow = FlowMode::ImportSchema;
                                        input = "/run-import-schema".to_string();
                                        focus_area = FocusArea::Input;
                                        current_page = ScreenPage::Main;
                                        logs.push("menu: 匯入結構 (flow=import-schema)".to_string());
                                        logs.push("請按下 Enter 確認執行".to_string());
                                    }
                                    "__menu_db_ops__" => {
                                        current_page = ScreenPage::DbOps;
                                        focus_area = FocusArea::DbPane;
                                        logs.push("menu: 進入 DB 操作頁".to_string());
                                    }
                                    "__menu_help__" => {
                                        help_popup = true;
                                        logs.push("menu: open help dialog".to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    KeyCode::F(12) => {
                        if !sql_popup && !table_edit_popup {
                            help_popup = true;
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        if table_edit_popup {
                            table_edit_popup = false;
                            table_edit_cell.clear();
                            table_edit_focus = TableEditFocus::CellInput;
                            continue;
                        }
                    }
                    KeyCode::F(2) if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        if table_edit_popup
                            && !table_preview_rows.is_empty()
                            && !table_preview_cols.is_empty()
                            && table_edit_col_idx < table_preview_cols.len()
                        {
                            let table_name = table_stats
                                .get(db_selected_idx)
                                .map(|t| t.name.clone())
                                .unwrap_or_default();
                            let key_col = table_preview_cols[0].clone();
                            let key_val = table_preview_rows
                                .get(table_edit_row_idx)
                                .and_then(|r| r.first())
                                .cloned()
                                .unwrap_or_default();
                            let target_col = table_preview_cols[table_edit_col_idx].clone();
                            match update_table_cell(
                                &cli,
                                &table_name,
                                &key_col,
                                &key_val,
                                &target_col,
                                table_edit_cell.trim(),
                            ) {
                                Ok(sql) => {
                                    table_edit_sql = sql;
                                    logs.push("table cell updated".to_string());
                                }
                                Err(e) => logs.push(format!("table update failed: {e}")),
                            }
                        }
                    }
                    KeyCode::F(4) if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        if table_edit_popup && !table_edit_table_name.is_empty() && !table_preview_cols.is_empty() {
                            let vals = table_preview_cols
                                .iter()
                                .enumerate()
                                .map(|(idx, _)| {
                                    if idx == 0 {
                                        String::new()
                                    } else {
                                        String::new()
                                    }
                                })
                                .collect::<Vec<_>>();
                            match insert_table_row(&cli, &table_edit_table_name, &table_preview_cols, &vals) {
                                Ok(sql) => {
                                    table_edit_sql = sql;
                                    logs.push("table row inserted".to_string());
                                    if let Ok((cols, rows, qsql)) = fetch_table_preview_page(
                                        &cli,
                                        &table_edit_table_name,
                                        table_page_no,
                                        table_page_size,
                                    ) {
                                        table_preview_cols = cols;
                                        table_preview_rows = rows;
                                        table_edit_sql = qsql;
                                    }
                                }
                                Err(e) => logs.push(format!("table insert failed: {e}")),
                            }
                        }
                    }
                    KeyCode::F(5) if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        if table_edit_popup
                            && !table_edit_table_name.is_empty()
                            && !table_preview_cols.is_empty()
                            && !table_preview_rows.is_empty()
                        {
                            let key_col = table_preview_cols[0].clone();
                            let key_val = table_preview_rows
                                .get(table_edit_row_idx)
                                .and_then(|r| r.first())
                                .cloned()
                                .unwrap_or_default();
                            match delete_table_row(&cli, &table_edit_table_name, &key_col, &key_val) {
                                Ok(sql) => {
                                    table_edit_sql = sql;
                                    logs.push("table row deleted".to_string());
                                    if let Ok((cols, rows, qsql)) = fetch_table_preview_page(
                                        &cli,
                                        &table_edit_table_name,
                                        table_page_no,
                                        table_page_size,
                                    ) {
                                        table_preview_cols = cols;
                                        table_preview_rows = rows;
                                        table_edit_sql = qsql;
                                        table_edit_row_idx = table_edit_row_idx.min(table_preview_rows.len().saturating_sub(1));
                                    }
                                }
                                Err(e) => logs.push(format!("table delete failed: {e}")),
                            }
                        }
                    }
                    KeyCode::F(3) if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        if table_edit_popup || object_def_popup {
                            table_edit_popup = false;
                            object_def_popup = false;
                            continue;
                        }
                    }
                    KeyCode::Esc => {
                        if help_popup {
                            help_popup = false;
                            continue;
                        }
                        if sql_popup {
                            sql_popup = false;
                            continue;
                        }
                        if table_edit_popup {
                            if row_form_popup {
                                row_form_popup = false;
                                table_edit_focus = TableEditFocus::Grid;
                                continue;
                            }
                            table_edit_popup = false;
                            table_edit_cell.clear();
                            table_edit_focus = TableEditFocus::CellInput;
                            continue;
                        }
                        break;
                    }
                    KeyCode::Backspace => {
                        if sql_popup {
                            sql_input.pop();
                        } else if table_edit_popup && table_edit_focus == TableEditFocus::SqlEditor {
                            table_edit_sql.pop();
                        } else if table_edit_popup && table_edit_focus == TableEditFocus::CellInput {
                            table_edit_cell.pop();
                        } else if table_edit_popup && row_form_popup && table_edit_focus == TableEditFocus::RowForm {
                            if let Some(v) = row_form_values.get_mut(row_form_idx) {
                                v.pop();
                            }
                        } else if editing_form {
                            form_input.pop();
                        } else if focus_area == FocusArea::DbPane && current_page == ScreenPage::DbOps {
                            db_search_query.pop();
                            if db_search_query.is_empty() {
                                db_search_last_input = None;
                            } else {
                                db_search_last_input = Some(Instant::now());
                            }
                        } else {
                            input.pop();
                            cmd_history_idx = None;
                        }
                    }
                    KeyCode::Up if !editing_form && focus_area == FocusArea::Form => {
                        selected_idx = selected_idx.saturating_sub(1);
                    }
                    KeyCode::Up
                        if table_edit_popup
                            && row_form_popup
                            && table_edit_focus == TableEditFocus::RowForm =>
                    {
                        row_form_idx = row_form_idx.saturating_sub(1).max(1);
                    }
                    KeyCode::Down
                        if table_edit_popup
                            && row_form_popup
                            && table_edit_focus == TableEditFocus::RowForm =>
                    {
                        if !table_preview_cols.is_empty() {
                            row_form_idx = (row_form_idx + 1).min(table_preview_cols.len().saturating_sub(1));
                        }
                    }
                    KeyCode::Up
                        if table_edit_popup
                            && table_edit_focus == TableEditFocus::Grid
                            && !table_preview_rows.is_empty() =>
                    {
                        table_edit_row_idx = table_edit_row_idx.saturating_sub(1);
                        table_edit_cell = table_preview_rows
                            .get(table_edit_row_idx)
                            .and_then(|r| r.get(table_edit_col_idx))
                            .cloned()
                            .unwrap_or_default();
                    }
                    KeyCode::Down
                        if table_edit_popup
                            && table_edit_focus == TableEditFocus::Grid
                            && !table_preview_rows.is_empty() =>
                    {
                        table_edit_row_idx = (table_edit_row_idx + 1).min(table_preview_rows.len().saturating_sub(1));
                        table_edit_cell = table_preview_rows
                            .get(table_edit_row_idx)
                            .and_then(|r| r.get(table_edit_col_idx))
                            .cloned()
                            .unwrap_or_default();
                    }
                    KeyCode::Left
                        if table_edit_popup
                            && table_edit_focus == TableEditFocus::Grid
                            && table_preview_cols.len() > 1 =>
                    {
                        table_edit_col_idx = table_edit_col_idx.saturating_sub(1).max(1);
                        table_edit_cell = table_preview_rows
                            .get(table_edit_row_idx)
                            .and_then(|r| r.get(table_edit_col_idx))
                            .cloned()
                            .unwrap_or_default();
                    }
                    KeyCode::Right
                        if table_edit_popup
                            && table_edit_focus == TableEditFocus::Grid
                            && table_preview_cols.len() > 1 =>
                    {
                        table_edit_col_idx = (table_edit_col_idx + 1).min(table_preview_cols.len().saturating_sub(1));
                        if table_edit_col_idx == 0 {
                            table_edit_col_idx = 1;
                        }
                        table_edit_cell = table_preview_rows
                            .get(table_edit_row_idx)
                            .and_then(|r| r.get(table_edit_col_idx))
                            .cloned()
                            .unwrap_or_default();
                    }
                    KeyCode::Up
                        if !editing_form
                            && focus_area == FocusArea::DbPane
                            && current_page == ScreenPage::DbOps =>
                    {
                        db_selected_idx = db_selected_idx.saturating_sub(1);
                    }
                    KeyCode::Down if !editing_form && focus_area == FocusArea::Form => {
                        selected_idx = (selected_idx + 1).min(FORM_FIELDS.len().saturating_sub(1));
                    }
                    KeyCode::Down
                        if !editing_form
                            && focus_area == FocusArea::DbPane
                            && current_page == ScreenPage::DbOps =>
                    {
                        let max_len = match db_tab {
                            DbTab::Table => table_stats.len(),
                            DbTab::View => views.len(),
                            DbTab::StoredProcedure => sps.len(),
                        };
                        if max_len > 0 {
                            db_selected_idx = (db_selected_idx + 1).min(max_len.saturating_sub(1));
                        }
                    }
                    KeyCode::Up if !editing_form && focus_area == FocusArea::CmdPane => {
                        cmd_selected_idx = cmd_selected_idx.saturating_sub(1);
                    }
                    KeyCode::Down if !editing_form && focus_area == FocusArea::CmdPane => {
                        if !cmd_items.is_empty() {
                            cmd_selected_idx = (cmd_selected_idx + 1).min(cmd_items.len().saturating_sub(1));
                        }
                    }
                    KeyCode::PageUp
                        if table_edit_popup
                            && current_page == ScreenPage::DbOps
                            && !table_stats.is_empty() =>
                    {
                        table_page_no = table_page_no.saturating_sub(1).max(1);
                        if let Some(ts) = table_stats.get(db_selected_idx) {
                            match fetch_table_preview_page(&cli, &ts.name, table_page_no, table_page_size) {
                                Ok((cols, rows, sql)) => {
                                    table_preview_cols = cols;
                                    table_preview_rows = rows;
                                    table_edit_sql = sql;
                                    table_edit_row_idx = 0;
                                    table_edit_col_idx = if table_preview_cols.len() > 1 { 1 } else { 0 };
                                    table_edit_cell = table_preview_rows
                                        .get(table_edit_row_idx)
                                        .and_then(|r| r.get(table_edit_col_idx))
                                        .cloned()
                                        .unwrap_or_default();
                                }
                                Err(e) => logs.push(format!("table preview paging failed: {e}")),
                            }
                        }
                    }
                    KeyCode::PageDown
                        if table_edit_popup
                            && current_page == ScreenPage::DbOps
                            && !table_stats.is_empty() =>
                    {
                        table_page_no += 1;
                        if let Some(ts) = table_stats.get(db_selected_idx) {
                            match fetch_table_preview_page(&cli, &ts.name, table_page_no, table_page_size) {
                                Ok((cols, rows, sql)) => {
                                    if rows.is_empty() && table_page_no > 1 {
                                        table_page_no -= 1;
                                    } else {
                                        table_preview_cols = cols;
                                        table_preview_rows = rows;
                                        table_edit_sql = sql;
                                        table_edit_row_idx = 0;
                                        table_edit_col_idx = if table_preview_cols.len() > 1 { 1 } else { 0 };
                                        table_edit_cell = table_preview_rows
                                            .get(table_edit_row_idx)
                                            .and_then(|r| r.get(table_edit_col_idx))
                                            .cloned()
                                            .unwrap_or_default();
                                    }
                                }
                                Err(e) => logs.push(format!("table preview paging failed: {e}")),
                            }
                        }
                    }
                    KeyCode::PageUp
                        if !editing_form
                            && focus_area == FocusArea::DbPane
                            && current_page == ScreenPage::DbOps =>
                    {
                        let page_step = 10usize;
                        db_selected_idx = db_selected_idx.saturating_sub(page_step);
                    }
                    KeyCode::PageDown
                        if !editing_form
                            && focus_area == FocusArea::DbPane
                            && current_page == ScreenPage::DbOps =>
                    {
                        let page_step = 10usize;
                        let max_len = match db_tab {
                            DbTab::Table => table_stats.len(),
                            DbTab::View => views.len(),
                            DbTab::StoredProcedure => sps.len(),
                        };
                        if max_len > 0 {
                            db_selected_idx = (db_selected_idx + page_step).min(max_len.saturating_sub(1));
                        }
                    }
                    KeyCode::PageUp if !editing_form && focus_area == FocusArea::CmdPane => {
                        let page_step = 10usize;
                        cmd_selected_idx = cmd_selected_idx.saturating_sub(page_step);
                    }
                    KeyCode::PageDown if !editing_form && focus_area == FocusArea::CmdPane => {
                        let page_step = 10usize;
                        if !cmd_items.is_empty() {
                            cmd_selected_idx =
                                (cmd_selected_idx + page_step).min(cmd_items.len().saturating_sub(1));
                        }
                    }
                    KeyCode::Left if !editing_form && focus_area == FocusArea::Form => {
                        let _ = cycle_field_option(&mut cli, FORM_FIELDS[selected_idx], -1);
                    }
                    KeyCode::Right if !editing_form && focus_area == FocusArea::Form => {
                        let _ = cycle_field_option(&mut cli, FORM_FIELDS[selected_idx], 1);
                    }
                    KeyCode::Up if focus_area == FocusArea::Input && !sql_popup && !editing_form => {
                        if !cmd_history.is_empty() {
                            let next_idx = match cmd_history_idx {
                                Some(i) if i > 0 => i - 1,
                                Some(i) => i,
                                None => cmd_history.len().saturating_sub(1),
                            };
                            cmd_history_idx = Some(next_idx);
                            input = cmd_history[next_idx].clone();
                        }
                    }
                    KeyCode::Down if focus_area == FocusArea::Input && !sql_popup && !editing_form => {
                        if let Some(i) = cmd_history_idx {
                            if i + 1 < cmd_history.len() {
                                let next_idx = i + 1;
                                cmd_history_idx = Some(next_idx);
                                input = cmd_history[next_idx].clone();
                            } else {
                                cmd_history_idx = None;
                                input.clear();
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if table_edit_popup {
                            if row_form_popup {
                                if table_edit_focus == TableEditFocus::RowFormCancelBtn {
                                    row_form_popup = false;
                                    table_edit_focus = TableEditFocus::Grid;
                                    continue;
                                }
                                if table_edit_focus == TableEditFocus::RowFormSaveBtn || table_edit_focus == TableEditFocus::RowForm {
                                    if row_form_insert_mode {
                                        let mut vals = row_form_values.clone();
                                        for (idx, col) in table_preview_cols.iter().enumerate() {
                                            if idx == 0 && row_form_pk_auto_inc.contains(col) {
                                                vals[idx].clear();
                                            } else if idx == 0 {
                                                let ty = table_columns_info.get(idx).map(|v| v.1.to_ascii_lowercase()).unwrap_or_default();
                                                if ty.contains("char") || ty.contains("text") || ty.contains("varchar") || ty.contains("nchar") || ty.contains("nvarchar") {
                                                    if vals[idx].trim().is_empty() {
                                                        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|v| v.as_nanos()).unwrap_or(0);
                                                        vals[idx] = format!("{:032x}", ts);
                                                    }
                                                }
                                            }
                                        }
                                        match insert_table_row(&cli, &table_edit_table_name, &table_preview_cols, &vals) {
                                            Ok(sql) => {
                                                table_edit_sql = sql;
                                                logs.push("table row inserted".to_string());
                                            }
                                            Err(e) => logs.push(format!("table insert failed: {e}")),
                                        }
                                    } else {
                                        if !row_form_original.is_empty() && !table_preview_cols.is_empty() {
                                            let table_name = table_edit_table_name.clone();
                                            let key_col = table_preview_cols[0].clone();
                                            let key_val = row_form_original.first().cloned().unwrap_or_default();
                                            for idx in 1..table_preview_cols.len() {
                                                let new_val = row_form_values.get(idx).cloned().unwrap_or_default();
                                                let old_val = row_form_original.get(idx).cloned().unwrap_or_default();
                                                if new_val != old_val {
                                                    let target_col = table_preview_cols[idx].clone();
                                                    if let Err(e) = update_table_cell(&cli, &table_name, &key_col, &key_val, &target_col, &new_val) {
                                                        logs.push(format!("table update failed: {e}"));
                                                        break;
                                                    }
                                                }
                                            }
                                            logs.push("table row saved".to_string());
                                        }
                                    }
                                    row_form_popup = false;
                                    table_edit_focus = TableEditFocus::Grid;
                                    continue;
                                }
                            }
                            if table_edit_focus == TableEditFocus::SqlEditor {
                                match fetch_rows_by_sql(&cli, table_edit_sql.trim(), table_page_size) {
                                    Ok((cols, rows)) => {
                                        table_preview_cols = cols;
                                        table_preview_rows = rows;
                                        table_edit_row_idx = 0;
                                        table_edit_col_idx = if table_preview_cols.len() > 1 { 1 } else { 0 };
                                        table_edit_cell = table_preview_rows
                                            .get(table_edit_row_idx)
                                            .and_then(|r| r.get(table_edit_col_idx))
                                            .cloned()
                                            .unwrap_or_default();
                                        logs.push("table edit sql applied".to_string());
                                    }
                                    Err(e) => logs.push(format!("apply table edit sql failed: {e}")),
                                }
                                continue;
                            }
                            if table_edit_focus == TableEditFocus::CancelBtn {
                                table_edit_popup = false;
                                table_edit_focus = TableEditFocus::SqlEditor;
                                continue;
                            }
                            if table_edit_focus == TableEditFocus::InsertBtn {
                                if !table_edit_table_name.is_empty() && !table_preview_cols.is_empty() {
                                    row_form_values = table_preview_cols.iter().map(|_| String::new()).collect::<Vec<_>>();
                                    row_form_original = row_form_values.clone();
                                    row_form_idx = 1.min(table_preview_cols.len().saturating_sub(1));
                                    row_form_insert_mode = true;
                                    row_form_pk_auto_inc = auto_increment_pk_column_set(&mut DbConnection::connect(&cli).unwrap_or_else(|_| panic!("connect failed")), cli.db_type, &table_edit_table_name).unwrap_or_default();
                                    row_form_popup = true;
                                    table_edit_focus = TableEditFocus::RowForm;
                                }
                                continue;
                            }
                            if table_edit_focus == TableEditFocus::DeleteBtn {
                                if !table_edit_table_name.is_empty()
                                    && !table_preview_cols.is_empty()
                                    && !table_preview_rows.is_empty()
                                {
                                    let key_col = table_preview_cols[0].clone();
                                    let key_val = table_preview_rows
                                        .get(table_edit_row_idx)
                                        .and_then(|r| r.first())
                                        .cloned()
                                        .unwrap_or_default();
                                    match delete_table_row(&cli, &table_edit_table_name, &key_col, &key_val) {
                                        Ok(sql) => {
                                            table_edit_sql = sql;
                                            logs.push("table row deleted".to_string());
                                        }
                                        Err(e) => logs.push(format!("table delete failed: {e}")),
                                    }
                                }
                                continue;
                            }
                            if table_edit_focus == TableEditFocus::SaveBtn {
                                if !table_preview_rows.is_empty()
                                    && !table_preview_cols.is_empty()
                                    && table_edit_col_idx < table_preview_cols.len()
                                {
                                    let table_name = table_stats
                                        .get(db_selected_idx)
                                        .map(|t| t.name.clone())
                                        .unwrap_or_default();
                                    let key_col = table_preview_cols[0].clone();
                                    let key_val = table_preview_rows
                                        .get(table_edit_row_idx)
                                        .and_then(|r| r.first())
                                        .cloned()
                                        .unwrap_or_default();
                                    let target_col = table_preview_cols[table_edit_col_idx].clone();
                                    match update_table_cell(
                                        &cli,
                                        &table_name,
                                        &key_col,
                                        &key_val,
                                        &target_col,
                                        table_edit_cell.trim(),
                                    ) {
                                        Ok(sql) => {
                                            table_edit_sql = sql;
                                            if let Some(row) = table_preview_rows.get_mut(table_edit_row_idx) {
                                                if table_edit_col_idx < row.len() {
                                                    row[table_edit_col_idx] = table_edit_cell.trim().to_string();
                                                }
                                            }
                                            logs.push("table cell updated".to_string());
                                        }
                                        Err(e) => logs.push(format!("table update failed: {e}")),
                                    }
                                }
                                continue;
                            }
                            if key.modifiers.contains(event::KeyModifiers::SHIFT)
                                && !table_preview_rows.is_empty()
                                && !table_preview_cols.is_empty()
                                && table_edit_col_idx < table_preview_cols.len()
                            {
                                let table_name = table_stats
                                    .get(db_selected_idx)
                                    .map(|t| t.name.clone())
                                    .unwrap_or_default();
                                let key_col = table_preview_cols[0].clone();
                                let key_val = table_preview_rows
                                    .get(table_edit_row_idx)
                                    .and_then(|r| r.first())
                                    .cloned()
                                    .unwrap_or_default();
                                let target_col = table_preview_cols[table_edit_col_idx].clone();
                                match update_table_cell(
                                    &cli,
                                    &table_name,
                                    &key_col,
                                    &key_val,
                                    &target_col,
                                    table_edit_cell.trim(),
                                ) {
                                    Ok(sql) => {
                                        table_edit_sql = sql;
                                        if let Some(row) = table_preview_rows.get_mut(table_edit_row_idx) {
                                            if table_edit_col_idx < row.len() {
                                                row[table_edit_col_idx] = table_edit_cell.trim().to_string();
                                            }
                                        }
                                        logs.push("table cell updated".to_string());
                                    }
                                    Err(e) => logs.push(format!("table update failed: {e}")),
                                }
                            }
                            continue;
                        }

                        if table_edit_popup && table_edit_focus == TableEditFocus::Grid {
                            if let Some(sel_row) = table_preview_rows.get(table_edit_row_idx).cloned() {
                                row_form_values = sel_row.clone();
                                row_form_original = sel_row;
                                row_form_idx = 1.min(table_preview_cols.len().saturating_sub(1));
                                row_form_insert_mode = false;
                                row_form_pk_auto_inc.clear();
                                row_form_popup = true;
                                table_edit_focus = TableEditFocus::RowForm;
                            }
                            continue;
                        }

                        if sql_popup {
                            logs.push("/exec-sql =>".to_string());
                            match tui_exec_sql(&cli, sql_input.trim(), 50) {
                                Ok(rows) => logs.extend(rows),
                                Err(e) => logs.push(format!("/exec-sql failed: {e}")),
                            }
                            sql_popup = false;
                            continue;
                        }

                        if editing_form {
                            let field = FORM_FIELDS[selected_idx];
                            match set_field_value(&mut cli, field, form_input.trim()) {
                                Ok(_) => logs.push(format!("{} {field}={}", tr(cli.lang, "ok_set"), form_input.trim())),
                                Err(e) => logs.push(format!("{}: {e}", tr(cli.lang, "err"))),
                            }
                            editing_form = false;
                            form_input.clear();
                            continue;
                        }

                        if focus_area == FocusArea::Form {
                            let sel_field = FORM_FIELDS[selected_idx];
                            if field_options(sel_field).is_none() {
                                // 部分終端機不會回傳 Shift+Enter 修飾鍵，
                                // 因此在表單欄位（非下拉）允許 Enter 直接進入編輯。
                                editing_form = true;
                                form_input = get_field_value(&cli, sel_field);
                                continue;
                            } else if input.trim().is_empty() {
                                let _ = cycle_field_option(&mut cli, sel_field, 1);
                                continue;
                            }
                        }

                        if focus_area == FocusArea::DbPane && current_page == ScreenPage::DbOps {
                            match db_tab {
                                DbTab::Table => {
                                    if let Some(ts) = table_stats.get(db_selected_idx) {
                                        match fetch_table_preview_page(&cli, &ts.name, table_page_no, table_page_size) {
                                            Ok((cols, rows, sql)) => {
                                                table_preview_cols = cols;
                                                table_preview_rows = rows;
                                                table_edit_sql = sql;
                                                table_columns_info = fetch_table_columns_info(&cli, &ts.name)
                                                    .unwrap_or_default();
                                                table_edit_row_idx = 0;
                                                table_edit_col_idx = if table_preview_cols.len() > 1 { 1 } else { 0 };
                                                table_edit_cell = table_preview_rows
                                                    .get(table_edit_row_idx)
                                                    .and_then(|r| r.get(table_edit_col_idx))
                                                    .cloned()
                                                .unwrap_or_default();
                                                table_edit_popup = true;
                                                table_edit_table_name = ts.name.clone();
                                                table_page_no = 1;
                                                table_edit_focus = TableEditFocus::SqlEditor;
                                            }
                                            Err(e) => logs.push(format!("table preview failed: {e}")),
                                        }
                                    }
                                }
                                DbTab::View => {
                                    if let Some(name) = views.get(db_selected_idx) {
                                        object_def_name = name.clone();
                                        object_def_is_sp = false;
                                        object_def_sql = fetch_db_object_definition(&cli, name, false)
                                            .unwrap_or_else(|e| format!("load definition failed: {e}"));
                                        object_def_popup = true;
                                    }
                                }
                                DbTab::StoredProcedure => {
                                    if let Some(name) = sps.get(db_selected_idx) {
                                        object_def_name = name.clone();
                                        object_def_is_sp = true;
                                        object_def_sql = fetch_db_object_definition(&cli, name, true)
                                            .unwrap_or_else(|e| format!("load definition failed: {e}"));
                                        object_def_popup = true;
                                    }
                                }
                            }
                            continue;
                        }

                        if focus_area == FocusArea::CmdPane {
                            if let Some((cmd, _)) = cmd_items.get(cmd_selected_idx) {
                                input = (*cmd).to_string();
                                focus_area = FocusArea::Input;
                                logs.push(format!("command selected: {cmd} (press Enter)"));
                            }
                            continue;
                        }

                        let cmd = input.trim().to_string();
                        input.clear();
                        cmd_history_idx = None;
                        if cmd.is_empty() {
                            continue;
                        }
                        if cmd_history.last().map(|v| v != &cmd).unwrap_or(true) {
                            cmd_history.push(cmd.clone());
                        }

                        if cmd == "/quit" {
                            break;
                        } else if cmd == "/help" {
                            help_popup = true;
                            logs.push("open help dialog".to_string());
                        } else if cmd == "/show-hot-key" {
                            logs.push("[HOT-KEY]".to_string());
                            logs.push("F8               : top menu next item".to_string());
                            logs.push("F9               : apply menu (export/import/db/help)".to_string());
                            logs.push("F12              : open help dialog".to_string());
                            logs.push("Tab              : switch focus (main: form/input/cmd, db頁: form/input/db/cmd)".to_string());
                            logs.push("Shift-Tab        : switch DB tab (table/view/stored procedure, DB 操作頁)".to_string());
                            logs.push("F10              : switch DB connection source (export/import)".to_string());
                            logs.push("Up/Down          : form select row OR input history OR db list select".to_string());
                            logs.push("PageUp/PageDown  : db list / command list paging".to_string());
                            logs.push("command pane     : TAB focus then ↑↓/PgUp/PgDn/Enter".to_string());
                            logs.push("Left/Right       : switch dropdown options in form".to_string());
                            logs.push("Shift-Enter      : edit/save form field / table cell save in table edit".to_string());
                            logs.push("Ctrl+Enter       : open table-edit popup (on DB table tab)".to_string());
                            logs.push("Ctrl+T           : cancel export/import dialog".to_string());
                            logs.push("Ctrl+Q           : close table-edit dialog".to_string());
                            logs.push("Esc              : close popup / quit tui".to_string());
                            logs.push("Enter            : run command in input area".to_string());
                            logs.push("/persist-to-sqlite: create cort-dbio-kit.sqlite and persist config".to_string());
                        } else if cmd == "/test-connect" {
                            match tui_test_connect(&cli) {
                                Ok(_) => logs.push("/test-connect: OK".to_string()),
                                Err(e) => logs.push(format!("/test-connect: FAIL: {e}")),
                            }
                        } else if cmd == "/list-table" {
                            match tui_list_tables(&cli) {
                                Ok(v) => {
                                    logs.push(format!("/list-table: {}", v.len()));
                                    logs.extend(v.into_iter().take(200));
                                }
                                Err(e) => logs.push(format!("/list-table failed: {e}")),
                            }
                        } else if cmd == "/list-view" {
                            match tui_list_views(&cli) {
                                Ok(v) => {
                                    logs.push(format!("/list-view: {}", v.len()));
                                    logs.extend(v.into_iter().take(200));
                                }
                                Err(e) => logs.push(format!("/list-view failed: {e}")),
                            }
                        } else if cmd == "/list-stored-procedure" {
                            match tui_list_stored_procedures(&cli) {
                                Ok(v) => {
                                    logs.push(format!("/list-stored-procedure: {}", v.len()));
                                    logs.extend(v.into_iter().take(200));
                                }
                                Err(e) => logs.push(format!("/list-stored-procedure failed: {e}")),
                            }
                        } else if cmd == "/exec-sql" {
                            sql_popup = true;
                            sql_input.clear();
                        } else if cmd == "/show" {
                            logs.extend(cli_snapshot_lines(&cli));
                        } else if cmd == "/persist-to-sqlite" {
                            match persist_to_cort_sqlite(&cli) {
                                Ok(path) => logs.push(format!("persisted to sqlite: {}", path.display())),
                                Err(e) => logs.push(format!("persist-to-sqlite failed: {e}")),
                            }
                        } else if cmd == "/refresh-db" {
                            refresh_db_panel(
                                &cli,
                                db_conn_source,
                                &mut table_stats,
                                &mut views,
                                &mut sps,
                                &mut logs,
                            );
                        } else if let Some(rest) = cmd.strip_prefix("/set ") {
                            let mut parts = rest.splitn(2, ' ');
                            let field = parts.next().unwrap_or("").trim();
                            let value = parts.next().unwrap_or("").trim();
                            if field.is_empty() || value.is_empty() {
                                logs.push(tr(cli.lang, "set_usage").to_string());
                            } else {
                                match apply_set_field(&mut cli, field, value) {
                                    Ok(_) => logs.push(format!("{} {field}={value}", tr(cli.lang, "ok_set"))),
                                    Err(e) => logs.push(format!("{}: {e}", tr(cli.lang, "err"))),
                                }
                            }
                        } else if let Some(field) = cmd.strip_prefix("/unset ") {
                            match apply_unset_field(&mut cli, field.trim()) {
                                Ok(_) => logs.push(format!("{} {}", tr(cli.lang, "ok_unset"), field.trim())),
                                Err(e) => logs.push(format!("{}: {e}", tr(cli.lang, "err"))),
                            }
                        } else if cmd == "/run-export-data" || cmd == "/run-import-data" {
                            let mut cli_run = cli.clone();
                            cli_run.flow = if cmd == "/run-export-data" {
                                FlowMode::Export
                            } else {
                                FlowMode::Import
                            };
                            cli.flow = cli_run.flow;
                            logs.push(match cli_run.flow {
                                FlowMode::Export | FlowMode::ExportSchema => tr(cli.lang, "run_start"),
                                FlowMode::Import | FlowMode::ImportSchema => tr(cli.lang, "run_start_import"),
                            });
                            let cancel_flag = Arc::new(AtomicBool::new(false));
                            let cancel_flag2 = cancel_flag.clone();
                            let progress_state = Arc::new(Mutex::new(RunProgressSnapshot::default()));
                            let progress_state2 = progress_state.clone();
                            let handle = thread::spawn(move || {
                                let r = match cli_run.flow {
                                    FlowMode::Export | FlowMode::ExportSchema => {
                                        execute_export_with_cancel(
                                            &cli_run,
                                            true,
                                            &cancel_flag2,
                                            Some(&progress_state2),
                                        )
                                    }
                                    FlowMode::Import | FlowMode::ImportSchema => {
                                        execute_import_with_cancel(
                                            &cli_run,
                                            true,
                                            &cancel_flag2,
                                            Some(&progress_state2),
                                        )
                                    }
                                };
                                r
                            });
                            let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                            let mut spin_idx = 0usize;
                            let run_started = Instant::now();

                            while !handle.is_finished() {
                                if event::poll(Duration::from_millis(10))? {
                                    if let Event::Key(k) = event::read()? {
                                        if k.kind == KeyEventKind::Press
                                            && matches!(k.code, KeyCode::Char('t'))
                                            && k.modifiers.contains(event::KeyModifiers::CONTROL)
                                        {
                                            cancel_flag.store(true, Ordering::Relaxed);
                                        }
                                    }
                                }
                                let snapshot = progress_state
                                    .lock()
                                    .map(|s| s.clone())
                                    .unwrap_or_else(|_| RunProgressSnapshot::default());
                                let msg = format!(
                                    "{} {}  {}ms",
                                    spinner[spin_idx % spinner.len()],
                                    tr(cli.lang, "run_progress"),
                                    run_started.elapsed().as_millis()
                                );
                                spin_idx += 1;
                                terminal.draw(|f| {
                                    let bg = Layout::default()
                                        .direction(Direction::Vertical)
                                        .constraints([
                                            Constraint::Percentage(30),
                                            Constraint::Percentage(35),
                                            Constraint::Percentage(35),
                                        ])
                                        .split(f.size());
                                    f.render_widget(
                                        Block::default().style(Style::default().bg(Color::Rgb(185, 225, 255))),
                                        bg[0],
                                    );
                                    f.render_widget(
                                        Block::default().style(Style::default().bg(Color::Rgb(95, 165, 235))),
                                        bg[1],
                                    );
                                    f.render_widget(
                                        Block::default().style(Style::default().bg(Color::Rgb(28, 78, 156))),
                                        bg[2],
                                    );

                                    let chunks = Layout::default()
                                        .direction(Direction::Vertical)
                                        .constraints([
                                            Constraint::Length(1),
                                            Constraint::Min(8),
                                            Constraint::Length(1),
                                        ])
                                        .split(f.size());

                                    let mut top_spans = vec![Span::styled(
                                        format!(" {} ", tr(cli.lang, "top_title")),
                                        Style::default().fg(theme_text).bg(theme_bar),
                                    )];
                                    for (idx, (label, _cmd)) in menu_items.iter().enumerate() {
                                        top_spans.push(Span::raw(" "));
                                        let is_sel = idx == menu_selected_idx;
                                        top_spans.push(Span::styled(
                                            format!(" {label} "),
                                            if is_sel {
                                                Style::default().fg(Color::Black).bg(theme_focus)
                                            } else {
                                                Style::default().fg(theme_dim).bg(theme_bar)
                                            },
                                        ));
                                    }
                                    top_spans.push(Span::raw("  "));
                                    top_spans.push(Span::styled(
                                        "F8/F9",
                                        Style::default().fg(theme_focus).bg(theme_bar),
                                    ));
                                    top_spans.push(Span::styled(
                                        " menu",
                                        Style::default().fg(theme_dim).bg(theme_bar),
                                    ));
                                    let top_title = Paragraph::new(Line::from(top_spans))
                                        .style(Style::default().bg(theme_bar));
                                    f.render_widget(top_title, chunks[0]);

                                    let status_widget = Paragraph::new(Line::from(vec![
                                        Span::styled(" Ctrl+T ", Style::default().fg(Color::Black).bg(theme_focus)),
                                        Span::styled(" cancel export/import  ", Style::default().fg(theme_dim).bg(theme_bg)),
                                        Span::styled(msg.clone(), Style::default().fg(theme_text).bg(theme_bg)),
                                    ]))
                                    .style(Style::default().bg(theme_bg));
                                    f.render_widget(status_widget, chunks[2]);

                                    let dialog_area = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints([
                                            Constraint::Percentage(12),
                                            Constraint::Percentage(76),
                                            Constraint::Percentage(12),
                                        ])
                                        .split(chunks[1])[1];
                                    let dialog_area = Layout::default()
                                        .direction(Direction::Vertical)
                                        .constraints([
                                            Constraint::Percentage(20),
                                            Constraint::Length(12),
                                            Constraint::Percentage(20),
                                        ])
                                        .split(dialog_area)[1];
                                    f.render_widget(Clear, dialog_area);
                                    let block = Block::default()
                                        .title(match cli.flow {
                                            FlowMode::Export | FlowMode::ExportSchema => "匯出中",
                                            FlowMode::Import | FlowMode::ImportSchema => "匯入中",
                                        })
                                        .borders(Borders::ALL)
                                        .border_style(Style::default().fg(theme_focus))
                                        .style(Style::default().bg(theme_panel_active));
                                    f.render_widget(block, dialog_area);

                                    let inner = Layout::default()
                                        .direction(Direction::Vertical)
                                        .constraints([
                                            Constraint::Length(2),
                                            Constraint::Length(2),
                                            Constraint::Length(1),
                                            Constraint::Length(2),
                                            Constraint::Length(2),
                                            Constraint::Min(1),
                                        ])
                                        .margin(1)
                                        .split(dialog_area);

                                    let overall_ratio = if snapshot.total_items == 0 {
                                        0.0
                                    } else {
                                        (snapshot.done_items.min(snapshot.total_items) as f64)
                                            / (snapshot.total_items as f64)
                                    };
                                    let overall = Gauge::default()
                                        .block(Block::default().title("整體進度"))
                                        .gauge_style(Style::default().fg(theme_focus).bg(theme_panel))
                                        .ratio(overall_ratio)
                                        .label(format!("{}/{}", snapshot.done_items, snapshot.total_items));
                                    f.render_widget(overall, inner[0]);

                                    let current_name = if snapshot.current_item.is_empty() {
                                        "(準備中)".to_string()
                                    } else {
                                        snapshot.current_item.clone()
                                    };
                                    let name_line = Paragraph::new(Line::from(vec![
                                        Span::styled("Table: ", Style::default().fg(theme_dim)),
                                        Span::styled(current_name, Style::default().fg(theme_text)),
                                    ]));
                                    f.render_widget(name_line, inner[1]);

                                    let current_ratio = if snapshot.current_item_rows_total == 0 {
                                        0.0
                                    } else {
                                        (snapshot
                                            .current_item_rows_done
                                            .min(snapshot.current_item_rows_total)
                                            as f64)
                                            / (snapshot.current_item_rows_total as f64)
                                    };
                                    let current = Gauge::default()
                                        .block(Block::default().title("目前表進度"))
                                        .gauge_style(Style::default().fg(Color::Cyan).bg(theme_panel))
                                        .ratio(current_ratio)
                                        .label(format!(
                                            "{}/{}",
                                            snapshot.current_item_rows_done, snapshot.current_item_rows_total
                                        ));
                                    f.render_widget(current, inner[3]);

                                    let hint = Paragraph::new(Line::from(vec![
                                        Span::styled("狀態: ", Style::default().fg(theme_dim)),
                                        Span::styled(snapshot.status, Style::default().fg(theme_text)),
                                        Span::raw("   "),
                                        Span::styled("[Ctrl+T] 中斷", Style::default().fg(theme_focus)),
                                    ]));
                                    f.render_widget(hint, inner[4]);
                                })?;
                                thread::sleep(Duration::from_millis(120));
                            }

                            let run_once = handle
                                .join()
                                .map_err(|_| anyhow!("run thread panicked"))?;
                            match run_once {
                                Ok(_) => logs.push(tr(cli.lang, "run_ok").to_string()),
                                Err(e) => logs.push(format!("{}: {e}", tr(cli.lang, "run_fail"))),
                            }
                        } else {
                            logs.push(format!("{}: {cmd}", tr(cli.lang, "unknown_cmd")));
                        }
                    }
                    KeyCode::Char(ch) => {
                        if sql_popup {
                            sql_input.push(ch)
                        } else if table_edit_popup && table_edit_focus == TableEditFocus::SqlEditor {
                            table_edit_sql.push(ch)
                        } else if table_edit_popup && table_edit_focus == TableEditFocus::CellInput {
                            table_edit_cell.push(ch)
                        } else if table_edit_popup && row_form_popup && table_edit_focus == TableEditFocus::RowForm {
                            if let Some(v) = row_form_values.get_mut(row_form_idx) {
                                v.push(ch);
                            }
                        } else if editing_form {
                            form_input.push(ch)
                        } else if focus_area == FocusArea::DbPane
                            && current_page == ScreenPage::DbOps
                            && !table_edit_popup
                            && ch.is_ascii_alphabetic()
                        {
                            let now = Instant::now();
                            if db_search_last_input
                                .map(|last| now.duration_since(last) > Duration::from_millis(1200))
                                .unwrap_or(false)
                            {
                                db_search_query.clear();
                            }
                            db_search_last_input = Some(now);
                            db_search_query.push(ch.to_ascii_lowercase());

                            let find_in_current = |needle: &str| -> Option<usize> {
                                if needle.is_empty() {
                                    return None;
                                }
                                match db_tab {
                                    DbTab::Table => table_stats
                                        .iter()
                                        .position(|t| t.name.to_ascii_lowercase().starts_with(needle))
                                        .or_else(|| {
                                            table_stats.iter().position(|t| {
                                                t.name.to_ascii_lowercase().contains(needle)
                                            })
                                        }),
                                    DbTab::View => views
                                        .iter()
                                        .position(|v| v.to_ascii_lowercase().starts_with(needle))
                                        .or_else(|| {
                                            views.iter().position(|v| v.to_ascii_lowercase().contains(needle))
                                        }),
                                    DbTab::StoredProcedure => sps
                                        .iter()
                                        .position(|v| v.to_ascii_lowercase().starts_with(needle))
                                        .or_else(|| {
                                            sps.iter().position(|v| v.to_ascii_lowercase().contains(needle))
                                        }),
                                }
                            };

                            if let Some(found_idx) = find_in_current(&db_search_query) {
                                db_selected_idx = found_idx;
                            } else {
                                let single = ch.to_ascii_lowercase().to_string();
                                if let Some(found_idx) = find_in_current(&single) {
                                    db_search_query = single;
                                    db_selected_idx = found_idx;
                                }
                            }
                        } else {
                            input.push(ch);
                            cmd_history_idx = None;
                        }
                    }
                    _ => {}
                }
                }
                _ => {}
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    terminal.backend_mut().execute(DisableMouseCapture)?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    run_result?;
    Ok(cli)
}
