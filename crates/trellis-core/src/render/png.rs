/// Convert SVG data to PNG using resvg.
pub fn svg_to_png(svg_data: &[u8]) -> Result<Vec<u8>, PngError> {
    let mut options = resvg::usvg::Options::default();
    let fontdb = options.fontdb_mut();
    fontdb.load_system_fonts();

    // fontdb's generic family defaults (e.g. SansSerif → "Arial") may not exist
    // in minimal environments like Docker containers. Detect and remap to an
    // available font so that text with font-family="…, sans-serif" still renders.
    set_generic_families(fontdb);

    let tree = resvg::usvg::Tree::from_data(svg_data, &options).map_err(|e| PngError {
        message: format!("Failed to parse SVG: {}", e),
    })?;

    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    if width == 0 || height == 0 {
        return Err(PngError {
            message: "SVG has zero dimensions".to_string(),
        });
    }

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| PngError {
        message: format!("Failed to create pixmap {}x{}", width, height),
    })?;

    // White background
    pixmap.fill(resvg::tiny_skia::Color::WHITE);

    resvg::render(
        &tree,
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );

    pixmap.encode_png().map_err(|e| PngError {
        message: format!("Failed to encode PNG: {}", e),
    })
}

/// Check if a font family name exists in the database.
fn has_family(fontdb: &resvg::usvg::fontdb::Database, name: &str) -> bool {
    fontdb
        .faces()
        .any(|face| face.families.iter().any(|(f, _)| f == name))
}

/// Set fontdb generic family mappings to fonts that are actually available.
///
/// By default fontdb maps `SansSerif` → "Arial", `Serif` → "Times New Roman",
/// `Monospace` → "Courier New". These fonts may not be installed (e.g. Docker).
/// We detect this and remap to commonly-available alternatives.
fn set_generic_families(fontdb: &mut resvg::usvg::fontdb::Database) {
    const SANS: &[&str] = &["Arial", "Liberation Sans", "DejaVu Sans", "Noto Sans"];
    const SERIF: &[&str] = &[
        "Times New Roman",
        "Liberation Serif",
        "DejaVu Serif",
        "Noto Serif",
    ];
    const MONO: &[&str] = &[
        "Courier New",
        "Liberation Mono",
        "DejaVu Sans Mono",
        "Noto Sans Mono",
    ];

    if let Some(family) = SANS.iter().find(|f| has_family(fontdb, f)) {
        fontdb.set_sans_serif_family(family.to_string());
    }
    if let Some(family) = SERIF.iter().find(|f| has_family(fontdb, f)) {
        fontdb.set_serif_family(family.to_string());
    }
    if let Some(family) = MONO.iter().find(|f| has_family(fontdb, f)) {
        fontdb.set_monospace_family(family.to_string());
    }
}

/// Error type for PNG conversion failures.
#[derive(Debug, Clone)]
pub struct PngError {
    pub message: String,
}

impl std::fmt::Display for PngError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PNG conversion error: {}", self.message)
    }
}

impl std::error::Error for PngError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_svg_to_png() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\">\
                     <rect width=\"100\" height=\"100\" fill=\"red\"/></svg>";
        let result = svg_to_png(svg);
        assert!(result.is_ok());
        let png = result.unwrap();
        // PNG starts with the magic number
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_svg_with_text_renders_to_png() {
        let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"100\">\
                     <rect width=\"200\" height=\"100\" fill=\"white\"/>\
                     <text x=\"100\" y=\"50\" text-anchor=\"middle\" font-size=\"14\" \
                      font-family=\"Arial, Helvetica, sans-serif\" fill=\"black\">Hello</text></svg>";
        let result = svg_to_png(svg);
        assert!(result.is_ok());
        let png = result.unwrap();
        // PNG with rendered text should be larger than a plain white image
        let blank_svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"100\">\
                          <rect width=\"200\" height=\"100\" fill=\"white\"/></svg>";
        let blank_png = svg_to_png(blank_svg).unwrap();
        assert!(
            png.len() > blank_png.len(),
            "PNG with text ({} bytes) should be larger than blank PNG ({} bytes)",
            png.len(),
            blank_png.len()
        );
    }

    #[test]
    fn test_invalid_svg_to_png() {
        let result = svg_to_png(b"not valid svg");
        assert!(result.is_err());
    }
}
