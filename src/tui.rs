use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::program::Program;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::{self, border},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Terminal,
};
use std::{fmt::Write, io, panic, str::FromStr};
use tracing::{trace, Level};
use tui_input::{backend::crossterm::EventHandler, Input};
use tui_logger::TuiLoggerWidget;

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;
/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui {
    terminal: CrosstermTerminal,
    pub input: Input,
    pub history: Vec<String>,
    history_index: usize,
}

impl Tui {
    /// Constructs a new instance of [`Tui`].
    /// Also initializes the terminal interface:
    /// It enables the raw mode and sets terminal properties.
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(std::io::stderr());
        let terminal = Terminal::new(backend)?;
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), Hide, EnterAlternateScreen, EnableMouseCapture)?;
        trace!("created Tui instance.");

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset_term().expect("failed to reset the terminal");
            panic_hook(panic);
        }));
        Ok(Self {
            terminal,
            input: Input::default(),
            history: Vec::new(),
            history_index: 0,
        })
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw(&mut self, programs: &[Program]) -> io::Result<()> {
        self.terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Min(1),
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                    Constraint::Min(3),
                ])
                .split(frame.size());

            frame.render_widget(
                Paragraph::new(
                    Line::styled(
                        format!(
                            "{}. by {}",
                            env!("CARGO_PKG_NAME"),
                            env!("CARGO_PKG_AUTHORS").replace(':', " and ")
                        ),
                        Style::default().fg(Color::Cyan),
                    )
                    .alignment(Alignment::Center),
                ),
                layout[0],
            );
            frame.render_widget(
                TuiLoggerWidget::default()
                    .output_line(false)
                    .output_file(false)
                    .style_error(Style::default().fg(Color::Red))
                    .style_debug(Style::default().fg(Color::Green))
                    .style_warn(Style::default().fg(Color::Yellow))
                    .style_trace(Style::default().fg(Color::Magenta))
                    .style_info(Style::default().fg(Color::Cyan))
                    .block(
                        Block::default()
                            .title("Logs")
                            .title_alignment(Alignment::Center)
                            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT),
                    ),
                layout[1],
            );

            let status = status(programs);
            frame.render_widget(
                status.block(
                    Block::default()
                        .title("Status")
                        .title_alignment(Alignment::Center)
                        .border_set(border::Set {
                            top_left: symbols::line::NORMAL.vertical_right,
                            top_right: symbols::line::NORMAL.vertical_left,
                            ..Default::default()
                        })
                        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT),
                ),
                layout[2],
            );

            let style = if self.input.value().parse::<Command>().is_ok() {
                Style::new().light_green()
            } else {
                Style::new().light_red()
            };
            let line = Line::from(vec!["> ".into(), Span::styled(self.input.value(), style)]);
            frame.render_widget(
                Paragraph::new(line).block(
                    Block::default()
                        .title_top(Line::from("Shell").alignment(Alignment::Center))
                        .title_bottom(Line::from(Command::HELP).alignment(Alignment::Right))
                        .border_set(border::Set {
                            top_left: symbols::line::NORMAL.vertical_right,
                            top_right: symbols::line::NORMAL.vertical_left,
                            ..Default::default()
                        })
                        .borders(Borders::ALL),
                ),
                layout[3],
            );
            frame.set_cursor(
                // Put cursor past the end of the input text. + 3 because one for offset and two for the "> ".
                layout[3].x + self.input.visual_cursor() as u16 + 3,
                // Move one line down, from the border to the input line
                layout[3].y + 1,
            )
        })?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook and drop impl to revert
    /// the terminal properties if unexpected errors occur.
    pub fn reset_term() -> io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(
            io::stdout(),
            Show,
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        Ok(())
    }

    /// scroll down the history, towards more recents commands
    pub fn history_down(&mut self) {
        if self.history_index == 0 {
            return;
        }
        self.history_index -= 1;
        if self.history_index == 0 {
            self.input.reset();
            return;
        }
        if let Some(cmd) = self.history.iter().rev().nth(self.history_index - 1) {
            self.input = Input::new(cmd.to_string());
        }
    }

    /// scroll up the history, towards older commands
    pub fn history_up(&mut self) {
        if self.history_index == self.history.len() {
            return;
        }
        self.history_index += 1;
        if let Some(cmd) = self.history.iter().rev().nth(self.history_index - 1) {
            self.input = Input::new(cmd.to_string());
        }
    }

    /// pass the event to the Input handler
    pub fn handle_other_event(&mut self, key: &crossterm::event::Event) {
        self.history_index = 0;
        self.input.handle_event(key);
    }

    /// check for a valid command and reset the input
    pub fn handle_enter(&mut self) -> Option<Command> {
        let input = self.input.value().to_string();
        self.history_index = 0;
        self.input.reset();
        match input.parse::<Command>() {
            Ok(cmd) => {
                self.history.push(input.to_string());
                Some(cmd)
            }
            Err(_) => None,
        }
    }
}

fn status(programs: &[Program]) -> Table {
    let mut rows = vec![Row::new(vec!["NAME", "RUNNING", "SINCE"])];
    // TODO: add a row for each program
    // https://docs.rs/ratatui/latest/ratatui/widgets/struct.Table.html
	for i in 0..programs.len() {
		rows.push(programs[i].status());
	}
	Table::new(rows, &[])
}

impl Drop for Tui {
    fn drop(&mut self) {
        Tui::reset_term().expect("failed to reset the terminal");
    }
}

#[derive(Debug)]
pub enum Command {
    Quit,
    Start(String),
    Stop(String),
    Restart(String),
    Reload(String),
    LogLevel(Level),
}
impl FromStr for Command {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        let mut s = lower.split_whitespace();
        let cmd = s.next().ok_or(())?;
        let arg = s.next().unwrap_or_default().to_string();
        if s.next().is_some() {
            return Err(());
        }
        if "quit".starts_with(cmd) {
            return Ok(Self::Quit);
        } else if "start".starts_with(cmd) {
            return Ok(Self::Start(arg));
        } else if "stop".starts_with(cmd) {
            return Ok(Self::Stop(arg));
        } else if "restart".starts_with(cmd) {
            return Ok(Self::Restart(arg));
        } else if "reload".starts_with(cmd) {
            return Ok(Self::Reload(arg));
        } else if "loglevel".starts_with(cmd) && !arg.is_empty() {
            return Ok(Self::LogLevel(Level::from_str(&arg).map_err(|_| ())?));
        }
        Err(())
    }
}
impl Command {
    pub const HELP: &'static str =
        "quit (2x to force) | start <name?> | stop <name?> | restart <name?> | reload <path?> | loglevel <level>";
}
