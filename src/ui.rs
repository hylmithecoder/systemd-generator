use crate::app::{ActiveStep, App, ConfigField};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs, Wrap},
};

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header & Stepper
            Constraint::Length(4), // System / Device Info Panel
            Constraint::Min(10),   // Main Step Content
            Constraint::Length(3), // Footer Help
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_device_info(f, app, chunks[1]);

    match app.step {
        ActiveStep::BinaryPicker => render_binary_picker(f, app, chunks[2]),
        ActiveStep::ServiceConfig => render_service_config(f, app, chunks[2]),
        ActiveStep::PreviewDeploy => render_preview_deploy(f, app, chunks[2]),
        ActiveStep::StatusModal => {
            render_preview_deploy(f, app, chunks[2]);
            render_status_modal(f, app);
        }
    }

    render_footer(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        " 1. Pick Binary ",
        " 2. Configure Service ",
        " 3. Preview & Deploy ",
    ];

    let selected_index = match app.step {
        ActiveStep::BinaryPicker => 0,
        ActiveStep::ServiceConfig => 1,
        ActiveStep::PreviewDeploy | ActiveStep::StatusModal => 2,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    " Service File Generator TUI ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .select(selected_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn render_device_info(f: &mut Frame, app: &App, area: Rect) {
    let sys = &app.sys_info;
    let target_str = app.target_init_system.as_str();
    let is_detected = app.target_init_system == sys.init_system;
    let init_badge = if is_detected {
        format!("{} (Auto-detected)", target_str)
    } else {
        format!("{} (Manual)", target_str)
    };

    let text = vec![Line::from(vec![
        Span::styled("Host: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            &sys.hostname,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("OS: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{} ({})", sys.os_name, sys.os_version),
            Style::default().fg(Color::LightBlue),
        ),
        Span::raw(" | "),
        Span::styled("Arch: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&sys.cpu_arch, Style::default().fg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled("Init: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            init_badge,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("User: ", Style::default().fg(Color::DarkGray)),
        Span::styled(&sys.current_user, Style::default().fg(Color::LightGreen)),
    ])];

    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Target Device Environment "),
    );

    f.render_widget(p, area);
}

