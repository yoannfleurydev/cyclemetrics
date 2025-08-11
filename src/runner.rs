use anyhow::{Ok, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use gpx::read;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style, Stylize, palette::tailwind::SLATE},
    symbols::{self},
    text::{Line, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use crate::{
    gpx::{gpx_elevation_gain, gpx_start_end_date, gpx_track_name},
    gpx_total_distance,
};

const SELECTED_STYLE: Style = Style::new().add_modifier(Modifier::BOLD);

/// Compute the total track distance of one or more GPX files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Paths or glob patterns pointing to GPX files
    #[arg(required = true)]
    gpx_files: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct App {
    file_list: FileList,
    grand_total_km: f64,
    exit: bool,
}

#[derive(Debug)]
struct FileList {
    files: Vec<FileItem>,
    state: ListState,
}

#[derive(Debug, Clone)]
struct FileItem {
    file_name: String,
    distance: f64,
    name: String,
    start_date: String,
    elevation: String,
}

pub fn run_cyclemetrics(args: Args) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = App::default().run(&mut terminal, args);
    ratatui::restore();

    result
}

impl Default for App {
    fn default() -> Self {
        Self {
            file_list: FileList {
                files: vec![],
                state: ListState::default(),
            },
            grand_total_km: 0.0,
            exit: false,
        }
    }
}

impl FileItem {
    fn new(
        file_name: String,
        distance: f64,
        start_date: String,
        elevation: String,
        name: String,
    ) -> Self {
        Self {
            file_name,
            distance,
            start_date,
            elevation,
            name,
        }
    }
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal, args: Args) -> Result<()> {
        // Iterate over the supplied paths / glob patterns
        for gpx_path in &args.gpx_files {
            // Resolve glob patterns if necessary
            let files = glob::glob(gpx_path.to_str().unwrap())?;
            for file_res in files {
                let file_path = file_res?;

                // Read the GPX file
                let file = File::open(&file_path)?;
                let reader = BufReader::new(file);
                let gpx = read(reader)?;

                // Compute distance
                let distance_m = gpx_total_distance(&gpx);
                let distance_km = distance_m / 1_000.0;

                let name = gpx_track_name(&gpx).unwrap_or("Activity");
                let elevation = gpx_elevation_gain(&gpx);

                let start_end_dates = gpx_start_end_date(&gpx);

                self.file_list.files.push(FileItem::new(
                    start_end_dates.map_or(String::new(), |(start, _)| {
                        format!("{} {}", start.format("%d-%m-%Y"), name.to_string())
                    }),
                    distance_km,
                    start_end_dates.map_or(String::new(), |(start, _)| {
                        format!("{}", start.format("%d-%m-%Y"))
                    }),
                    format!("{}", elevation.round()),
                    name.to_string(),
                ));
                self.grand_total_km += distance_km;
            }
        }

        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_next(&mut self) {
        self.file_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.file_list.state.select_previous();
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main_area, footer_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);

        let [list_area, detail_area] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        self.render_list(list_area, buf);
        self.render_detail(detail_area, buf);
        self.render_footer(footer_area, buf);
    }
}

impl App {
    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Activities").centered())
            .borders(Borders::RIGHT)
            .border_set(symbols::border::EMPTY);

        let items: Vec<ListItem> = self
            .file_list
            .files
            .iter()
            .enumerate()
            .map(|(_, file)| ListItem::from(file))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.file_list.state);
    }

    fn render_footer(&mut self, area: Rect, buf: &mut Buffer) {
        let grand_total = Text::from(vec![Line::from(vec![
            "Grand Total: ".into(),
            format_distance(self.grand_total_km).to_string().yellow(),
        ])]);
        Paragraph::new(grand_total).centered().render(area, buf);
    }

    fn render_detail(&mut self, area: Rect, buf: &mut Buffer) {
        let info = if let Some(i) = self.file_list.state.selected() {
            let file_info: FileItem = self.file_list.files[i].clone();

            format!(
                "Distance: {} Elevation: {:>4}m {}, Start date: {}",
                format_distance(file_info.distance),
                file_info.elevation,
                file_info.name,
                file_info.start_date
            )
        } else {
            "No activity selected...".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("Activity Detail").centered())
            .border_set(symbols::border::EMPTY)
            .padding(Padding::horizontal(1));

        // We can now render the item infoq
        Paragraph::new(info)
            .block(block)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl From<&FileItem> for ListItem<'_> {
    fn from(value: &FileItem) -> Self {
        let line = Line::styled(format!("{}", value.file_name), SLATE.c200);

        ListItem::new(line)
    }
}

fn format_distance(distance: f64) -> String {
    format!("{:>8.3}km", distance)
}
