use crate::library::Library;

#[derive(Debug, Clone)]
pub struct SearchHit {
    #[allow(dead_code)]
    pub work_id: String,
    pub work_title: String,
    pub section_index: usize,
    pub section_title: String,
    pub paragraph_index: usize,
    pub snippet: String,
    pub command: String,
    pub score: i32,
}

#[derive(Debug, Clone)]
struct SearchRecord {
    work_id: String,
    work_title: String,
    section_index: usize,
    section_title: String,
    paragraph_index: usize,
    paragraph: String,
}

#[derive(Debug, Clone)]
pub struct SearchIndex {
    records: Vec<SearchRecord>,
}

impl SearchIndex {
    pub fn new(library: &Library) -> Self {
        let mut records = Vec::new();

        for work in library.works() {
            for (section_index, section) in work.sections.iter().enumerate() {
                for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
                    records.push(SearchRecord {
                        work_id: work.id.clone(),
                        work_title: work.title.clone(),
                        section_index,
                        section_title: section.title.clone(),
                        paragraph_index,
                        paragraph: paragraph.clone(),
                    });
                }
            }
        }

        Self { records }
    }

    pub fn search(&self, query: &str) -> Vec<SearchHit> {
        self.search_internal(query, None)
    }

    pub fn search_work(&self, work_id: &str, query: &str) -> Vec<SearchHit> {
        self.search_internal(query, Some(work_id))
    }

    fn search_internal(&self, query: &str, only_work: Option<&str>) -> Vec<SearchHit> {
        let normalized_query = query.trim().to_ascii_lowercase();
        if normalized_query.is_empty() {
            return Vec::new();
        }

        let mut hits = self
            .records
            .iter()
            .filter(|record| only_work.map(|id| id == record.work_id).unwrap_or(true))
            .filter_map(|record| {
                match_position(&record.work_title, &normalized_query)
                    .or_else(|| match_position(&record.section_title, &normalized_query))
                    .or_else(|| match_position(&record.paragraph, &normalized_query))
                    .map(|(position, word_boundary)| {
                        let score = 100 + if word_boundary { 25 } else { 0 }
                            - i32::try_from(position).unwrap_or(0);
                        SearchHit {
                            work_id: record.work_id.clone(),
                            work_title: record.work_title.clone(),
                            section_index: record.section_index,
                            section_title: record.section_title.clone(),
                            paragraph_index: record.paragraph_index,
                            snippet: snippet(&record.paragraph, position, normalized_query.len()),
                            command: format!(
                                "emarx read {} {}",
                                record.work_id,
                                record.section_index + 1
                            ),
                            score,
                        }
                    })
            })
            .collect::<Vec<_>>();

        hits.sort_by(|left, right| right.score.cmp(&left.score));
        hits
    }
}

fn match_position(haystack: &str, query: &str) -> Option<(usize, bool)> {
    let lower = haystack.to_ascii_lowercase();
    if let Some(position) = lower.find(query) {
        let boundary = position == 0
            || !lower[..position]
                .chars()
                .last()
                .map(|ch| ch.is_ascii_alphanumeric())
                .unwrap_or(false);
        return Some((position, boundary));
    }

    let normalized_haystack = normalize_search_text(&lower);
    let normalized_query = normalize_search_text(query);
    if !normalized_query.is_empty() && normalized_haystack.contains(&normalized_query) {
        return Some((0, true));
    }

    let terms = query
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();

    if terms.len() > 1 && terms.iter().all(|term| lower.contains(term)) {
        let position = terms
            .iter()
            .filter_map(|term| lower.find(term))
            .min()
            .unwrap_or(0);
        return Some((position, true));
    }

    None
}

fn normalize_search_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut previous_space = true;
    for ch in text.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            ' '
        };
        if mapped == ' ' {
            if !previous_space {
                output.push(' ');
            }
            previous_space = true;
        } else {
            output.push(mapped);
            previous_space = false;
        }
    }
    output.trim().to_string()
}

fn snippet(text: &str, position: usize, len: usize) -> String {
    let start = position.saturating_sub(50);
    let end = (position + len + 90).min(text.len());
    let raw = text.get(start..end).unwrap_or(text).trim();
    let prefix = if start > 0 { "..." } else { "" };
    let suffix = if end < text.len() { "..." } else { "" };
    format!("{prefix}{raw}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::SearchIndex;
    use crate::library::Library;

    #[test]
    fn search_finds_known_phrase() {
        let library = Library::load().expect("library should load");
        let index = SearchIndex::new(&library);
        let hits = index.search("working men of all countries unite");

        assert!(!hits.is_empty());
        assert!(hits.iter().any(|hit| hit.work_id == "communist-manifesto"));
    }
}
