/// Character-based text size estimation (no font metrics).
///
/// These heuristics approximate text dimensions in pixels based on
/// character count. They will be refined in later milestones when
/// actual font rendering is available.

const CHAR_WIDTH: f64 = 8.0;
const LINE_HEIGHT: f64 = 16.0;
const PADDING_H: f64 = 20.0;
const PADDING_V: f64 = 12.0;

/// Estimate the rendered width of text in pixels.
///
/// Uses average character width multiplied by the longest line,
/// plus horizontal padding on both sides.
pub fn calculate_text_width(text: &str) -> f64 {
    let max_line_len = text
        .lines()
        .map(|line| line.len())
        .max()
        .unwrap_or(0);
    max_line_len as f64 * CHAR_WIDTH + PADDING_H * 2.0
}

/// Estimate the rendered height of text in pixels.
///
/// Uses line count multiplied by line height, plus vertical
/// padding on both sides.
pub fn calculate_text_height(text: &str) -> f64 {
    let num_lines = text.lines().count().max(1);
    num_lines as f64 * LINE_HEIGHT + PADDING_V * 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_char_width() {
        let w = calculate_text_width("A");
        assert_eq!(w, 1.0 * CHAR_WIDTH + PADDING_H * 2.0);
    }

    #[test]
    fn test_multiline_height() {
        let h = calculate_text_height("line1\nline2\nline3");
        assert_eq!(h, 3.0 * LINE_HEIGHT + PADDING_V * 2.0);
    }

    #[test]
    fn test_empty_text() {
        let w = calculate_text_width("");
        assert_eq!(w, PADDING_H * 2.0);
        let h = calculate_text_height("");
        assert_eq!(h, 1.0 * LINE_HEIGHT + PADDING_V * 2.0);
    }

    #[test]
    fn test_multiline_width_uses_longest() {
        let w = calculate_text_width("short\nthis is much longer");
        let expected = "this is much longer".len() as f64 * CHAR_WIDTH + PADDING_H * 2.0;
        assert_eq!(w, expected);
    }
}
