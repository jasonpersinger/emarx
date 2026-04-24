use crate::parser;
use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

const WORK_FILES: [(&str, &str); 8] = [
    (
        "communist-manifesto",
        include_str!("../data/works/communist-manifesto.json"),
    ),
    (
        "wage-labour-capital",
        include_str!("../data/works/wage-labour-capital.json"),
    ),
    (
        "value-price-profit",
        include_str!("../data/works/value-price-profit.json"),
    ),
    (
        "critique-gotha-programme",
        include_str!("../data/works/critique-gotha-programme.json"),
    ),
    (
        "eighteenth-brumaire",
        include_str!("../data/works/eighteenth-brumaire.json"),
    ),
    (
        "theses-feuerbach",
        include_str!("../data/works/theses-feuerbach.json"),
    ),
    (
        "preface-contribution",
        include_str!("../data/works/preface-contribution.json"),
    ),
    (
        "capital-vol-1",
        include_str!("../data/works/capital-vol-1.json"),
    ),
];

const METADATA_FILES: [(&str, &str); 8] = [
    (
        "communist-manifesto",
        include_str!("../data/metadata/communist-manifesto.json"),
    ),
    (
        "wage-labour-capital",
        include_str!("../data/metadata/wage-labour-capital.json"),
    ),
    (
        "value-price-profit",
        include_str!("../data/metadata/value-price-profit.json"),
    ),
    (
        "critique-gotha-programme",
        include_str!("../data/metadata/critique-gotha-programme.json"),
    ),
    (
        "eighteenth-brumaire",
        include_str!("../data/metadata/eighteenth-brumaire.json"),
    ),
    (
        "theses-feuerbach",
        include_str!("../data/metadata/theses-feuerbach.json"),
    ),
    (
        "preface-contribution",
        include_str!("../data/metadata/preface-contribution.json"),
    ),
    (
        "capital-vol-1",
        include_str!("../data/metadata/capital-vol-1.json"),
    ),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub paragraphs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: i32,
    pub source: String,
    pub license: String,
    pub translator: Option<String>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkMetadata {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: i32,
    pub source: String,
    pub source_url: String,
    pub license: String,
    pub translator: Option<String>,
    pub aliases: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct PassageSelection {
    pub work_id: String,
    pub work_title: String,
    pub section_index: usize,
    pub section_title: String,
    pub paragraph: String,
}

#[derive(Debug, Clone)]
pub struct Library {
    works: Vec<Work>,
    metadata: Vec<WorkMetadata>,
}

impl Library {
    pub fn load() -> Result<Self> {
        let works = WORK_FILES
            .iter()
            .map(|(_, raw)| serde_json::from_str::<Work>(raw))
            .collect::<serde_json::Result<Vec<_>>>()
            .context("failed to parse bundled work data")?;
        let metadata = METADATA_FILES
            .iter()
            .map(|(_, raw)| serde_json::from_str::<WorkMetadata>(raw))
            .collect::<serde_json::Result<Vec<_>>>()
            .context("failed to parse bundled metadata")?;

        for (expected, work) in WORK_FILES.iter().zip(works.iter()) {
            if expected.0 != work.id {
                return Err(anyhow!(
                    "bundled work id mismatch: expected {}, found {}",
                    expected.0,
                    work.id
                ));
            }
        }

        for (expected, meta) in METADATA_FILES.iter().zip(metadata.iter()) {
            if expected.0 != meta.id {
                return Err(anyhow!(
                    "bundled metadata id mismatch: expected {}, found {}",
                    expected.0,
                    meta.id
                ));
            }
        }

        Ok(Self { works, metadata })
    }

    pub fn works(&self) -> &[Work] {
        &self.works
    }

    pub fn metadata(&self) -> &[WorkMetadata] {
        &self.metadata
    }

    pub fn work_by_id(&self, id: &str) -> Option<&Work> {
        self.works.iter().find(|work| work.id == id)
    }

    pub fn metadata_by_id(&self, id: &str) -> Option<&WorkMetadata> {
        self.metadata.iter().find(|meta| meta.id == id)
    }

    pub fn resolve_work(&self, input: &str) -> Option<&Work> {
        if let Some(id) = parser::canonical_work_id(input) {
            if let Some(work) = self.work_by_id(id) {
                return Some(work);
            }
        }

        let normalized = parser::normalized(input);
        self.works.iter().find(|work| {
            parser::normalized(&work.id) == normalized
                || parser::normalized(&work.title) == normalized
                || self
                    .metadata_by_id(&work.id)
                    .map(|meta| {
                        meta.aliases
                            .iter()
                            .any(|alias| parser::normalized(alias) == normalized)
                    })
                    .unwrap_or(false)
        })
    }

    pub fn resolve_metadata(&self, input: &str) -> Option<&WorkMetadata> {
        self.resolve_work(input)
            .and_then(|work| self.metadata_by_id(&work.id))
    }

    #[cfg(test)]
    pub fn section_by_index<'a>(&self, work: &'a Work, index: usize) -> Option<&'a Section> {
        if index == 0 {
            return None;
        }
        work.sections.get(index - 1)
    }

    #[cfg(test)]
    pub fn section_by_id<'a>(&self, work: &'a Work, id: &str) -> Option<&'a Section> {
        work.sections.iter().find(|section| section.id == id)
    }

    pub fn passages(&self) -> Vec<PassageSelection> {
        let mut passages = Vec::new();
        for work in &self.works {
            for (section_index, section) in work.sections.iter().enumerate() {
                for paragraph in &section.paragraphs {
                    passages.push(PassageSelection {
                        work_id: work.id.clone(),
                        work_title: work.title.clone(),
                        section_index,
                        section_title: section.title.clone(),
                        paragraph: paragraph.clone(),
                    });
                }
            }
        }
        passages
    }

    pub fn random_passage(&self) -> Result<PassageSelection> {
        let passages = self.passages();
        if passages.is_empty() {
            return Err(anyhow!("no passages are available"));
        }

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock appears to be before the Unix epoch")?
            .as_nanos() as usize;

        Ok(passages[nanos % passages.len()].clone())
    }

    pub fn passage_for_date(&self, date: NaiveDate) -> Result<PassageSelection> {
        let passages = self.passages();
        if passages.is_empty() {
            return Err(anyhow!("no passages are available"));
        }

        let mut hasher = DefaultHasher::new();
        date.format("%Y-%m-%d").to_string().hash(&mut hasher);
        let index = (hasher.finish() as usize) % passages.len();
        Ok(passages[index].clone())
    }
}

