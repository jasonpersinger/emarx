use crate::config::{AppConfig, Bookmark, ConfigStore, ReadingPosition};
use crate::library::{Library, Section, Work};
use crate::search::{SearchHit, SearchIndex};
use anyhow::Result;
use chrono::Utc;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io::{self, Write};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivePanel {
    Works,
    Sections,
    Text,
}

#[derive(Debug, Clone, Copy)]
struct Theme {
    name: &'static str,
    bg: Color,
    fg: Color,
    accent: Color,
    border: Color,
    muted: Color,
    highlight_bg: Color,
    highlight_fg: Color,
}

impl Theme {
    fn all() -> [Theme; 5] {
        [
            Theme {
                name: "Dialectic Dark",
                bg: Color::Rgb(18, 18, 20),
                fg: Color::Rgb(230, 225, 220),
                accent: Color::Rgb(196, 39, 54),
                border: Color::Rgb(140, 32, 43),
                muted: Color::Rgb(130, 130, 135),
                highlight_bg: Color::Rgb(128, 27, 37),
                highlight_fg: Color::Rgb(250, 240, 235),
            },
            Theme {
                name: "Reading Room",
                bg: Color::Rgb(247, 241, 227),
                fg: Color::Rgb(49, 44, 35),
                accent: Color::Rgb(92, 61, 46),
                border: Color::Rgb(178, 151, 111),
                muted: Color::Rgb(122, 107, 88),
                highlight_bg: Color::Rgb(215, 197, 142),
                highlight_fg: Color::Rgb(38, 34, 28),
            },
            Theme {
                name: "Red Banner",
                bg: Color::Rgb(90, 8, 17),
                fg: Color::Rgb(252, 240, 236),
                accent: Color::Rgb(255, 205, 92),
                border: Color::Rgb(180, 25, 41),
                muted: Color::Rgb(244, 190, 190),
                highlight_bg: Color::Rgb(221, 53, 72),
                highlight_fg: Color::Rgb(255, 248, 242),
            },
            Theme {
                name: "Parchment",
                bg: Color::Rgb(242, 226, 179),
                fg: Color::Rgb(69, 46, 24),
                accent: Color::Rgb(126, 81, 37),
                border: Color::Rgb(168, 127, 76),
                muted: Color::Rgb(124, 102, 69),
                highlight_bg: Color::Rgb(198, 154, 92),
                highlight_fg: Color::Rgb(50, 35, 18),
            },
            Theme {
                name: "Monochrome",
                bg: Color::Black,
                fg: Color::White,
                accent: Color::White,
                border: Color::Gray,
                muted: Color::DarkGray,
                highlight_bg: Color::White,
                highlight_fg: Color::Black,
            },
        ]
    }
}

pub fn print_intro(animated: bool) -> Result<()> {
    let mut stdout = io::stdout();
    let lines = [
        "███████╗███╗   ███╗ █████╗ ██████╗ ██╗  ██╗",
        "██╔════╝████╗ ████║██╔══██╗██╔══██╗╚██╗██╔╝",
        "█████╗  ██╔████╔██║███████║██████╔╝ ╚███╔╝ ",
        "██╔══╝  ██║╚██╔╝██║██╔══██║██╔══██╗ ██╔██╗ ",
        "███████╗██║ ╚═╝ ██║██║  ██║██║  ██║██╔╝ ██╗",
        "╚══════╝╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝",
        "",
        "Read Marx from the command line.",
    ];

    for line in lines {
        writeln!(stdout, "{line}")?;
        stdout.flush()?;
        if animated {
            std::thread::sleep(Duration::from_millis(40));
        }
    }

    Ok(())
}

pub fn run_tui(library: &Library, config: &mut ConfigStore) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(library, config.settings.clone());
    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    config.settings = app.settings;
    config.save()?;

    result
}

