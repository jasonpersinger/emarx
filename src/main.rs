mod cli;
mod config;
mod library;
mod parser;
mod search;
mod tui;

use anyhow::{anyhow, Context, Result};
use chrono::{Local, Utc};
use clap::Parser;
use cli::{parse_read_reference, BookmarkCommand, Cli, Command};
use config::{Bookmark, ConfigStore};
use crossterm::terminal;
use library::{format_passage, format_work, Library};
use search::SearchIndex;
use std::io::{self, IsTerminal, Write};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let library = Library::load()?;
    let mut config = ConfigStore::load()?;

    match cli.command {
        None => tui::run_tui(&library, &mut config),
        Some(Command::List) => list_works(&library),
        Some(Command::Read { reference }) => {
            let (alias, section_number) = parse_read_reference(&reference);
            read_work(&library, &alias, section_number)
        }
        Some(Command::Search { query }) => search_works(&library, &query.join(" ")),
        Some(Command::Random) => {
            let passage = library.random_passage()?;
            print_output(&format_passage(&passage, terminal_width()));
            Ok(())
        }
        Some(Command::Today) => {
            let passage = library.passage_for_date(Local::now().date_naive())?;
            print_output(&format_passage(&passage, terminal_width()));
            Ok(())
        }
        Some(Command::Info { alias }) => show_info(&library, &alias.join(" ")),
        Some(Command::Sections { alias }) => show_sections(&library, &alias.join(" ")),
        Some(Command::Stats) => show_stats(&library),
        Some(Command::Bookmarks { command }) => manage_bookmarks(
            &library,
            &mut config,
            command.unwrap_or(BookmarkCommand::List),
        ),
        Some(Command::Sources) => show_sources(&library),
        Some(Command::Intro) => tui::print_intro(io::stdout().is_terminal()),
        Some(Command::Config) => show_config(&config),
    }
}

fn list_works(library: &Library) -> Result<()> {
    let mut output = String::new();
    for work in library.works() {
        output.push_str(&format!("{} ({}) [{}]\n", work.title, work.year, work.id));
    }
    print_output(&output);
    Ok(())
}

fn read_work(library: &Library, alias: &str, section_number: Option<usize>) -> Result<()> {
    let work = library
        .resolve_work(alias)
        .with_context(|| format!("no bundled work matched '{}'", alias))?;
    let formatted = format_work(work, section_number, terminal_width());
    print_output(&formatted?);
    Ok(())
}

fn search_works(library: &Library, query: &str) -> Result<()> {
    let index = SearchIndex::new(library);
    let hits = index.search(query);
    if hits.is_empty() {
        print_output("No passages found.\n");
        return Ok(());
    }

    let mut output = String::new();
    for hit in hits.iter().take(20) {
        output.push_str(&format!(
            "{}\n  Section: {} ({})\n  Paragraph: {}\n  Snippet: {}\n  Open: {}\n",
            hit.work_title,
            hit.section_title,
            hit.section_index + 1,
            hit.paragraph_index + 1,
            highlight(&hit.snippet, query),
            hit.command
        ));
    }
    print_output(&output);

    Ok(())
}

fn show_info(library: &Library, alias: &str) -> Result<()> {
    let metadata = library
        .resolve_metadata(alias)
        .with_context(|| format!("no bundled work matched '{}'", alias))?;
    let work = library
        .work_by_id(&metadata.id)
        .ok_or_else(|| anyhow!("missing work for metadata {}", metadata.id))?;

    let mut output = String::new();
    output.push_str(&format!("{}\n", metadata.title));
    output.push_str(&format!("Authors: {}\n", metadata.authors.join(", ")));
    output.push_str(&format!("Year: {}\n", metadata.year));
    if let Some(translator) = &metadata.translator {
        output.push_str(&format!("Translator: {}\n", translator));
    }
    output.push_str(&format!("Source: {}\n", metadata.source));
    output.push_str(&format!("URL: {}\n", metadata.source_url));
    output.push_str(&format!("License: {}\n", metadata.license));
    output.push_str(&format!("Sections: {}\n", work.sections.len()));
    output.push_str(&format!("Aliases: {}\n\n", metadata.aliases.join(", ")));
    output.push_str(&format!("{}\n", metadata.description));
    print_output(&output);

    Ok(())
}

