/// Tio is a module used to control the underlying I/O of the terminal.
/// The main responsibilities of this module includes:
/// - Terminal I/O Control:
///     - Setting up the terminal before application starts: such as enabling raw mode, enabling mouse events, etc.
///     - Cleanning up the terminal after application exits: such as disabling raw mode, showing cursor, etc.
/// - Reading from Terminal Input, aka events:
///     - Basical terminal events: such as key pressed, terminal control sequence, mouse events.
///     - User-defined events: such as Tick and Render, which is generated by a async timer.
/// - Writing Output to Terminal, aka rendering:
///     - Here, we only used `ratatui::terminal::draw` to render UI in the terminal.
use color_eyre::eyre::Result;
use crossterm::{
    cursor,
    event::{Event as RawEvent, KeyEvent, MouseEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{future::FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

// TODO: Use stderr or stdout? this should be configable
type SysIO = std::io::Stderr;
fn sys_io() -> SysIO {
    std::io::stderr()
}

#[derive(Debug)]
pub enum TerminalEvent {
    Tick,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
    // (cols, rows), or say (width, height)
    Resize(u16, u16),
    Ignore,
    Error,
}

impl From<std::io::Result<RawEvent>> for TerminalEvent {
    fn from(raw_event: std::io::Result<RawEvent>) -> Self {
        if let Err(_err) = raw_event {
            // TODO: print this unknow error to log file
            return TerminalEvent::Error;
        }

        // unwrap safe here, since we have already checked the error
        match raw_event.unwrap() {
            RawEvent::Key(keyev) if keyev.kind == crossterm::event::KeyEventKind::Press => {
                TerminalEvent::Key(keyev)
            }
            RawEvent::Mouse(mouse) => TerminalEvent::Mouse(mouse),
            RawEvent::Resize(cols, rows) => TerminalEvent::Resize(cols, rows),
            _ => TerminalEvent::Ignore,
        }
    }
}

pub struct Tio {
    pub canvas: ratatui::terminal::Terminal<Backend<SysIO>>,
    pub event_rx: UnboundedReceiver<TerminalEvent>,
    pub event_tx: UnboundedSender<TerminalEvent>,
    // Token is used to stop tokio task polling the terminal event
    cancellation_token: CancellationToken,
    task: JoinHandle<()>,
    tick_rate: f64,
    render_rate: f64,
}

impl Tio {
    /// Create a new Tio instance with given `tick_rate` and `render_rate`
    ///
    /// `tick_rate`: how many times per second to send Tick event
    /// `render_rate`: how many times per second to send Render event
    pub fn new(tick_rate: f64, render_rate: f64) -> Result<Self> {
        // Convertion: Err(std::io::error::Error) -> Err(color_eyre::eyre::Report)
        let canvas = ratatui::terminal::Terminal::new(Backend::new(sys_io()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async {});

        Ok(Self {
            canvas,
            event_rx,
            event_tx,
            task,
            cancellation_token,
            tick_rate,
            render_rate,
        })
    }

    fn start(&mut self) {
        let tick_interval = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_interval = std::time::Duration::from_secs_f64(1.0 / self.render_rate);
        // _cancel_token and _event_tx will be used in a separated routine
        let _cancel_token = self.cancellation_token.clone();
        let _evt_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            // As user press key or move mouse, tty will yield events;
            // event_reader is used to read these events.
            let mut raw_event_reader = crossterm::event::EventStream::new();
            let mut tick_timer = tokio::time::interval(tick_interval);
            let mut render_timer = tokio::time::interval(render_interval);
            loop {
                let future_tick = tick_timer.tick();
                let future_render = render_timer.tick();
                let future_raw_event = raw_event_reader.next().fuse();
                tokio::select! {
                    _ = _cancel_token.cancelled() => {
                        break;
                    },
                    raw_evt = future_raw_event => {
                        if let Some(raw) = raw_evt {
                            _evt_tx.send(TerminalEvent::from(raw)).unwrap();
                        }
                    },
                    _ = future_tick => {
                        _evt_tx.send(TerminalEvent::Tick).unwrap();
                    },
                    _ = future_render => {
                        _evt_tx.send(TerminalEvent::Render).unwrap();
                    },
                }
            }
        });
    }

    fn stop(&mut self) {
        self.cancellation_token.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    pub fn enter(&mut self) -> Result<()> {
        // every process has been assigned a tty at its starting point,
        // enter_raw_mode only affected this particula virtual device.
        crossterm::terminal::enable_raw_mode()?;
        // However, orienting this tty, we still need to specify stdout or stderr to use for output
        crossterm::execute!(sys_io(), EnterAlternateScreen, cursor::Hide)?;
        self.start();
        Ok(())
    }

    pub fn leave(&mut self) -> Result<()> {
        self.stop();
        if crossterm::terminal::is_raw_mode_enabled()? {
            crossterm::execute!(sys_io(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub async fn next_event(&mut self) -> Option<TerminalEvent> {
        self.event_rx.recv().await
    }
}

mod tests {

    #[tokio::test]
    async fn test_tio() {
        use super::*;
        use crossterm::event::KeyCode;
        use ratatui::widgets::{Block, Borders, Paragraph};
        // let mut tio = Tio::new(4.0, 60.0);
        let mut tio = Tio::new(4.0, 60.0).unwrap();
        tio.enter().unwrap();
        let mut text = "Press some Key!!!".to_string();
        loop {
            if let Some(TerminalEvent::Key(k)) = tio.next_event().await {
                if k.code == KeyCode::Char('q') {
                    break;
                } else {
                    // print key code
                    text = format!("You pressed {:?}", k);
                }
            }
            tio.canvas
                .draw(|frame| {
                    frame.render_widget(
                        Paragraph::new(text.clone()).block(
                            Block::default()
                                .title("Some Styled Text")
                                .borders(Borders::ALL),
                        ),
                        frame.size(),
                    );
                })
                .unwrap();
        }
        tio.leave().unwrap();
    }
}