struct App<'a> {
    library: &'a Library,
    search_index: SearchIndex,
    settings: AppConfig,
    work_index: usize,
    section_index: usize,
    active_panel: ActivePanel,
    scroll: u16,
    search_mode: bool,
    search_input: String,
    jump_mode: bool,
    jump_input: String,
    work_hits: Vec<SearchHit>,
    hit_index: usize,
    status: String,
    show_splash: bool,
    show_help: bool,
    show_bookmarks: bool,
    bookmark_index: usize,
}

impl<'a> App<'a> {
    fn new(library: &'a Library, settings: AppConfig) -> Self {
        let mut app = Self {
            library,
            search_index: SearchIndex::new(library),
            settings,
            work_index: 0,
            section_index: 0,
            active_panel: ActivePanel::Works,
            scroll: 0,
            search_mode: false,
            search_input: String::new(),
            jump_mode: false,
            jump_input: String::new(),
            work_hits: Vec::new(),
            hit_index: 0,
            status: String::from("Ready"),
            show_splash: true,
            show_help: false,
            show_bookmarks: false,
            bookmark_index: 0,
        };
        app.restore_position();
        app
    }

    fn restore_position(&mut self) {
        if let Some(position) = self.settings.last_position.clone() {
            if let Some(work_index) = self
                .library
                .works()
                .iter()
                .position(|work| work.id == position.work_id)
            {
                self.work_index = work_index;
                if let Some(section_index) = self
                    .current_work()
                    .sections
                    .iter()
                    .position(|section| section.id == position.section_id)
                {
                    self.section_index = section_index;
                    self.scroll = position.scroll;
                }
            }
        }
    }

    fn current_work(&self) -> &Work {
        &self.library.works()[self.work_index]
    }

    fn current_section(&self) -> &Section {
        &self.current_work().sections[self.section_index]
    }

