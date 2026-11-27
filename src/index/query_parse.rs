use crate::index::arena::fold_string;
use crate::index::query::{ParsedQuery, TypeFilter};

pub fn tokenize_query(raw: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for ch in raw.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if ch.is_whitespace() && !in_quotes {
            if !cur.is_empty() {
                tokens.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(ch);
        }
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

pub fn apply_token(query: &mut ParsedQuery, token: &str) {
    let folded = fold_string(token);
    if folded.is_empty() {
        return;
    }
    if let Some(neg) = folded.strip_prefix('-') {
        apply_negated_token(query, neg);
        return;
    }
    if let Some(val) = strip_any(&folded, &["klasor:", "dir:"]) {
        push_csv(&mut query.dir_terms, val, false);
    } else if let Some(val) = folded.strip_prefix("ext:") {
        push_csv(&mut query.ext_filters, val, true);
    } else if let Some(val) = strip_any(&folded, &["type:", "tur:", "kind:"]) {
        apply_type_filter(query, val);
    } else if let Some(val) = strip_any(&folded, &["cam:", "kam:"]) {
        push_csv(&mut query.cam_filters, val, false);
    } else if let Some(val) = strip_any(&folded, &["date:", "tarih:", "year:"]) {
        apply_year_filter(query, val);
    } else if let Some(val) = strip_any(&folded, &["boyut:", "size:"]) {
        apply_size_filter(query, val);
    } else {
        query.terms.push(folded);
    }
}

fn apply_negated_token(query: &mut ParsedQuery, neg: &str) {
    if neg.is_empty() {
        return;
    }
    if let Some(val) = neg.strip_prefix("ext:") {
        push_csv(&mut query.exclude_ext_filters, val, true);
    } else if let Some(val) = strip_any(neg, &["klasor:", "dir:"]) {
        push_csv(&mut query.exclude_dir_terms, val, false);
    } else {
        query.exclude_terms.push(neg.to_owned());
    }
}

fn strip_any<'a>(text: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes.iter().find_map(|p| text.strip_prefix(p))
}

fn push_csv(dst: &mut Vec<String>, val: &str, trim_dot: bool) {
    for part in val.split(',') {
        let clean = if trim_dot {
            part.trim_start_matches('.')
        } else {
            part
        };
        if !clean.is_empty() {
            dst.push(clean.to_owned());
        }
    }
}

fn apply_type_filter(query: &mut ParsedQuery, val: &str) {
    for part in val.split(',') {
        let kind = match part {
            "photo" | "photos" | "image" | "images" | "img" | "foto" => {
                Some(TypeFilter::Image)
            }
            "video" | "videos" | "vid" => Some(TypeFilter::Video),
            "raw" => Some(TypeFilter::Raw),
            "other" | "file" | "doc" => Some(TypeFilter::Other),
            _ => None,
        };
        if let Some(k) = kind.filter(|k| !query.type_filters.contains(k)) {
            query.type_filters.push(k);
        }
    }
}

fn apply_year_filter(query: &mut ParsedQuery, expr: &str) {
    if let Some((a, b)) = expr.split_once("..") {
        query.min_year = a.parse::<u16>().ok().or(query.min_year);
        query.max_year = b.parse::<u16>().ok().or(query.max_year);
    } else if let Some(rest) = expr.strip_prefix(">=") {
        query.min_year = rest.parse::<u16>().ok();
    } else if let Some(rest) = expr.strip_prefix('>') {
        query.min_year = rest.parse::<u16>().ok().map(|y| y.saturating_add(1));
    } else if let Some(rest) = expr.strip_prefix("<=") {
        query.max_year = rest.parse::<u16>().ok();
    } else if let Some(rest) = expr.strip_prefix('<') {
        query.max_year = rest.parse::<u16>().ok().map(|y| y.saturating_sub(1));
    } else if let Ok(exact) = expr.parse::<u16>() {
        query.min_year = Some(exact);
        query.max_year = Some(exact);
    }
}

fn apply_size_filter(query: &mut ParsedQuery, expr: &str) {
    if let Some((a, b)) = expr.split_once("..") {
        query.min_size = parse_size_bytes(a).or(query.min_size);
        query.max_size = parse_size_bytes(b).or(query.max_size);
    } else if let Some(rest) = expr.strip_prefix(">=").or_else(|| expr.strip_prefix('>')) {
        query.min_size = parse_size_bytes(rest);
    } else if let Some(rest) = expr.strip_prefix("<=").or_else(|| expr.strip_prefix('<')) {
        query.max_size = parse_size_bytes(rest);
    }
}

pub fn parse_size_bytes(raw: &str) -> Option<u64> {
    let (num_str, mult) = if let Some(n) = raw.strip_suffix("gb") {
        (n, 1024_f64 * 1024.0 * 1024.0)
    } else if let Some(n) = raw.strip_suffix("mb") {
        (n, 1024_f64 * 1024.0)
    } else if let Some(n) = raw.strip_suffix("kb") {
        (n, 1024_f64)
    } else if let Some(n) = raw.strip_suffix('b') {
        (n, 1.0_f64)
    } else {
        (raw, 1.0_f64)
    };
    let val: f64 = num_str.parse().ok()?;
    if !val.is_finite() || val < 0.0 {
        return None;
    }
    Some((val * mult).round() as u64)
}
