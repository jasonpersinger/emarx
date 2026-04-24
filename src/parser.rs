fn normalize_alias(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut previous_space = true;

    for ch in input.chars() {
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

pub fn canonical_work_id(input: &str) -> Option<&'static str> {
    let normalized = normalize_alias(input);

    match normalized.as_str() {
        "communist manifesto" | "manifesto" | "the communist manifesto" => {
            Some("communist-manifesto")
        }
        "capital" | "das kapital" | "capital 1" | "capital vol 1" => Some("capital-vol-1"),
        "gotha" | "critique gotha programme" | "critique of the gotha programme" => {
            Some("critique-gotha-programme")
        }
        "brumaire" | "eighteenth brumaire" | "the eighteenth brumaire of louis bonaparte" => {
            Some("eighteenth-brumaire")
        }
        "wage labour" | "wage labor" | "wage labour and capital" | "wage labor and capital" => {
            Some("wage-labour-capital")
        }
        "value price profit" | "value price and profit" => Some("value-price-profit"),
        "feuerbach" | "theses on feuerbach" => Some("theses-feuerbach"),
        "preface" | "preface to a contribution to the critique of political economy" => {
            Some("preface-contribution")
        }
        "communist manifesto samuel moore" => Some("communist-manifesto"),
        _ if normalized.contains("communist manifesto") => Some("communist-manifesto"),
        _ if normalized.contains("value") && normalized.contains("profit") => {
            Some("value-price-profit")
        }
        _ if normalized.contains("wage")
            && (normalized.contains("labour") || normalized.contains("labor")) =>
        {
            Some("wage-labour-capital")
        }
        _ => None,
    }
}

pub fn normalized(input: &str) -> String {
    normalize_alias(input)
}

#[cfg(test)]
mod tests {
    use super::canonical_work_id;

    #[test]
    fn resolves_known_aliases() {
        assert_eq!(canonical_work_id("manifesto"), Some("communist-manifesto"));
        assert_eq!(
            canonical_work_id("communist manifesto"),
            Some("communist-manifesto")
        );
        assert_eq!(canonical_work_id("wage labor"), Some("wage-labour-capital"));
        assert_eq!(canonical_work_id("gotha"), Some("critique-gotha-programme"));
        assert_eq!(canonical_work_id("brumaire"), Some("eighteenth-brumaire"));
        assert_eq!(canonical_work_id("preface"), Some("preface-contribution"));
        assert_eq!(canonical_work_id("capital vol 1"), Some("capital-vol-1"));
    }
}
