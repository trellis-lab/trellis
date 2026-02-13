use wasm_bindgen::prelude::*;

/// Render a Mermaid diagram to SVG
///
/// This is a placeholder implementation for M1.
/// Full implementation will be in M13.
///
/// # Arguments
/// * `input` - Mermaid diagram source code
/// * `config_json` - Optional JSON configuration string
///
/// # Returns
/// SVG string or error message
#[wasm_bindgen]
pub fn render(input: &str, config_json: Option<String>) -> Result<String, JsValue> {
    let _ = (input, config_json);

    let graph = trellis_parser::parse(input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let config = trellis_core::TrellisConfig::default();

    let result = trellis_core::render(&graph, &config, trellis_core::OutputFormat::Svg)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let svg = String::from_utf8(result.data)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(svg)
}

/// Render a Mermaid diagram and return metrics
///
/// # Arguments
/// * `input` - Mermaid diagram source code
/// * `config_json` - Optional JSON configuration string
///
/// # Returns
/// JSON object with `svg` and `metrics` fields
#[wasm_bindgen]
pub fn render_with_metrics(input: &str, config_json: Option<String>) -> Result<String, JsValue> {
    let _ = config_json;

    let graph = trellis_parser::parse(input)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let config = trellis_core::TrellisConfig::default();

    let result = trellis_core::render(&graph, &config, trellis_core::OutputFormat::Svg)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let svg = String::from_utf8(result.data)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let output = serde_json::json!({
        "svg": svg,
        "metrics": result.metrics
    });

    Ok(output.to_string())
}
