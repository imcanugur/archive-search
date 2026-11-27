use crate::index::query_parse::{apply_token, tokenize_query};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeFilter {
    Image,
    Video,
    Raw,
    Other,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedQuery {
    pub terms: Vec<String>,
    pub exclude_terms: Vec<String>,
    pub dir_terms: Vec<String>,
    pub exclude_dir_terms: Vec<String>,
    pub ext_filters: Vec<String>,
    pub exclude_ext_filters: Vec<String>,
    pub type_filters: Vec<TypeFilter>,
    pub cam_filters: Vec<String>,
    pub min_year: Option<u16>,
    pub max_year: Option<u16>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub selected_dir: Option<u32>,
}

pub fn parse_query(raw: &str, selected_dir: Option<u32>) -> ParsedQuery {
    let mut query = ParsedQuery {
        selected_dir,
        ..Default::default()
    };
    for token in tokenize_query(raw) {
        apply_token(&mut query, &token);
    }
    query
}

#[cfg(test)]
mod tests {
    use super::{parse_query, TypeFilter};

    #[test]
    fn parses_mixed_turkish_and_meta_query() {
        let q = parse_query(
            "klasör:Tatil ext:.MP4,mov kam:Sony tarih:2024 boyut:>50mb Antalya",
            Some(3),
        );
        assert_eq!(q.dir_terms, vec!["tatil"]);
        assert_eq!(q.ext_filters, vec!["mp4", "mov"]);
        assert_eq!(q.cam_filters, vec!["sony"]);
        assert_eq!(q.min_year, Some(2024));
        assert_eq!(q.max_year, Some(2024));
        assert_eq!(q.min_size, Some(50 * 1024 * 1024));
        assert_eq!(q.terms, vec!["antalya"]);
        assert_eq!(q.selected_dir, Some(3));
    }

    #[test]
    fn parses_ranges_negation_types_and_quotes() {
        let q = parse_query(
            "type:raw,video size:1.5mb..10mb year:2020..2024 -ext:tmp -dir:cache -thumb \"yaz tatili\"",
            None,
        );
        assert_eq!(q.type_filters, vec![TypeFilter::Raw, TypeFilter::Video]);
        assert_eq!(q.min_size, Some(1_572_864));
        assert_eq!(q.max_size, Some(10 * 1024 * 1024));
        assert_eq!(q.min_year, Some(2020));
        assert_eq!(q.max_year, Some(2024));
        assert_eq!(q.exclude_ext_filters, vec!["tmp"]);
        assert_eq!(q.exclude_dir_terms, vec!["cache"]);
        assert_eq!(q.exclude_terms, vec!["thumb"]);
        assert_eq!(q.terms, vec!["yaz tatili"]);
    }
}
