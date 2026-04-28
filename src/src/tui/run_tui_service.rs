    let run_result = (|| -> Result<()> {
        loop {
            terminal.draw(|f| {
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

                let top_title = Paragraph::new(Line::from(vec![
                    Span::styled("◆ ", Style::default().fg(Color::Yellow)),
                    Span::raw(tr(cli.lang, "top_title")),
                ]));
                f.render_widget(top_title, chunks[0]);

                let top = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
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

                let state_widget = Paragraph::new(form_lines).block(
                    Block::default()
                        .title(tr(cli.lang, "form_title"))
                        .borders(Borders::ALL),
                );
                f.render_widget(state_widget, top[0]);

                let cmd_widget = Paragraph::new(vec![
                    Line::from("/set <field> <value>"),
                    Line::from("/unset <field>"),
                    Line::from("/show"),
                    Line::from("/show-hot-key"),
                    Line::from("/refresh-db"),
                    Line::from("/test-connect"),
                    Line::from("/list-table"),
                    Line::from("/list-view"),
                    Line::from("/list-stored-procedure"),
                    Line::from("/exec-sql"),
                    Line::from("/run"),
                    Line::from("/quit"),
                ])
                .block(Block::default().title(tr(cli.lang, "cmd_title")).borders(Borders::ALL));
                f.render_widget(cmd_widget, top[1]);

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
                let logs_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .split(chunks[2]);

                let log_widget = Paragraph::new(log_lines)
                    .block(Block::default().title(tr(cli.lang, "log_title")).borders(Borders::ALL));
                f.render_widget(log_widget, logs_area[0]);

                let db_title = match db_tab {
                    DbTab::Table => "DB: table",
                    DbTab::View => "DB: view",
                    DbTab::StoredProcedure => "DB: stored procedure",
                };
                let db_lines = match db_tab {
                    DbTab::Table => table_stats
                        .iter()
                        .enumerate()
                        .take((logs_area[1].height as usize).saturating_sub(2))
                        .map(|(idx, t)| {
                            let prefix = if idx == db_selected_idx { ">" } else { " " };
                            Line::from(format!(
                                "{prefix} {} | rows={} | size={}",
                                t.name, t.rows, t.size_bytes
                            ))
                        })
                        .collect::<Vec<_>>(),
                    DbTab::View => views
                        .iter()
                        .enumerate()
                        .take((logs_area[1].height as usize).saturating_sub(2))
                        .map(|(idx, v)| {
                            let prefix = if idx == db_selected_idx { ">" } else { " " };
                            Line::from(format!("{prefix} {v}"))
                        })
                        .collect::<Vec<_>>(),
                    DbTab::StoredProcedure => sps
                        .iter()
                        .enumerate()
                        .take((logs_area[1].height as usize).saturating_sub(2))
                        .map(|(idx, v)| {
                            let prefix = if idx == db_selected_idx { ">" } else { " " };
                            Line::from(format!("{prefix} {v}"))
                        })
                        .collect::<Vec<_>>(),
                };
                let db_widget = Paragraph::new(db_lines)
                    .block(Block::default().title(db_title).borders(Borders::ALL));
                f.render_widget(db_widget, logs_area[1]);

                let input_widget = Paragraph::new(Line::from(vec![
                    Span::styled("> ", Style::default().fg(Color::Yellow)),
                    Span::raw(input.as_str()),
                ]))
                .block(
                    Block::default()
                        .title(if focus_area == FocusArea::Input {
                            tr(cli.lang, "input_title_focus")
                        } else {
                            tr(cli.lang, "input_title")
                        })
                        .borders(Borders::ALL),
                );
                f.render_widget(input_widget, chunks[3]);

                let status_widget = Paragraph::new(Line::from(vec![
                    Span::styled("STATUS: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!(
                        "{} | {}",
                        tr(cli.lang, "status_default"),
                        match db_tab {
                            DbTab::Table => tr(cli.lang, "status_tab_table"),
                            DbTab::View => tr(cli.lang, "status_tab_view"),
                            DbTab::StoredProcedure => tr(cli.lang, "status_tab_sp"),
                        }
                    )),
                ]));
                f.render_widget(status_widget, chunks[4]);

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
                            Constraint::Percentage(15),
                            Constraint::Length(12),
                            Constraint::Percentage(15),
                        ])
                        .split(f.size())[1];
                    f.render_widget(Clear, area);
                    let mut lines = vec![
                        Line::from(format!("SQL: {}", table_edit_sql)),
                        Line::from("------------------------------------------------"),
                    ];
                    if !table_preview_cols.is_empty() {
                        lines.push(Line::from(table_preview_cols.join(" | ")));
                        lines.push(Line::from("------------------------------------------------"));
                    }
                    for (ri, row) in table_preview_rows.iter().enumerate().take(6) {
                        let marker = if ri == table_edit_row_idx { ">" } else { " " };
                        lines.push(Line::from(format!("{marker} {}", row.join(" | "))));
                    }
                    lines.push(Line::from(""));
                    lines.push(Line::from(format!("edit cell: {}", table_edit_cell)));
                    lines.push(Line::from("[Shift+Enter] save  [Esc] close"));
                    let dlg = Paragraph::new(lines)
                        .block(Block::default().title("table edit").borders(Borders::ALL));
                    f.render_widget(dlg, area);
                }
            })?;

            if !event::poll(Duration::from_millis(200))? {
                continue;
            }
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Tab => {
                        if !sql_popup && !table_edit_popup {
                            if editing_form {
                                editing_form = false;
                                form_input.clear();
                            }
                            focus_area = match focus_area {
                                FocusArea::Form => FocusArea::Input,
                                FocusArea::Input => FocusArea::DbPane,
                                FocusArea::DbPane => FocusArea::Form,
                            };
                            logs.push(match focus_area {
                                FocusArea::Form => tr(cli.lang, "focus_form"),
                                FocusArea::Input => tr(cli.lang, "focus_input"),
                                FocusArea::DbPane => tr(cli.lang, "focus_db"),
                            });
                        }
                    }
                    KeyCode::BackTab => {
                        if !sql_popup && !table_edit_popup {
                            db_tab = match db_tab {
                                DbTab::Table => DbTab::StoredProcedure,
                                DbTab::View => DbTab::Table,
                                DbTab::StoredProcedure => DbTab::View,
                            };
                            db_selected_idx = 0;
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        break;
                    }
                    KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        if table_edit_popup {
                            table_edit_popup = false;
                            table_edit_cell.clear();
                            continue;
                        }
                    }
                    KeyCode::Esc => {
                        if table_edit_popup {
                            table_edit_popup = false;
                            table_edit_cell.clear();
                            continue;
                        }
                        break;
                    }
                    KeyCode::Backspace => {
                        if sql_popup {
                            sql_input.pop();
                        } else if editing_form {
                            form_input.pop();
                        } else {
                            input.pop();
                            cmd_history_idx = None;
                        }
                    }
                    KeyCode::Up if !editing_form && focus_area == FocusArea::Form => {
                        selected_idx = selected_idx.saturating_sub(1);
                    }
                    KeyCode::Up if !editing_form && focus_area == FocusArea::DbPane => {
                        db_selected_idx = db_selected_idx.saturating_sub(1);
                    }
                    KeyCode::Down if !editing_form && focus_area == FocusArea::Form => {
                        selected_idx = (selected_idx + 1).min(FORM_FIELDS.len().saturating_sub(1));
                    }
                    KeyCode::Down if !editing_form && focus_area == FocusArea::DbPane => {
                        let max_len = match db_tab {
                            DbTab::Table => table_stats.len(),
                            DbTab::View => views.len(),
                            DbTab::StoredProcedure => sps.len(),
                        };
                        if max_len > 0 {
                            db_selected_idx = (db_selected_idx + 1).min(max_len.saturating_sub(1));
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
                                if key.modifiers.contains(event::KeyModifiers::SHIFT) {
                                    editing_form = true;
                                    form_input = get_field_value(&cli, sel_field);
                                    continue;
                                }
                                continue;
                            } else if input.trim().is_empty() {
                                let _ = cycle_field_option(&mut cli, sel_field, 1);
                                continue;
                            }
                        }

                        if focus_area == FocusArea::DbPane {
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) {
                                if let DbTab::Table = db_tab {
                                    if let Some(ts) = table_stats.get(db_selected_idx) {
                                        match fetch_table_preview(&cli, &ts.name, 20) {
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
                                                table_edit_sql = format!(
                                                    "UPDATE {} SET <col>=<val> WHERE {}=<key>",
                                                    ts.name,
                                                    table_preview_cols.get(0).cloned().unwrap_or_default()
                                                );
                                                table_edit_popup = true;
                                            }
                                            Err(e) => logs.push(format!("table preview failed: {e}")),
                                        }
                                    }
                                }
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
                            logs.push(tr(cli.lang, "help_line").to_string());
                            if cli.lang != Lang::En {
                                logs.push("help: /set <field> <value>, /unset <field>, /show, /show-hot-key, /run, /quit, /test-connect, /list-table, /list-view, /list-stored-procedure, /exec-sql".to_string());
                            }
                        } else if cmd == "/show-hot-key" {
                            logs.push("[HOT-KEY]".to_string());
                            logs.push("Tab              : switch focus (form/input/db pane)".to_string());
                            logs.push("Shift-Tab        : switch DB tab (table/view/stored procedure)".to_string());
                            logs.push("Up/Down          : form select row OR input history OR db list select".to_string());
                            logs.push("Left/Right       : switch dropdown options in form".to_string());
                            logs.push("Enter      : edit/save form field / table cell save in table edit".to_string());
                            logs.push("Ctrl+Enter       : open table-edit popup (on DB table tab)".to_string());
                            logs.push("Ctrl+Q           : cancel /run dialog OR close table-edit dialog".to_string());
                            logs.push("Esc              : close popup / quit tui".to_string());
                            logs.push("Enter            : run command in input area".to_string());
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
                        } else if cmd == "/refresh-db" {
                            match list_table_stats(&cli) {
                                Ok(v) => table_stats = v,
                                Err(e) => logs.push(format!("table stats failed: {e}")),
                            }
                            match tui_list_views(&cli) {
                                Ok(v) => views = v,
                                Err(e) => logs.push(format!("view list failed: {e}")),
                            }
                            match tui_list_stored_procedures(&cli) {
                                Ok(v) => sps = v,
                                Err(e) => logs.push(format!("sp list failed: {e}")),
                            }
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
                        } else if cmd == "/run" {
                            logs.push(tr(cli.lang, "run_start"));
                            let cli_run = cli.clone();
                            let cancel_flag = Arc::new(AtomicBool::new(false));
                            let run_state = Arc::new(Mutex::new(String::new()));
                            let cancel_flag2 = cancel_flag.clone();
                            let run_state2 = run_state.clone();
                            let handle = thread::spawn(move || {
                                let r = execute_export_with_cancel(&cli_run, true, &cancel_flag2);
                                if let Ok(mut s) = run_state2.lock() {
                                    *s = match &r {
                                        Ok(_) => "ok".to_string(),
                                        Err(e) => format!("{e}"),
                                    };
                                }
                                r
                            });
                            let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                            let mut spin_idx = 0usize;
                            let run_started = Instant::now();

                            while !handle.is_finished() {
                                if event::poll(Duration::from_millis(10))? {
                                    if let Event::Key(k) = event::read()? {
                                        if k.kind == KeyEventKind::Press
                                            && matches!(k.code, KeyCode::Char('q'))
                                            && k.modifiers.contains(event::KeyModifiers::CONTROL)
                                        {
                                            cancel_flag.store(true, Ordering::Relaxed);
                                        }
                                    }
                                }
                                let msg = format!(
                                    "{} {}  {}ms",
                                    spinner[spin_idx % spinner.len()],
                                    tr(cli.lang, "run_progress"),
                                    run_started.elapsed().as_millis()
                                );
                                spin_idx += 1;
                                terminal.draw(|f| {
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

                                    let top_title = Paragraph::new(Line::from(vec![
                                        Span::styled("◆ ", Style::default().fg(Color::Yellow)),
                                        Span::raw(tr(cli.lang, "top_title")),
                                    ]));
                                    f.render_widget(top_title, chunks[0]);

                                    let top = Layout::default()
                                        .direction(Direction::Horizontal)
                                        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
                                        .split(chunks[1]);

                                    let snapshot = cli_snapshot_lines(&cli)
                                        .into_iter()
                                        .take(8)
                                        .map(Line::from)
                                        .collect::<Vec<_>>();
                                    let state_widget = Paragraph::new(snapshot).block(
                                        Block::default()
                                            .title(tr(cli.lang, "form_title"))
                                            .borders(Borders::ALL),
                                    );
                                    f.render_widget(state_widget, top[0]);

                                    let cmd_widget = Paragraph::new(vec![
                                        Line::from("/set <field> <value>"),
                                        Line::from("/unset <field>"),
                                        Line::from("/show"),
                                        Line::from("/show-hot-key"),
                                        Line::from("/refresh-db"),
                                        Line::from("/test-connect"),
                                        Line::from("/list-table"),
                                        Line::from("/list-view"),
                                        Line::from("/list-stored-procedure"),
                                        Line::from("/exec-sql"),
                                        Line::from("/run"),
                                        Line::from("/quit"),
                                    ])
                                    .block(
                                        Block::default()
                                            .title(tr(cli.lang, "cmd_title"))
                                            .borders(Borders::ALL),
                                    );
                                    f.render_widget(cmd_widget, top[1]);

                                    let mut render_logs = logs.clone();
                                    render_logs.push(msg.clone());
                                    let log_lines = render_logs
                                        .iter()
                                        .rev()
                                        .take((chunks[1].height as usize).saturating_sub(2))
                                        .cloned()
                                        .collect::<Vec<_>>()
                                        .into_iter()
                                        .rev()
                                        .map(Line::from)
                                        .collect::<Vec<_>>();
                                    let log_widget = Paragraph::new(log_lines).block(
                                        Block::default()
                                            .title(tr(cli.lang, "log_title"))
                                            .borders(Borders::ALL),
                                    );
                                    f.render_widget(log_widget, chunks[2]);

                                    let input_widget = Paragraph::new(Line::from(vec![
                                        Span::styled("> ", Style::default().fg(Color::Yellow)),
                                        Span::raw("(running...)"),
                                    ]))
                                    .block(
                                        Block::default()
                                            .title(tr(cli.lang, "input_title"))
                                            .borders(Borders::ALL),
                                    );
                                    f.render_widget(input_widget, chunks[3]);

                                    let status_widget = Paragraph::new(Line::from(vec![
                                        Span::styled("STATUS: ", Style::default().fg(Color::Yellow)),
                                        Span::raw(format!(
                                            "{} | {}",
                                            tr(cli.lang, "status_default"),
                                            tr(cli.lang, "run_cancel_hint")
                                        )),
                                    ]));
                                    f.render_widget(status_widget, chunks[4]);
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
                        } else if editing_form {
                            form_input.push(ch)
                        } else {
                            input.push(ch);
                            cmd_history_idx = None;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    run_result?;
    Ok(cli)