fn show_sections(library: &Library, alias: &str) -> Result<()> {
    let work = library
        .resolve_work(alias)
        .with_context(|| format!("no bundled work matched '{}'", alias))?;

    let mut output = String::new();
    output.push_str(&format!("{}\n", work.title));
    output.push_str(&format!("{} section(s)\n\n", work.sections.len()));
    for (index, section) in work.sections.iter().enumerate() {
        let paragraph_count = section.paragraphs.len();
        output.push_str(&format!(
            "{:>2}. {}  [{} paragraph{}]\n",
            index + 1,
            section.title,
            paragraph_count,
            if paragraph_count == 1 { "" } else { "s" }
        ));
    }
    print_output(&output);
    Ok(())
}

fn show_stats(library: &Library) -> Result<()> {
    let works = library.works().len();
    let sections = library
        .works()
        .iter()
        .map(|work| work.sections.len())
        .sum::<usize>();
    let paragraphs = library
        .works()
        .iter()
        .flat_map(|work| &work.sections)
        .map(|section| section.paragraphs.len())
        .sum::<usize>();
    let words = library
        .works()
        .iter()
        .flat_map(|work| &work.sections)
        .flat_map(|section| &section.paragraphs)
        .map(|paragraph| paragraph.split_whitespace().count())
        .sum::<usize>();

    let mut output = String::new();
    output.push_str("EMARX library statistics\n\n");
    output.push_str(&format!("Works:      {}\n", works));
    output.push_str(&format!("Sections:   {}\n", sections));
    output.push_str(&format!("Paragraphs: {}\n", paragraphs));
    output.push_str(&format!("Words:      {}\n", words));
    print_output(&output);
    Ok(())
}

fn manage_bookmarks(
    library: &Library,
    config: &mut ConfigStore,
    command: BookmarkCommand,
) -> Result<()> {
    match command {
        BookmarkCommand::List => list_bookmarks(library, config),
        BookmarkCommand::Add { reference } => add_bookmark(library, config, &reference),
        BookmarkCommand::Remove { number } => remove_bookmark(config, number),
        BookmarkCommand::Clear => {
            config.settings.bookmarks.clear();
            config.save()?;
            print_output("Bookmarks cleared.\n");
            Ok(())
        }
    }
}

fn list_bookmarks(library: &Library, config: &ConfigStore) -> Result<()> {
    let mut output = String::new();
    if config.settings.bookmarks.is_empty() {
        output.push_str("No bookmarks saved.\n");
    } else {
        for (index, bookmark) in config.settings.bookmarks.iter().enumerate() {
            let work_title = library
                .work_by_id(&bookmark.work_id)
                .map(|work| work.title.as_str())
                .unwrap_or(&bookmark.work_id);
            output.push_str(&format!(
                "{:>2}. {} — {}\n    Saved: {}\n    Open: emarx read {} {}\n",
                index + 1,
                work_title,
                bookmark.section_title,
                bookmark.saved_at,
                bookmark.work_id,
                bookmark_section_number(library, bookmark).unwrap_or(1)
            ));
        }
    }
    print_output(&output);
    Ok(())
}

