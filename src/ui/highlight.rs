use crate::theme;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use std::collections::HashSet;

/// Render `text` as a `Line`, bolding+yellowing every character whose index
/// appears in `indices` (precomputed by the filter step, never recomputed at
/// render time).  Falls back to plain `normal`-styled text when the slice is
/// empty.
pub fn highlight_cached(text: &str, indices: &[usize], normal: Style) -> Line<'static> {
    if indices.is_empty() {
        return Line::from(Span::styled(text.to_owned(), normal));
    }

    let matched: HashSet<usize> = indices.iter().copied().collect();
    let hl = theme::match_highlight();

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let mut in_hl = matched.contains(&0);

    for (i, ch) in text.chars().enumerate() {
        let want_hl = matched.contains(&i);
        if want_hl != in_hl {
            if !buf.is_empty() {
                spans.push(Span::styled(
                    buf.clone(),
                    if in_hl { hl } else { normal },
                ));
                buf.clear();
            }
            in_hl = want_hl;
        }
        buf.push(ch);
    }
    if !buf.is_empty() {
        spans.push(Span::styled(buf, if in_hl { hl } else { normal }));
    }

    Line::from(spans)
}