pub fn format_work(work: &Work, section_number: Option<usize>, width: usize) -> Result<String> {
    let mut lines = Vec::new();
    lines.push(work.title.clone());
    lines.push(format!("{} ({})", work.authors.join(", "), work.year));
    if let Some(translator) = &work.translator {
        lines.push(format!("Translator: {}", translator));
    }
    lines.push(String::new());

    let sections: Vec<(usize, &Section)> = match section_number {
        Some(number) => vec![(
            number,
            work.sections
                .get(number.saturating_sub(1))
                .ok_or_else(|| anyhow!("section {} not found in {}", number, work.title))?,
        )],
        None => work
            .sections
            .iter()
            .enumerate()
            .map(|(index, section)| (index + 1, section))
            .collect(),
    };

    for (number, section) in sections {
        lines.push(format!("[{}] {}", number, section.title));
        lines.push(String::new());
        for paragraph in &section.paragraphs {
            lines.extend(wrap_paragraph(paragraph, width));
            lines.push(String::new());
        }
    }

    Ok(lines.join("\n"))
}

pub fn format_passage(passage: &PassageSelection, width: usize) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "{} — {}",
        passage.work_title, passage.section_title
    ));
    lines.push(String::new());
    lines.extend(wrap_paragraph(&passage.paragraph, width));
    lines.push(String::new());
    lines.push(format!(
        "Open: emarx read {} {}",
        passage.work_id,
        passage.section_index + 1
    ));
    lines.join("\n")
}

pub fn wrap_paragraph(text: &str, width: usize) -> Vec<String> {
    let max_width = width.max(24);
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if candidate_len > max_width && !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::Library;

    #[test]
    fn loads_metadata() {
        let library = Library::load().expect("library should load");
        let metadata = library
            .metadata_by_id("communist-manifesto")
            .expect("metadata should exist");
        assert_eq!(metadata.title, "The Communist Manifesto");
        assert!(metadata.source_url.contains("gutenberg.org"));
    }

    #[test]
    fn finds_sections_by_index_and_id() {
        let library = Library::load().expect("library should load");
        let work = library
            .work_by_id("theses-feuerbach")
            .expect("work should exist");
        let by_index = library
            .section_by_index(work, 1)
            .expect("section 1 should exist");
        let by_id = library
            .section_by_id(work, &by_index.id)
            .expect("section id should exist");

        assert!(!by_index.paragraphs.is_empty());
        assert_eq!(by_id.title, by_index.title);
    }
}
