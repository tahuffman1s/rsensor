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

            // First divide the screen vertically into three sections:
            // 1. CPU section (if present and is the first mouse)
            // 2. Memory and GPU section (horizontal layout)
            // 3. Remaining empty space

            let cpu_mouse = mice.first(); // Get the first mouse (CPU), if present

            // Calculate heights for CPU section
            let cpu_height = cpu_mouse
                .map(|mouse| mouse.content.len() as u16 + 2) // +2 for borders
                .unwrap_or(0);

            // Calculate height for horizontal section (max height of remaining mice)
            let horizontal_section_height = mice
                .iter()
                .skip(1) // Skip the CPU mouse
                .map(|mouse| mouse.content.len() as u16 + 2) // +2 for borders
                .max()
                .unwrap_or(0);

            // Main vertical layout
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(cpu_height),
                    Constraint::Length(horizontal_section_height),
                    Constraint::Min(0), // Remaining space
                ])
                .split(area);

            // Render CPU mouse at the top if present
            if let Some(cpu_mouse) = cpu_mouse {
                let paragraph = cpu_mouse.get_paragraph();
                frame.render_widget(paragraph, main_chunks[0]);
            }

            // Create horizontal layout for the remaining mice (memory and GPU)
            let remaining_mice = mice.iter().skip(1).collect::<Vec<_>>();

            if !remaining_mice.is_empty() {
                let horizontal_constraints: Vec<Constraint> = remaining_mice
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
                    .split(main_chunks[1]);

                // Render the remaining mice horizontally
                for (idx, mouse) in remaining_mice.iter().enumerate() {
                    if idx < horizontal_chunks.len() {
                        let paragraph = mouse.get_paragraph();
                        frame.render_widget(paragraph, horizontal_chunks[idx]);
                    }
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