    fn current_theme(&self) -> Theme {
        Theme::all()[self.settings.theme % Theme::all().len()]
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key(key)? {
                        break;
                    }
                }
            }
        }

        self.persist_position();
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.show_splash {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') => return Ok(true),
                _ => {
                    self.show_splash = false;
                    return Ok(false);
                }
            }
        }

        if self.show_help {
            match key.code {
                KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('?') | KeyCode::Char('q') => {
                    self.show_help = false;
                }
                _ => {}
            }
            return Ok(false);
        }

        if self.show_bookmarks {
            return Ok(self.handle_bookmark_key(key));
        }

        if self.search_mode {
            return Ok(self.handle_search_key(key));
        }

        if self.jump_mode {
            return Ok(self.handle_jump_key(key));
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::Home => self.home(),
            KeyCode::End => self.end(),
            KeyCode::Left => self.prev_panel(),
            KeyCode::Right => self.next_panel(),
            KeyCode::Tab => self.next_panel(),
            KeyCode::BackTab => self.prev_panel(),
            KeyCode::Enter => self.activate_panel(),
            KeyCode::Char('/') => {
                self.search_mode = true;
                self.search_input.clear();
                self.status = "Search current work".to_string();
            }
            KeyCode::Char('n') => self.next_search_hit(),
            KeyCode::Char('N') => self.prev_search_hit(),
            KeyCode::Char('t') => {
                self.settings.theme = (self.settings.theme + 1) % Theme::all().len();
                self.status = format!("Theme: {}", self.current_theme().name);
            }
            KeyCode::Char('b') => {
                let added = self.toggle_bookmark();
                self.status = if added {
                    "Bookmark added".to_string()
                } else {
                    "Bookmark removed".to_string()
                };
            }
            KeyCode::Char('B') => {
                self.show_bookmarks = true;
                self.bookmark_index = self
                    .bookmark_index
                    .min(self.settings.bookmarks.len().saturating_sub(1));
            }
            KeyCode::Char('g') => {
                self.jump_mode = true;
                self.jump_input.clear();
            }
            KeyCode::Char('h') | KeyCode::Char('?') => {
                self.show_help = true;
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_bookmark_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('B') | KeyCode::Char('q') => {
                self.show_bookmarks = false;
            }
            KeyCode::Up => {
                self.bookmark_index = self.bookmark_index.saturating_sub(1);
            }
            KeyCode::Down => {
                if self.bookmark_index + 1 < self.settings.bookmarks.len() {
                    self.bookmark_index += 1;
                }
            }
            KeyCode::Enter => {
                self.jump_to_bookmark();
                self.show_bookmarks = false;
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if !self.settings.bookmarks.is_empty() {
                    self.settings.bookmarks.remove(self.bookmark_index);
                    self.bookmark_index = self
                        .bookmark_index
                        .min(self.settings.bookmarks.len().saturating_sub(1));
                    self.status = "Bookmark deleted".to_string();
                }
            }
            _ => {}
        }
        false
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
            }
            KeyCode::Enter => {
                self.work_hits = self
                    .search_index
                    .search_work(&self.current_work().id, &self.search_input);
                self.hit_index = 0;
                self.search_mode = false;
                if self.work_hits.is_empty() {
                    self.status = format!("No results for '{}'", self.search_input);
                } else {
                    self.status = format!(
                        "{} result(s) in {}",
                        self.work_hits.len(),
                        self.current_work().title
                    );
                    self.jump_to_hit(0);
                }
            }
            KeyCode::Backspace => {
                self.search_input.pop();
            }
            KeyCode::Char(ch) => {
                self.search_input.push(ch);
            }
            _ => {}
        }
        false
    }

    fn handle_jump_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => self.jump_mode = false,
            KeyCode::Enter => {
                if let Ok(section_number) = self.jump_input.parse::<usize>() {
                    if section_number > 0 && section_number <= self.current_work().sections.len() {
                        self.section_index = section_number - 1;
                        self.scroll = 0;
                        self.status = format!("Jumped to section {}", section_number);
                    } else {
                        self.status = "Section out of range".to_string();
                    }
                }
                self.jump_mode = false;
            }
            KeyCode::Backspace => {
                self.jump_input.pop();
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => self.jump_input.push(ch),
            _ => {}
        }
        false
    }

    fn move_up(&mut self) {
        match self.active_panel {
            ActivePanel::Works => {
                if self.work_index > 0 {
                    self.work_index -= 1;
                    self.section_index = 0;
                    self.scroll = 0;
                }
            }
            ActivePanel::Sections => {
                if self.section_index > 0 {
                    self.section_index -= 1;
                    self.scroll = 0;
                }
            }
            ActivePanel::Text => {
                self.scroll = self.scroll.saturating_sub(1);
            }
        }
        self.persist_position();
    }

    fn move_down(&mut self) {
        match self.active_panel {
            ActivePanel::Works => {
                if self.work_index + 1 < self.library.works().len() {
                    self.work_index += 1;
                    self.section_index = 0;
                    self.scroll = 0;
                }
            }
            ActivePanel::Sections => {
                if self.section_index + 1 < self.current_work().sections.len() {
                    self.section_index += 1;
                    self.scroll = 0;
                }
            }
            ActivePanel::Text => {
                self.scroll = self.scroll.saturating_add(1);
            }
        }
        self.persist_position();
    }

    fn page_up(&mut self) {
        if self.active_panel == ActivePanel::Text {
            self.scroll = self.scroll.saturating_sub(10);
            self.persist_position();
        }
    }

    fn page_down(&mut self) {
        if self.active_panel == ActivePanel::Text {
            self.scroll = self.scroll.saturating_add(10);
            self.persist_position();
        }
    }

    fn home(&mut self) {
        match self.active_panel {
            ActivePanel::Works => {
                self.work_index = 0;
                self.section_index = 0;
            }
            ActivePanel::Sections => self.section_index = 0,
            ActivePanel::Text => self.scroll = 0,
        }
        self.persist_position();
    }

    fn end(&mut self) {
        match self.active_panel {
            ActivePanel::Works => {
                self.work_index = self.library.works().len().saturating_sub(1);
                self.section_index = 0;
            }
            ActivePanel::Sections => {
                self.section_index = self.current_work().sections.len().saturating_sub(1);
            }
            ActivePanel::Text => self.scroll = self.scroll.saturating_add(1000),
        }
        self.persist_position();
    }

    fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Works => ActivePanel::Sections,
            ActivePanel::Sections => ActivePanel::Text,
            ActivePanel::Text => ActivePanel::Works,
        };
    }

    fn prev_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Works => ActivePanel::Text,
            ActivePanel::Sections => ActivePanel::Works,
            ActivePanel::Text => ActivePanel::Sections,
        };
    }

    fn activate_panel(&mut self) {
        self.active_panel = match self.active_panel {
            ActivePanel::Works => ActivePanel::Sections,
            ActivePanel::Sections => ActivePanel::Text,
            ActivePanel::Text => ActivePanel::Text,
        };
        self.persist_position();
    }

    fn next_search_hit(&mut self) {
        if self.work_hits.is_empty() {
            return;
        }
        self.hit_index = (self.hit_index + 1) % self.work_hits.len();
        self.jump_to_hit(self.hit_index);
    }

    fn prev_search_hit(&mut self) {
        if self.work_hits.is_empty() {
            return;
        }
        self.hit_index = if self.hit_index == 0 {
            self.work_hits.len() - 1
        } else {
            self.hit_index - 1
        };
        self.jump_to_hit(self.hit_index);
    }

    fn jump_to_hit(&mut self, index: usize) {
        if let Some(hit) = self.work_hits.get(index) {
            self.section_index = hit.section_index;
            self.scroll = 0;
            self.status = format!("Result {} of {}", index + 1, self.work_hits.len());
            self.persist_position();
        }
    }

    fn persist_position(&mut self) {
        self.settings.last_position = Some(ReadingPosition {
            work_id: self.current_work().id.clone(),
            section_id: self.current_section().id.clone(),
            section_index: self.section_index,
            scroll: self.scroll,
        });
    }

    fn toggle_bookmark(&mut self) -> bool {
        let work_id = self.current_work().id.clone();
        let section_id = self.current_section().id.clone();
        let section_title = self.current_section().title.clone();

        if let Some(index) =
            self.settings.bookmarks.iter().position(|bookmark| {
                bookmark.work_id == work_id && bookmark.section_id == section_id
            })
        {
            self.settings.bookmarks.remove(index);
            false
        } else {
            self.settings.bookmarks.push(Bookmark {
                work_id,
                section_id,
                section_title,
                saved_at: Utc::now().to_rfc3339(),
            });
            true
        }
    }

    fn jump_to_bookmark(&mut self) {
        if let Some(bookmark) = self.settings.bookmarks.get(self.bookmark_index) {
            if let Some(work_index) = self
                .library
                .works()
                .iter()
                .position(|work| work.id == bookmark.work_id)
            {
                self.work_index = work_index;
                if let Some(section_index) = self
                    .current_work()
                    .sections
                    .iter()
                    .position(|section| section.id == bookmark.section_id)
                {
                    self.section_index = section_index;
                    self.scroll = 0;
                    self.active_panel = ActivePanel::Text;
                    self.status = "Jumped to bookmark".to_string();
                    self.persist_position();
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let theme = self.current_theme();
        let full = frame.area();
        frame.render_widget(Block::default().style(Style::default().bg(theme.bg)), full);

        if self.show_splash {
            self.render_splash(frame, full);
            return;
        }

        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(full);

        let header = Paragraph::new(
            "EMARX — A terminal reader for Marx, Engels, and related public-domain texts.",
        )
        .style(Style::default().fg(theme.fg).bg(theme.bg).bold())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
        frame.render_widget(header, chunks[0]);

        let body = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

        self.render_works(frame, body[0], theme);
        self.render_sections(frame, body[1], theme);
        self.render_text(frame, body[2], theme);

        let bookmark_marker = if self.settings.bookmarks.iter().any(|bookmark| {
            bookmark.work_id == self.current_work().id
                && bookmark.section_id == self.current_section().id
        }) {
            " bookmarked"
        } else {
            ""
        };

        let status = Paragraph::new(format!(
            "↑/↓ navigate  ←/→ or Tab switch  Enter select  / search  n/N results  t theme  b bookmark  g jump  h help  q quit  |  {}  |  {}{}",
            self.current_theme().name,
            self.status,
            bookmark_marker
        ))
        .style(Style::default().fg(theme.muted).bg(theme.bg))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(theme.border)),
        );
        frame.render_widget(status, chunks[2]);

        if self.search_mode {
            self.render_popup(frame, "Search", &self.search_input, theme);
        } else if self.jump_mode {
            self.render_popup(frame, "Jump to Section", &self.jump_input, theme);
        } else if self.show_help {
            self.render_help(frame, theme);
        } else if self.show_bookmarks {
            self.render_bookmarks(frame, theme);
        }
    }

    fn render_works(&self, frame: &mut Frame, area: Rect, theme: Theme) {
        let items = self
            .library
            .works()
            .iter()
            .map(|work| ListItem::new(work.title.clone()))
            .collect::<Vec<_>>();
        let mut state = ListState::default().with_selected(Some(self.work_index));
        let widget = List::new(items)
            .block(self.panel_block("Works", self.active_panel == ActivePanel::Works, theme))
            .highlight_style(
                Style::default()
                    .bg(theme.highlight_bg)
                    .fg(theme.highlight_fg)
                    .bold(),
            )
            .highlight_symbol("» ");
        frame.render_stateful_widget(widget, area, &mut state);
    }

    fn render_sections(&self, frame: &mut Frame, area: Rect, theme: Theme) {
        let items = self
            .current_work()
            .sections
            .iter()
            .enumerate()
            .map(|(index, section)| ListItem::new(format!("{}. {}", index + 1, section.title)))
            .collect::<Vec<_>>();
        let mut state = ListState::default().with_selected(Some(self.section_index));
        let widget = List::new(items)
            .block(self.panel_block(
                "Sections",
                self.active_panel == ActivePanel::Sections,
                theme,
            ))
            .highlight_style(
                Style::default()
                    .bg(theme.highlight_bg)
                    .fg(theme.highlight_fg)
                    .bold(),
            )
            .highlight_symbol("» ");
        frame.render_stateful_widget(widget, area, &mut state);
    }

    fn render_text(&self, frame: &mut Frame, area: Rect, theme: Theme) {
        let section = self.current_section();
        let mut lines = vec![Line::from(vec![Span::styled(
            section.title.clone(),
            Style::default().fg(theme.accent).bold(),
        )])];
        lines.push(Line::default());

        for paragraph in &section.paragraphs {
            lines.push(highlight_line(
                paragraph,
                if self.search_input.is_empty() && !self.work_hits.is_empty() {
                    self.work_hits.get(self.hit_index).map(|_| "")
                } else {
                    Some(self.search_input.as_str())
                }
                .unwrap_or(""),
                theme,
            ));
            lines.push(Line::default());
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .style(Style::default().fg(theme.fg).bg(theme.bg))
            .block(self.panel_block("Text", self.active_panel == ActivePanel::Text, theme))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        frame.render_widget(paragraph, area);
    }

    fn render_popup(&self, frame: &mut Frame, title: &str, value: &str, theme: Theme) {
        let popup = centered_rect(60, 20, frame.area());
        frame.render_widget(Clear, popup);
        let widget = Paragraph::new(value.to_string())
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.accent))
                    .style(Style::default().bg(theme.bg).fg(theme.fg)),
            )
            .style(Style::default().fg(theme.fg).bg(theme.bg));
        frame.render_widget(widget, popup);
    }

    fn render_splash(&self, frame: &mut Frame, area: Rect) {
        let bg = Color::Rgb(28, 5, 8);
        let cover_red = Color::Rgb(158, 17, 31);
        let spine_red = Color::Rgb(92, 8, 16);
        let gold = Color::Rgb(245, 196, 74);
        let paper = Color::Rgb(238, 224, 190);
        let muted = Color::Rgb(204, 151, 137);

        frame.render_widget(Block::default().style(Style::default().bg(bg)), area);

        let book_height = if area.height >= 42 {
            32
        } else if area.height >= 34 {
            28
        } else {
            18
        };
        let layout = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(book_height),
            Constraint::Length(5),
            Constraint::Min(1),
        ])
        .split(area);
        let book = centered_rect(42, 100, layout[1]);

        let cover = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(gold))
            .style(Style::default().bg(cover_red));
        frame.render_widget(cover, book);

        if book.width > 12 && book.height > 5 {
            let spine = Rect {
                x: book.x + 2,
                y: book.y + 1,
                width: 3,
                height: book.height.saturating_sub(2),
            };
            frame.render_widget(
                Block::default().style(Style::default().bg(spine_red)),
                spine,
            );

            let pages = Rect {
                x: book.x + book.width.saturating_sub(4),
                y: book.y + 2,
                width: 2,
                height: book.height.saturating_sub(4),
            };
            frame.render_widget(Block::default().style(Style::default().bg(paper)), pages);
        }

        let art_height = if book.height >= 30 {
            20
        } else if book.height >= 24 {
            16
        } else {
            8
        };
        let art_width = if art_height >= 16 {
            book.width.saturating_sub(18).clamp(30, 36)
        } else {
            book.width.saturating_sub(18).clamp(26, 32)
        };
        let mut cover_lines = vec![Line::from("")];
        cover_lines.extend(scaled_hammer_sickle_art(
            art_width, art_height, gold, cover_red,
        ));
        cover_lines.extend([
            Line::from(vec![Span::styled(
                "EMARX",
                Style::default().fg(paper).bg(cover_red).bold(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "READ MARX FROM THE COMMAND LINE",
                Style::default().fg(gold).bg(cover_red),
            )]),
        ]);

        let title = Paragraph::new(Text::from(cover_lines))
            .alignment(Alignment::Center)
            .style(Style::default().bg(cover_red));
        frame.render_widget(
            title,
            book.inner(Margin {
                horizontal: 5,
                vertical: 2,
            }),
        );

        let footer = Paragraph::new(Text::from(vec![
            Line::from(vec![Span::styled(
                "A terminal reader for Marx, Engels, and related public-domain texts.",
                Style::default().fg(paper).bg(bg),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press any key to open the library.  Esc/q quits.",
                Style::default().fg(muted).bg(bg),
            )]),
        ]))
        .alignment(Alignment::Center)
        .style(Style::default().bg(bg));
        frame.render_widget(footer, layout[2]);
    }

    fn render_help(&self, frame: &mut Frame, theme: Theme) {
        let area = centered_rect(70, 70, frame.area());
        frame.render_widget(Clear, area);
        let lines = vec![
            Line::from(vec![Span::styled(
                "EMARX Help",
                Style::default().fg(theme.accent).bold(),
            )]),
            Line::from(""),
            Line::from("Up/Down           Navigate the active panel"),
            Line::from("Left/Right, Tab   Switch panels"),
            Line::from("Enter             Select work/section or focus text"),
            Line::from("PageUp/PageDown   Scroll text by a page"),
            Line::from("Home/End          Jump to start/end of active panel"),
            Line::from("/                 Search within current work"),
            Line::from("n / N             Next/previous search result"),
            Line::from("t                 Cycle themes"),
            Line::from("b                 Toggle bookmark at current section"),
            Line::from("B                 Open bookmarks"),
            Line::from("g                 Jump to section number"),
            Line::from("h / ?             Toggle this help"),
            Line::from("Esc / q           Close this help menu"),
            Line::from("q                 Quit from the main reader"),
            Line::from(""),
            Line::from("Search: type query, Enter searches current work, Esc cancels."),
            Line::from("Jump: type section number, Enter jumps, Esc cancels."),
            Line::from("Bookmarks: Up/Down select, Enter jumps, d deletes, Esc/q closes."),
        ];
        let widget = Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(theme.fg).bg(theme.bg))
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.accent))
                    .style(Style::default().bg(theme.bg)),
            );
        frame.render_widget(widget, area);
    }

    fn render_bookmarks(&self, frame: &mut Frame, theme: Theme) {
        let area = centered_rect(72, 70, frame.area());
        frame.render_widget(Clear, area);

        let items = if self.settings.bookmarks.is_empty() {
            vec![ListItem::new("No bookmarks saved.")]
        } else {
            self.settings
                .bookmarks
                .iter()
                .map(|bookmark| {
                    let work_title = self
                        .library
                        .work_by_id(&bookmark.work_id)
                        .map(|work| work.title.as_str())
                        .unwrap_or(bookmark.work_id.as_str());
                    ListItem::new(format!("{} — {}", work_title, bookmark.section_title))
                })
                .collect::<Vec<_>>()
        };
        let mut state = if self.settings.bookmarks.is_empty() {
            ListState::default()
        } else {
            ListState::default().with_selected(Some(self.bookmark_index))
        };
        let widget = List::new(items)
            .block(
                Block::default()
                    .title("Bookmarks  Enter jump  d delete  Esc close")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.accent))
                    .style(Style::default().bg(theme.bg)),
            )
            .style(Style::default().fg(theme.fg).bg(theme.bg))
            .highlight_style(
                Style::default()
                    .bg(theme.highlight_bg)
                    .fg(theme.highlight_fg)
                    .bold(),
            )
            .highlight_symbol("» ");
        frame.render_stateful_widget(widget, area, &mut state);
    }

    fn panel_block(&self, title: &str, active: bool, theme: Theme) -> Block<'static> {
        Block::default()
            .title(title.to_string())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if active { theme.accent } else { theme.border }))
            .style(Style::default().bg(theme.bg))
    }
}

