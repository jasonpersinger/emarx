use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "emarx", version, about = "Read Marx from the command line.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List bundled works
    List,
    /// Read a work or section
    Read {
        #[arg(required = true)]
        reference: Vec<String>,
    },
    /// Search across all works
    Search {
        #[arg(required = true)]
        query: Vec<String>,
    },
    /// Print a random passage
    Random,
    /// Print the deterministic passage of the day
    Today,
    /// Show source and license metadata for a work
    Info {
        #[arg(required = true)]
        alias: Vec<String>,
    },
    /// List source URLs and license notes
    Sources,
    /// Replay the startup banner
    Intro,
    /// Show config path and current settings
    Config,
}

pub fn parse_read_reference(parts: &[String]) -> (String, Option<usize>) {
    if parts.len() >= 2 {
        let marker = parts[parts.len() - 2].to_ascii_lowercase();
        if matches!(
            marker.as_str(),
            "chapter" | "chap" | "ch" | "section" | "sec" | "part"
        ) {
            if let Ok(section) = parts[parts.len() - 1].parse::<usize>() {
                if parts.len() > 2 {
                    return (parts[..parts.len() - 2].join(" "), Some(section));
                }
            }
        }
    }

    if let Some(last) = parts.last() {
        if let Some(section) = parse_number_token(last) {
            if parts.len() > 1 {
                return (parts[..parts.len() - 1].join(" "), Some(section));
            }
        }
    }

    (parts.join(" "), None)
}

fn parse_number_token(token: &str) -> Option<usize> {
    token
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
        .parse::<usize>()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::parse_read_reference;

    fn parts(input: &[&str]) -> Vec<String> {
        input.iter().map(|part| part.to_string()).collect()
    }

    #[test]
    fn parses_trailing_section_numbers() {
        assert_eq!(
            parse_read_reference(&parts(&["manifesto", "1"])),
            ("manifesto".to_string(), Some(1))
        );
    }

    #[test]
    fn parses_chapter_style_references() {
        assert_eq!(
            parse_read_reference(&parts(&["capital", "vol", "1", "chapter", "1"])),
            ("capital vol 1".to_string(), Some(1))
        );
    }
}
