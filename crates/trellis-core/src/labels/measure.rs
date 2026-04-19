/// Arial/Helvetica character advance widths at font-size 10px.
/// Indexed by (char as usize) - 32 for printable ASCII (U+0020..=U+007E).
/// Values are in pixels; scale by (font_size / 10.0) for other sizes.
#[rustfmt::skip]
static ARIAL_WIDTHS_AT_10: [f32; 95] = [
    2.8, 2.8, 3.6, 5.6, 5.6, 8.9, 6.7, 1.9, // ' ' ! " # $ % & '
    3.3, 3.3, 3.9, 5.9, 2.8, 3.3, 2.8, 3.1, // ( ) * + , - . /
    5.6, 5.6, 5.6, 5.6, 5.6, 5.6, 5.6, 5.6, // 0 1 2 3 4 5 6 7
    5.6, 5.6, 2.8, 2.8, 5.9, 5.9, 5.9, 5.6, // 8 9 : ; < = > ?
    10.2,6.7, 6.7, 7.2, 7.2, 6.7, 6.1, 7.8, // @ A B C D E F G
    7.2, 2.8, 5.0, 6.7, 5.6, 8.3, 7.2, 7.8, // H I J K L M N O
    6.1, 7.8, 6.7, 6.1, 5.6, 7.2, 6.7, 9.4, // P Q R S T U V W
    6.7, 6.7, 6.1, 2.8, 3.1, 2.8, 5.9, 5.6, // X Y Z [ \ ] ^ _
    5.6, 5.6, 5.6, 5.0, 5.6, 5.6, 2.8, 5.6, // ` a b c d e f g
    5.6, 2.2, 2.2, 5.0, 2.2, 8.3, 5.6, 5.6, // h i j k l m n o
    5.6, 5.6, 3.3, 5.0, 2.8, 5.6, 5.0, 6.9, // p q r s t u v w
    5.0, 5.0, 5.0, 3.4, 2.6, 3.4, 5.9,       // x y z { | } ~
];

/// Returns true if the font family string names an Arial/Helvetica-like sans-serif
/// for which the lookup table applies.
fn is_arial_like(font_family: &str) -> bool {
    let lower = font_family.to_lowercase();
    lower.contains("arial") || lower.contains("helvetica")
}

/// Measure the pixel width of `text` at the given font size and family.
///
/// Uses a pre-measured Arial lookup table for Arial/Helvetica families.
/// Falls back to `font_size * 0.60` per character for other families.
pub fn measure_text_width(text: &str, font_size: f64, font_family: &str) -> f64 {
    let scale = font_size / 10.0;
    if is_arial_like(font_family) {
        text.chars()
            .map(|c| {
                let idx = c as usize;
                if (32..=126).contains(&idx) {
                    ARIAL_WIDTHS_AT_10[idx - 32] as f64
                } else {
                    5.6 // fallback: average Latin character width
                }
            })
            .sum::<f64>()
            * scale
    } else {
        text.chars().count() as f64 * font_size * 0.60
    }
}

/// Measure the width of the longest line in a multi-line text.
pub fn measure_text_width_multiline(text: &str, font_size: f64, font_family: &str) -> f64 {
    text.lines()
        .map(|line| measure_text_width(line, font_size, font_family))
        .fold(0.0_f64, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_line_scales_with_font_size() {
        let w10 = measure_text_width("Hello", 10.0, "Arial");
        let w20 = measure_text_width("Hello", 20.0, "Arial");
        assert!((w20 - w10 * 2.0).abs() < 0.01);
    }

    #[test]
    fn fallback_for_unknown_font() {
        let w = measure_text_width("ABC", 10.0, "Comic Sans MS");
        assert!((w - 3.0 * 10.0 * 0.60).abs() < 0.01);
    }

    #[test]
    fn multiline_returns_max_width() {
        let w = measure_text_width_multiline("Hi\nHello World", 10.0, "Arial");
        let expected = measure_text_width("Hello World", 10.0, "Arial");
        assert!((w - expected).abs() < 0.01);
    }
}
