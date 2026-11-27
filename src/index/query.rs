use crate::index::arena::fold_string;

#[derive(Debug, Clone, Default)]
pub struct ParsedQuery {
    pub terms: Vec<String>,
    pub dir_terms: Vec<String>,
    pub ext_filters: Vec<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub selected_dir: Option<u32>,
}

pub fn parse_query(raw: &str, selected_dir: Option<u32>) -> ParsedQuery {
    let mut query = ParsedQuery {
        selected_dir,
        ..Default::default()
    };

    for token in raw.split_whitespace() {
        let folded = fold_string(token);
        if let Some(val) = strip_prefix_any(&folded, &["klasor:", "dir:"]) {
            if !val.is_empty() {
                query.dir_terms.push(val.to_owned());
            }
        } else if let Some(val) = folded.strip_prefix("ext:") {
            let clean = val.trim_start_matches('.');
            if !clean.is_empty() {
                query.ext_filters.push(clean.to_owned());
            }
        } else if let Some(val) = strip_prefix_any(&folded, &["boyut:", "size:"]) {
            apply_size_filter(&mut query, val);
        } else if !folded.is_empty() {
            query.terms.push(folded);
        }
    }
    query
}

fn strip_prefix_any<'a>(text: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    for prefix in prefixes {
        if let Some(rest) = text.strip_prefix(prefix) {
            return Some(rest);
        }
    }
    None
}

fn apply_size_filter(query: &mut ParsedQuery, expr: &str) {
    if let Some(rest) = expr.strip_prefix('>') {
        if let Some(bytes) = parse_size_bytes(rest) {
            query.min_size = Some(bytes);
        }
    } else if let Some(rest) = expr.strip_prefix('<') {
        if let Some(bytes) = parse_size_bytes(rest) {
            query.max_size = Some(bytes);
        }
    }
}

fn parse_size_bytes(raw: &str) -> Option<u64> {
    let (num_str, mult) = if let Some(n) = raw.strip_suffix("gb") {
        (n, 1024_u64 * 1024 * 1024)
    } else if let Some(n) = raw.strip_suffix("mb") {
        (n, 1024_u64 * 1024)
    } else if let Some(n) = raw.strip_suffix("kb") {
        (n, 1024_u64)
    } else if let Some(n) = raw.strip_suffix('b') {
        (n, 1_u64)
    } else {
        (raw, 1_u64)
    };
    let value: u64 = num_str.parse().ok()?;
    value.checked_mul(mult)
}

#[cfg(test)]
mod tests {
    use super::parse_query;

    #[test]
    fn parses_mixed_turkish_and_meta_query() {
        let q = parse_query("klasör:Tatil ext:.MP4 boyut:>50mb Antalya", Some(3));
        assert_eq!(q.dir_terms, vec!["tatil"]);
        assert_eq!(q.ext_filters, vec!["mp4"]);
        assert_eq!(q.min_size, Some(50 * 1024 * 1024));
        assert_eq!(q.terms, vec!["antalya"]);
        assert_eq!(q.selected_dir, Some(3));
    }
}
