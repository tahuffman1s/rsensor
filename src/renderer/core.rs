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
            // Get available area
            let area = frame.size();

            // First create a layout that only uses the top portion of the screen
            // This will be sized to fit the tallest mouse
            let max_height = mice
                .iter()
                .map(|mouse| mouse.content.len() as u16 + 2) // +2 for borders
                .max()
                .unwrap_or(0);

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(max_height),
                    Constraint::Min(0), // Remaining space
                ])
                .split(area);

            // Now create a horizontal layout within the top chunk
            let horizontal_constraints: Vec<Constraint> = mice
                .iter()
                .map(|mouse| {
                    // Find the longest line in the content
                    let max_width = mouse
                        .content
                        .iter()
                        .map(|line| line.width())
                        .max()
                        .unwrap_or(0)
                        .max(mouse.title.len());

                    // Add padding for borders
                    Constraint::Min((max_width + 4) as u16)
                })
                .collect();

            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(horizontal_constraints)
                .split(main_chunks[0]);

            // Now for each mouse, create an individual vertical layout to size to exact content height
            for (idx, mouse) in mice.iter().enumerate() {
                if idx < horizontal_chunks.len() {
                    let height = mouse.content.len() as u16 + 2; // +2 for borders

                    let mouse_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(height), Constraint::Min(0)])
                        .split(horizontal_chunks[idx]);

                    let paragraph = mouse.get_paragraph();
                    frame.render_widget(paragraph, mouse_chunks[0]);
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

    pub fn content_width(&self) -> usize {
        self.content
            .iter()
            .map(|line| line.width())
            .max()
            .unwrap_or(0)
            .max(self.title.len())
    }

    // Add this method to get the number of lines
    pub fn content_height(&self) -> usize {
        self.content.len() + 2 // +2 for title bar and bottom border
    }
}
