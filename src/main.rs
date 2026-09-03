use color_eyre::eyre::{Ok, Result};
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    widgets::{Paragraph, Widget},
};

fn main() -> Result<()> {
    color_eyre::install();
    let terminal = ratatui::init();

    let res = run(terminal);
    ratatui::restore();
    res
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        // Rendering
        terminal.draw(render);
        // Input handling
        if let Event::Key(key) = event::read()? {
            match key.code {
                event::KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn render(frame: &mut Frame) {
    Paragraph::new("text").render(frame.area(), frame.buffer_mut());
}
