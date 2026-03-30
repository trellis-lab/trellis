use wasm_bindgen::prelude::*;

/// Render a Mermaid diagram to SVG.
///
/// # Arguments
/// * `input` - Mermaid diagram source code
/// * `config_json` - Optional JSON configuration string (see `TrellisConfig`).
///   When `None` or `Some("")`, the default configuration is used.
///
/// # Returns
/// SVG string on success, or a JS error on failure.
#[wasm_bindgen]
pub fn render(input: &str, config_json: Option<String>) -> Result<String, JsValue> {
    let config = parse_config(config_json)?;

    let graph = trellis_parser::parse(input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result = trellis_core::render(&graph, &config, trellis_core::OutputFormat::Svg)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let svg = String::from_utf8(result.data)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(svg)
}

/// Render a Mermaid diagram and return SVG + render metrics as JSON.
///
/// # Arguments
/// * `input` - Mermaid diagram source code
/// * `config_json` - Optional JSON configuration string.
///
/// # Returns
/// JSON object with `svg` (string) and `metrics` (object) fields.
#[wasm_bindgen]
pub fn render_with_metrics(input: &str, config_json: Option<String>) -> Result<String, JsValue> {
    let config = parse_config(config_json)?;

    let graph = trellis_parser::parse(input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result = trellis_core::render(&graph, &config, trellis_core::OutputFormat::Svg)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let svg = String::from_utf8(result.data)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let output = serde_json::json!({
        "svg": svg,
        "metrics": result.metrics,
    });

    Ok(output.to_string())
}

/// Parse an optional JSON config string into `TrellisConfig`.
/// Returns the default config if the string is absent or empty.
fn parse_config(config_json: Option<String>) -> Result<trellis_core::TrellisConfig, JsValue> {
    match config_json.as_deref() {
        None | Some("") => Ok(trellis_core::TrellisConfig::default()),
        Some(json) => serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Invalid config JSON: {}", e))),
    }
}
