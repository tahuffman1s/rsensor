use ratatui::{
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub struct Rat {
    hole: Terminal<CrosstermBackend<std::io::Stdout>>,
    mice: Vec<Mouse>,
}

#[derive(PartialEq, Clone)]
pub struct Mouse {
    title: String,
    content: Vec<Line<'static>>,
}

impl Rat {
    pub fn new() -> Self {
        Rat {
            hole: ratatui::init(),
            mice: Vec::new(),
        }
    }

    pub fn add(&mut self, mouse: Mouse) {
        self.mice.push(mouse);
    }

    pub fn clear(&mut self) {
        self.mice.clear();
    }

    pub fn remove(&mut self, mouse: &Mouse) {
        self.mice.retain(|x| x != mouse);
    }

    pub fn draw(&mut self) -> std::io::Result<()> {
        let mice = self.mice.clone();
        self.hole.draw(|frame| {
            // Create a layout that divides the screen horizontally
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    mice.iter()
                        .map(|_| Constraint::Percentage((100 / mice.len()) as u16))
                        .collect::<Vec<Constraint>>(),
                )
                .split(frame.size());

            // Render each mouse in its own chunk
            for (idx, mouse) in mice.iter().enumerate() {
                if idx < chunks.len() {
                    let paragraph = mouse.get_paragraph();
                    frame.render_widget(paragraph, chunks[idx]);
                }
            }
        })?;

        Ok(())
    }

    pub fn cleanup(&mut self) -> std::io::Result<()> {
        use crossterm::execute;
        use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};

        disable_raw_mode()?;
        execute!(self.hole.backend_mut(), LeaveAlternateScreen)?;
        self.hole.show_cursor()?;

        Ok(())
    }
}

impl Mouse {
    pub fn new(title: String) -> Self {
        Mouse {
            title: title,
            content: Vec::new(),
        }
    }

    pub fn add(&mut self, content: String) {
        let line = Line::from(content);
        self.content.push(line);
    }

    pub fn get_paragraph(&self) -> Paragraph {
        Paragraph::new(self.content.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.title.clone()),
        )
    }
}
