use std::io;
use std::time::{Duration, Instant};

use clap::Parser;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    duration: String,
}

enum DurationUnit {
    Seconds,
    Minutes,
    Hours,
}

struct DurationInput {
    unit: DurationUnit,
    duration: u64,
}

impl DurationInput {
    pub fn get_duration(self) -> u64 {
        match self.unit {
            DurationUnit::Seconds => self.duration,
            DurationUnit::Minutes => 60 * self.duration,
            DurationUnit::Hours => 60 * 60 * self.duration,
        }
    }
}

impl TryFrom<String> for DurationInput {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let unit_char = value.chars().into_iter().last().ok_or("empty input")?;

        let unit = match unit_char {
            's' => DurationUnit::Seconds,
            'm' => DurationUnit::Minutes,
            'h' => DurationUnit::Hours,
            _ => return Err("unknown unit"),
        };

        let duration: u64 = match value[..value.len() - 1].parse() {
            Ok(v) => v,
            Err(_) => return Err("no duration"),
        };

        Ok(DurationInput { unit, duration })
    }
}

pub struct App {
    start: Instant,
    duration: u64,
    exit: bool,
}

impl App {
    pub fn new(duration: u64) -> Self {
        App {
            start: Instant::now(),
            duration: duration,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            self.elapsed_reached();
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::poll(Duration::from_secs(1)) {
            Ok(event_available) => {
                if event_available {
                    match event::read()? {
                        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                            self.handle_key_event(key_event)
                        }
                        _ => {}
                    };
                } else {
                    {}
                }
            }
            Err(_) => panic!("handle_events"),
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn elapsed_reached(&mut self) {
        let elapsed = self.start.elapsed().as_secs();

        if elapsed == self.duration {
            self.exit()
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Focus ".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let elapsed_text = self.start.elapsed().as_secs().to_string();

        let remaining_text = Text::from(vec![Line::from(vec![
            "Elapsed: ".into(),
            elapsed_text.into(),
            " / ".into(),
            self.duration.to_string().into(),
        ])]);

        Paragraph::new(remaining_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let duration_input = match DurationInput::try_from(args.duration) {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: {e}");
            return Ok(());
        }
    };

    ratatui::run(|terminal| App::new(duration_input.get_duration()).run(terminal))
}