fn add_bookmark(library: &Library, config: &mut ConfigStore, reference: &[String]) -> Result<()> {
    let (alias, section_number) = parse_read_reference(reference);
    let work = library
        .resolve_work(&alias)
        .with_context(|| format!("no bundled work matched '{}'", alias))?;
    let section_index = section_number.unwrap_or(1).saturating_sub(1);
    let section = work
        .sections
        .get(section_index)
        .ok_or_else(|| anyhow!("section {} not found in {}", section_index + 1, work.title))?;

    if let Some(existing) = config
        .settings
        .bookmarks
        .iter_mut()
        .find(|bookmark| bookmark.work_id == work.id && bookmark.section_id == section.id)
    {
        existing.saved_at = Utc::now().to_rfc3339();
        existing.section_title = section.title.clone();
    } else {
        config.settings.bookmarks.push(Bookmark {
            work_id: work.id.clone(),
            section_id: section.id.clone(),
            section_title: section.title.clone(),
            saved_at: Utc::now().to_rfc3339(),
        });
    }

    config.save()?;
    print_output(&format!("Bookmarked: {} — {}\n", work.title, section.title));
    Ok(())
}

fn remove_bookmark(config: &mut ConfigStore, number: usize) -> Result<()> {
    if number == 0 || number > config.settings.bookmarks.len() {
        return Err(anyhow!("bookmark {} not found", number));
    }
    let removed = config.settings.bookmarks.remove(number - 1);
    config.save()?;
    print_output(&format!(
        "Removed bookmark: {} — {}\n",
        removed.work_id, removed.section_title
    ));
    Ok(())
}

fn bookmark_section_number(library: &Library, bookmark: &Bookmark) -> Option<usize> {
    library.work_by_id(&bookmark.work_id).and_then(|work| {
        work.sections
            .iter()
            .position(|section| section.id == bookmark.section_id)
            .map(|index| index + 1)
    })
}

fn show_sources(library: &Library) -> Result<()> {
    let mut output = String::new();
    for metadata in library.metadata() {
        output.push_str(&format!("{}:\n", metadata.title));
        output.push_str(&format!("  Source: {}\n", metadata.source));
        output.push_str(&format!("  URL: {}\n", metadata.source_url));
        output.push_str(&format!("  License: {}\n\n", metadata.license));
    }
    print_output(&output);
    Ok(())
}

fn show_config(config: &ConfigStore) -> Result<()> {
    let output = format!(
        "{}\n\n{}\n",
        config.path().display(),
        config.as_pretty_json()?
    );
    print_output(&output);
    Ok(())
}

fn print_output(text: &str) {
    if !io::stdout().is_terminal() {
        let _ = io::stdout().lock().write_all(text.as_bytes());
        return;
    }

    if let Ok((_, height)) = terminal::size() {
        let page_size = usize::from(height.saturating_sub(2)).max(8);
        let lines = text.lines().collect::<Vec<_>>();
        if lines.len() > page_size {
            let _ = page_output(&lines, page_size);
            return;
        }
    }

    print!("{text}");
}

fn page_output(lines: &[&str], page_size: usize) -> Result<()> {
    let mut stdout = io::stdout();
    let mut index = 0usize;

    while index < lines.len() {
        let end = (index + page_size).min(lines.len());
        for line in &lines[index..end] {
            writeln!(stdout, "{line}")?;
        }
        index = end;

        if index < lines.len() {
            write!(stdout, "-- more -- [Enter/q] ")?;
            stdout.flush()?;
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer)?;
            writeln!(stdout)?;
            if buffer.trim().eq_ignore_ascii_case("q") {
                break;
            }
        }
    }

    Ok(())
}

fn terminal_width() -> usize {
    if io::stdout().is_terminal() {
        terminal::size()
            .map(|(width, _)| usize::from(width.saturating_sub(4)).max(40))
            .unwrap_or(80)
    } else {
        80
    }
}

fn highlight(snippet: &str, query: &str) -> String {
    let lower = snippet.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();
    if let Some(position) = lower.find(&query_lower) {
        let end = position + query_lower.len();
        format!(
            "{}[{}]{}",
            &snippet[..position],
            &snippet[position..end],
            &snippet[end..]
        )
    } else {
        snippet.to_string()
    }
}
