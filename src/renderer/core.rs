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
    color_shift_counter: usize,
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
            color_shift_counter: 0,
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

    // In src/renderer/core.rs
    // Inside the draw method of the Rat struct, after rendering the mice but before the final closing brace
    pub fn draw(&mut self) -> std::io::Result<()> {
        // Increment the color shift counter for each frame
        self.color_shift_counter = (self.color_shift_counter + 1) % 7;

        let mice = self.mice.clone();
        self.hole.draw(|frame| {
            // Get available area
            let area = frame.size();

            // Create a vertical layout with title, main content, and footer
            let vertical_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Title area (1 line)
                    Constraint::Min(3),    // Main content (at least 3 lines)
                    Constraint::Length(1), // Footer area (1 line)
                ])
                .margin(1)
                .split(area);

            let title_area = vertical_layout[0];
            let main_area = vertical_layout[1];
            let footer_area = vertical_layout[2];

            // Create rainbow title text with shifting colors
            let rainbow_colors = [
                ratatui::style::Color::Red,
                ratatui::style::Color::LightRed,
                ratatui::style::Color::Yellow,
                ratatui::style::Color::Green,
                ratatui::style::Color::Cyan,
                ratatui::style::Color::Blue,
                ratatui::style::Color::Magenta,
            ];

            // Split "Rsensor" into individual styled spans with shifting colors
            let title_chars = "Rsensor".chars();
            let mut spans = Vec::new();

            for (i, ch) in title_chars.enumerate() {
                // Use counter to offset color index, creating shifting effect
                let color_index = (i + self.color_shift_counter) % rainbow_colors.len();
                spans.push(ratatui::text::Span::styled(
                    ch.to_string(),
                    ratatui::style::Style::default()
                        .fg(rainbow_colors[color_index])
                        .add_modifier(ratatui::style::Modifier::BOLD),
                ));
            }

            // Create a styled line from the spans
            let title_line = ratatui::text::Line::from(spans);

            // Create paragraph from the line
            let title = Paragraph::new(vec![title_line]);
            frame.render_widget(title, title_area);

            // Rest of the draw method remains the same...
            // [existing code for CPU, memory, and GPU displays]

            // Now subdivide the main area for content
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

            // Content layout within main area
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(cpu_height),
                    Constraint::Length(horizontal_section_height),
                    Constraint::Min(0), // Remaining space
                ])
                .split(main_area);

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

            // Add help text in the footer area - aligned to the right
            let help_text =
                Paragraph::new("ctrl-c or q to quit").alignment(ratatui::layout::Alignment::Right);
            frame.render_widget(help_text, footer_area);
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