fn highlight_line(text: &str, query: &str, theme: Theme) -> Line<'static> {
    if query.trim().is_empty() {
        return Line::from(text.to_string());
    }

    let lower = text.to_ascii_lowercase();
    let needle = query.to_ascii_lowercase();
    let mut spans = Vec::new();
    let mut start = 0usize;
    let mut search_from = 0usize;

    while let Some(relative) = lower[search_from..].find(&needle) {
        let found = search_from + relative;
        if found > start {
            spans.push(Span::raw(text[start..found].to_string()));
        }
        let end = found + needle.len();
        spans.push(Span::styled(
            text[found..end].to_string(),
            Style::default()
                .bg(theme.highlight_bg)
                .fg(theme.highlight_fg)
                .bold(),
        ));
        start = end;
        search_from = end;
    }

    if start < text.len() {
        spans.push(Span::raw(text[start..].to_string()));
    }

    Line::from(spans)
}

const HAMMER_SICKLE_ART: &[&str] = &[
    "####################################################################################################",
    "####################################################################################################",
    "##################################################*+################################################",
    "####################################################*=-+*###########################################",
    "#######################################################=:-=*########################################",
    "#########################################################=:::-*#####################################",
    "##########################################################*-::::=*##################################",
    "############################################################*-::::-+################################",
    "##############################################################+-:::::-*#############################",
    "################################################################=:::::::+###########################",
    "##############################*-::::::--==++*****################*-::::::-=#########################",
    "############################*-::::::::::::::::::::-+###############=::::::::=#######################",
    "###########################-:::::::::::::::::::::=##################*-::::::::=#####################",
    "#########################=:::::::::::::::::::::-######################=:::::::::+###################",
    "#######################+:::::::::::::::::::::-*########################+:::::::::-*#################",
    "#####################+-::::::::::::::::::::-*###########################*-:::::::::+################",
    "###################*:::::::::::::::::::::-+##############################*-:::::::::-###############",
    "#################*-:::::::::::::::::::::=#################################*-:::::::::-*#############",
    "################=:::::::::::::::::::::::-+#################################*-:::::::::-*############",
    "##############=:::::::::::::::::::::::::::-*################################*-::::::::::*###########",
    "############+:::::::::::::::::::::::::::::::-*###############################+::::::::::-*##########",
    "##########*-::::::::::::::::::::::::::::::::::=###############################=::::::::::=##########",
    "###########*-:::::::::::::::::-+=:::::::::::::::=#############################*-::::::::::+#########",
    "#############+-::::::::::::::=####=:::::::::::::::+############################=::::::::::-#########",
    "###############=:::::::::::-#######*-:::::::::::::::+##########################*:::::::::::+########",
    "#################=:::::::-*##########*-::::::::::::::-*#########################-::::::::::=########",
    "##################*-:::-*##############+:::::::::::::::-*#######################=::::::::::-########",
    "####################*-+##################+:::::::::::::::=######################=:::::::::::########",
    "###########################################=:::::::::::::::+####################=::::::::::-########",
    "############################################*=::::::::::::::-+##################-::::::::::-########",
    "##############################################*-::::::::::::::-*###############*:::::::::::=########",
    "################################################*-::::::::::::::-*#############-:::::::::::*########",
    "##################################################+-::::::::::::::=###########=:::::::::::-#########",
    "####################################################=:::::::::::::::=########+-:::::::::::+#########",
    "######################################################=:::::::::::::::+#####+::::::::::::-##########",
    "#######################################################*-:::::::::::::::+##-::::::::::::-*##########",
    "#################################*#######################*-::::::::::::::--:::::::::::::+###########",
    "###########################*---=-:-=#######################+:::::::::::::::::::::::::::+############",
    "#####################*-::::::::::::::-*######################+::::::::::::::::::::::::+#############",
    "####################+:::::::::::::::::::=*#####################=::::::::::::::::::::-*##############",
    "###################*:::::::::::::::::::::::-=+*#############*+=-:::::::::::::::::::=################",
    "#################+-::::::::::::::::::::::::::::::---------::::::::::::::::::::::::-*################",
    "##############*-::::::::::::::+#=:::::::::::::::::::::::::::::::::::::::::::::::::::-*##############",
    "############=::::::::::::::-=*####+-::::::::::::::::::::::::::::::::::::::::::::::::::=#############",
    "#########*-:::::::::::::-+##########*=-:::::::::::::::::::::::::::::::::::::::::::::::::=###########",
    "########-:::::::::::::::*###############+-:::::::::::::::::::::::::::::=**=:::::::::::::::+#########",
    "######+:::::::::::::::-*###################*=--::::::::::::::::::--=*######*-:::::::::::::::+#######",
    "#####-:::::::::::::::-*##########################*+++======++**##############*-::::::::::::::-*#####",
    "####-:::::::::::::::=##########################################################+:::::::::::::::=####",
    "###=::::::::::::::-##############################################################+::::::::::::::*###",
    "###=::::::::::::-+#################################################################=::::::::::::*###",
    "####-:::::::::-+####################################################################*=:::::::::=####",
    "#####+-:::::=*########################################################################*-:::::-*#####",
    "####################################################################################################",
    "####################################################################################################",
];

fn scaled_hammer_sickle_art(width: u16, height: u16, fg: Color, bg: Color) -> Vec<Line<'static>> {
    let source_height = HAMMER_SICKLE_ART.len();
    let source_width = HAMMER_SICKLE_ART
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or_default();
    if source_height == 0 || source_width == 0 || width == 0 || height == 0 {
        return Vec::new();
    }

    (0..height as usize)
        .map(|row| {
            let source_row = row * source_height / height as usize;
            let chars = HAMMER_SICKLE_ART[source_row].chars().collect::<Vec<_>>();
            let spans = (0..width as usize)
                .map(|column| {
                    let source_column = column * source_width / width as usize;
                    let source_glyph = chars.get(source_column).copied().unwrap_or(' ');
                    let (glyph, color) = match source_glyph {
                        '*' | '+' | '=' => ('█', fg),
                        '-' => ('▓', Color::Rgb(234, 176, 56)),
                        ':' => ('▒', Color::Rgb(213, 132, 46)),
                        _ => (' ', bg),
                    };
                    Span::styled(glyph.to_string(), Style::default().fg(color).bg(bg).bold())
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
