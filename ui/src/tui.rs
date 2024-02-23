use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::{self, border},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, panic};
use tracing::trace;
use tui_input::Input;
use tui_logger::TuiLoggerWidget;
pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
pub struct Tui {
    /// Interface to the Terminal.
    terminal: CrosstermTerminal,
    /// Input handler.
    pub input: Input,
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
            input: Input::default().with_value("placeholder input".to_string()),
        })
    }

    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui:render
    pub fn draw(&mut self) -> io::Result<()> {
        self.terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(51),
                    Constraint::Percentage(50),
                    Constraint::Min(3),
                ])
                .split(frame.size());

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
                layout[0],
            );
            frame.render_widget(
                Paragraph::new(frame.count().to_string()).block(
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
                layout[1],
            );
            let line = Line::from(vec![
                "> ".into(),
                Span::styled(self.input.value(), Style::new().red()),
            ]);
            frame.render_widget(
                Paragraph::new(line).block(
                    Block::default()
                        .title("Shell")
                        .title_alignment(Alignment::Center)
                        .border_set(border::Set {
                            top_left: symbols::line::NORMAL.vertical_right,
                            top_right: symbols::line::NORMAL.vertical_left,
                            ..Default::default()
                        })
                        .borders(Borders::ALL),
                ),
                layout[2],
            );
            frame.set_cursor(
                // Put cursor past the end of the input text. + 3 because one for offset and two for the "> ".
                layout[2].x + self.input.visual_cursor() as u16 + 3,
                // Move one line down, from the border to the input line
                layout[2].y + 1,
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
}
impl Drop for Tui {
    fn drop(&mut self) {
        Tui::reset_term().expect("failed to reset the terminal");
    }
}
