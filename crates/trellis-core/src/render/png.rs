/// Convert SVG data to PNG using resvg.
pub fn svg_to_png(svg_data: &[u8]) -> Result<Vec<u8>, PngError> {
    let options = resvg::usvg::Options::default();

    let tree = resvg::usvg::Tree::from_data(svg_data, &options)
        .map_err(|e| PngError {
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

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| PngError {
            message: format!("Failed to create pixmap {}x{}", width, height),
        })?;

    // White background
    pixmap.fill(resvg::tiny_skia::Color::WHITE);

    resvg::render(&tree, resvg::usvg::Transform::default(), &mut pixmap.as_mut());

    pixmap.encode_png().map_err(|e| PngError {
        message: format!("Failed to encode PNG: {}", e),
    })
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
    fn test_invalid_svg_to_png() {
        let result = svg_to_png(b"not valid svg");
        assert!(result.is_err());
    }
}