fn render_binary_picker(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Input box
            Constraint::Min(5),    // Derived paths card
        ])
        .split(area);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Binary Executable Path (e.g. ~/Project/my-app/my-bin) ");

    let input_p = Paragraph::new(app.binary_path_input.as_str())
        .block(input_block)
        .style(Style::default().fg(Color::White));

    f.render_widget(input_p, chunks[0]);

    // Derived paths info
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Path Derivation & /opt Consistency ");

    let mut lines = vec![];
    if let Some(info) = &app.binary_info {
        let exists_str = if info.exists {
            Span::styled(
                " [FOUND ON DISK]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                " [NOT FOUND / PENDING]",
                Style::default().fg(Color::LightRed),
            )
        };

        lines.push(Line::from(vec![
            Span::styled("Source Binary: ", Style::default().fg(Color::Cyan)),
            Span::raw(info.source_path.to_string_lossy().to_string()),
            exists_str,
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Derived App Name: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                &info.app_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Binary Filename:  ", Style::default().fg(Color::Cyan)),
            Span::styled(&info.binary_name, Style::default().fg(Color::Yellow)),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Target ExecStart: ", Style::default().fg(Color::Green)),
            Span::styled(
                info.opt_binary_path.to_string_lossy().to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Target WorkDir:   ", Style::default().fg(Color::Green)),
            Span::styled(
                info.opt_dir.to_string_lossy().to_string(),
                Style::default().fg(Color::Green),
            ),
        ]));
    } else {
        lines.push(Line::from(Span::styled(
            "Type a binary file path above to derive service installation paths...",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let info_p = Paragraph::new(lines).block(info_block);
    f.render_widget(info_p, chunks[1]);
}

fn render_service_config(f: &mut Frame, app: &App, area: Rect) {
    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Service Configuration (Use Tab / Shift+Tab to switch fields) ");

    let inner_area = main_block.inner(area);
    f.render_widget(main_block, area);

    if let Some(cfg) = &app.config {
        let fields = vec![
            (ConfigField::UnitName, "Service Unit Name", &cfg.unit_name),
            (ConfigField::Description, "Description", &cfg.description),
            (ConfigField::ExecStart, "ExecStart Path", &cfg.exec_start),
            (
                ConfigField::WorkingDirectory,
                "Working Directory",
                &cfg.working_dir,
            ),
            (ConfigField::User, "System User", &cfg.user),
            (ConfigField::Group, "System Group", &cfg.group),
            (
                ConfigField::RestartPolicy,
                "Restart Policy (always/on-failure/no)",
                &cfg.restart_policy,
            ),
            (
                ConfigField::ExtraFlags,
                "Extra CLI Arguments",
                &cfg.extra_flags,
            ),
            (
                ConfigField::Environment,
                "Environment Variables (KEY=VAL)",
                &cfg.environment,
            ),
        ];

        let constraints: Vec<Constraint> = fields.iter().map(|_| Constraint::Length(3)).collect();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner_area);

        for (i, (field, label, val)) in fields.iter().enumerate() {
            if i >= chunks.len() {
                break;
            }
            let is_focused = app.active_config_field == *field;
            let border_color = if is_focused {
                Color::Yellow
            } else {
                Color::DarkGray
            };
            let title_color = if is_focused {
                Color::Yellow
            } else {
                Color::Gray
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    format!(" {} ", label),
                    Style::default().fg(title_color),
                ));

            let p = Paragraph::new(val.as_str()).block(block);
            f.render_widget(p, chunks[i]);
        }
    }
}

fn render_preview_deploy(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65), // Service preview
            Constraint::Percentage(35), // Deploy action panel
        ])
        .split(area);

    let preview_title = match app.target_init_system {
        crate::device::InitSystem::Systemd => " Generated Systemd .service Unit Content ",
        crate::device::InitSystem::OpenRC => " Generated OpenRC Init Script Content ",
    };

    // Render Preview
    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(preview_title);

    let preview_p = Paragraph::new(app.generated_service_content.as_str())
        .block(preview_block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Green));

    f.render_widget(preview_p, chunks[0]);

    // Render Action buttons
    let action_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Deployment Actions ");

    let action_inner = action_block.inner(chunks[1]);
    f.render_widget(action_block, chunks[1]);

    let button_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Option 0: Save local
            Constraint::Length(4), // Option 1: Deploy to system
            Constraint::Min(2),
        ])
        .split(action_inner);

    let btn0_style = if app.preview_action_index == 0 {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let btn1_style = if app.preview_action_index == 1 {
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    };

    let (label0, label1) = match app.target_init_system {
        crate::device::InitSystem::Systemd => (
            "\n [1] Save Unit File Locally ",
            "\n [2] Copy to /etc/systemd/system & Deploy ",
        ),
        crate::device::InitSystem::OpenRC => (
            "\n [1] Save Init Script Locally ",
            "\n [2] Copy to /etc/init.d & Enable ",
        ),
    };

    let p0 = Paragraph::new(label0)
        .alignment(Alignment::Center)
        .style(btn0_style);
    let p1 = Paragraph::new(label1)
        .alignment(Alignment::Center)
        .style(btn1_style);

    f.render_widget(p0, button_chunks[0]);
    f.render_widget(p1, button_chunks[1]);
}

fn render_status_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 60, f.area());
    f.render_widget(Clear, area);

    let title_color = if app.is_error_status {
        Color::Red
    } else {
        Color::Green
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(title_color))
        .title(Span::styled(
            " System Execution Log / Status ",
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        ));

    let p = Paragraph::new(app.status_message.as_str())
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));

    f.render_widget(p, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.step {
        ActiveStep::BinaryPicker => {
            " [Enter] Next Step | [i] Toggle Systemd/OpenRC | [Esc] Quit "
        }
        ActiveStep::ServiceConfig => {
            " [Tab / Up/Down] Select Field | [Enter] Next Step | [i] Toggle Systemd/OpenRC | [Esc] Back "
        }
        ActiveStep::PreviewDeploy => {
            " [Tab / Up/Down] Select Action | [Enter] Execute | [i] Toggle Systemd/OpenRC | [Esc] Back "
        }
        ActiveStep::StatusModal => " [Enter / Esc] Close Log Modal ",
    };

    let p = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(p, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
