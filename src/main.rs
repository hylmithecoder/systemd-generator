mod app;
mod device;
mod generator;
mod openrc;
mod picker;
mod ui;

use anyhow::Result;
use app::{ActiveStep, App};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, panic};

fn main() -> Result<()> {
    // Setup panic hook to ensure terminal is restored on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Initialize Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create App instance
    let mut app = App::new();

    // Run TUI main event loop
    let res = run_app(&mut terminal, &mut app);

    // Clean up terminal on shutdown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error running servicefilegenerator: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            // Global Quit shortcut Ctrl+C
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(());
            }

            // Alt+I or F2 to toggle Init System (Systemd / OpenRC) anywhere
            if (key.modifiers.contains(KeyModifiers::ALT)
                && (key.code == KeyCode::Char('i') || key.code == KeyCode::Char('I')))
                || key.code == KeyCode::F(2)
            {
                app.toggle_init_system();
                continue;
            }

            match key.code {
                KeyCode::Esc => {
                    if app.step == ActiveStep::BinaryPicker {
                        return Ok(());
                    } else {
                        app.back_step();
                    }
                }
                KeyCode::Enter => {
                    app.proceed_step();
                }
                KeyCode::Tab | KeyCode::Down => {
                    app.next_field_or_action();
                }
                KeyCode::BackTab | KeyCode::Up => {
                    app.prev_field_or_action();
                }
                KeyCode::Backspace => {
                    app.handle_backspace();
                }
                KeyCode::Char('i') | KeyCode::Char('I')
                    if app.step == ActiveStep::PreviewDeploy =>
                {
                    app.toggle_init_system();
                }
                KeyCode::Char(c) => {
                    app.handle_char_input(c);
                }
                _ => {}
            }
        }
    }
}

