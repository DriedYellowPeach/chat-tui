type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect, Alignment},
    prelude::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

#[derive(Default)]
struct App {
    should_exit: bool,
}

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    disable_raw_mode()?;
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    Ok(())
}

fn update(app: &mut App) -> Result<()> {
    use crossterm::event::{self, Event::Key, KeyCode::Char};
    if event::poll(std::time::Duration::from_millis(250))? {
        if let Key(k) = event::read()? {
            if k.kind == event::KeyEventKind::Press {
                match k.code {
                    KeyCode::Esc | Char('q') => app.should_exit = true,
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn ui(app: &App, frame: &mut Frame<'_>) {
    let outer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(frame.size());

    build_chat_panel(frame, outer_layout[1]);
    build_messages_panel(frame, outer_layout[0], 10);
}

fn build_chat_panel(frame: &mut Frame<'_>, chat_area: Rect) {

    let chat_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(chat_area);

    use ratatui::prelude::*;
    let text = vec![
        Line::from(vec![
            Span::styled("Kevin Wang", Style::new().light_blue().on_dark_gray().italic()),
            ":".into(),
        ]),
        Line::from("Hello!".blue()),
        Line::from(vec![
            Span::styled("me", Style::new().light_green().on_dark_gray().italic()),
            ":".into(),
        ]).alignment(Alignment::Right),
        Line::from("Hello Back!".green()).alignment(Alignment::Right),
    ];

    let chat_title = Span::styled("Chat", Style::default().fg(Color::Red).bg(Color::DarkGray));
    let chat_content_panel =
        Paragraph::new(text).block(Block::default().title(chat_title).title_alignment(Alignment::Center).borders(Borders::ALL));
    frame.render_widget(chat_content_panel, chat_layout[0]);

    let chat_input_panel = Paragraph::new("How is your day?").block(Block::default().title("Input").borders(Borders::ALL));
    frame.render_widget(chat_input_panel, chat_layout[1]);
}

fn build_messages_panel(frame: &mut Frame<'_>, msg_area: Rect, msg_count: usize) {
    let msg_title = Span::styled(
        "Messages",
        Style::default().fg(Color::Red).bg(Color::DarkGray),
    );
    let msg_block = Block::default().title(msg_title).title_alignment(Alignment::Center).borders(Borders::ALL);

    frame.render_widget(msg_block, msg_area);

    let shrink_area = msg_area.inner(&Margin {
        vertical: 1,
        horizontal: 1,
    });
    let cit: Vec<Constraint> = std::iter::repeat(Constraint::Ratio(1, msg_count as u32))
        .take(msg_count)
        .collect();
    let msg_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(cit)
        .split(shrink_area);

    for i in 0..msg_count {
        let msg_title = Span::styled(
            format!("Message: {i}"),
            Style::default().fg(Color::Yellow).bg(Color::DarkGray),
        );
        let message =
            Paragraph::new("Hello!").block(Block::default().title(msg_title).borders(Borders::ALL));
        frame.render_widget(message, msg_layout[i]);
    }
}

fn run() -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
    let mut app = App::default();
    loop {
        t.draw(|f| {
            ui(&app, f);
        })?;

        update(&mut app)?;

        if app.should_exit {
            break;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    startup()?;
    let res = run();
    shutdown()?;
    res?;
    Ok(())
}
