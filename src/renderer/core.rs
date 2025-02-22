use ratatui::{
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

    pub fn remove(&mut self, mouse: &Mouse) {
        self.mice.retain(|x| x != mouse);
    }

    pub fn draw(&mut self) -> std::io::Result<()> {
        let mice = self.mice.clone();
        self.hole.draw(|frame| {
            for mouse in mice {
                let area = frame.area();
                let paragraph = mouse.get_paragraph();
                frame.render_widget(paragraph, area);
            }
        })?;

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
