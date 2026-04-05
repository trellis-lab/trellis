use serde_json::{json, Value};

use crate::report::DiagramReport;

// --- Public types -------------------------------------------------------------

/// One fixture's data to include in the review HTML.
pub struct ReviewEntry<'a> {
    /// Base filename, e.g. `"b05.mmd"`.
    pub fixture_name: &'a str,
    /// The annotated SVG content (UTF-8).  If not available, pass the plain SVG.
    pub annotated_svg: &'a str,
    /// The quality report for this fixture.
    pub report: &'a DiagramReport,
}

// --- Public API ---------------------------------------------------------------

/// Generate a self-contained HTML review tool.
///
/// Open the returned bytes (valid UTF-8 HTML) in any modern browser:
/// - Fixture buttons in the left sidebar switch the active diagram.
/// - Click a coloured edge overlay path to open the review panel.
/// - Toggle **Mark as improvable** and write a free-text note, then **Save Note**.
/// - All feedback is persisted in `localStorage` across page reloads.
/// - **Download feedback.json** exports the structured feedback for the AI agent.
///
/// If `entries` is empty the page renders a "no fixtures" placeholder.
pub fn generate_review_html(entries: &[ReviewEntry<'_>]) -> Vec<u8> {
    let fixtures_json = build_fixtures_json(entries);
    HTML_TEMPLATE
        .replace("__FIXTURES_JSON__", &fixtures_json)
        .into_bytes()
}

// --- Private helpers ----------------------------------------------------------

fn build_fixtures_json(entries: &[ReviewEntry<'_>]) -> String {
    let fixtures: Vec<Value> = entries
        .iter()
        .map(|e| {
            let edges: Vec<Value> = e
                .report
                .edges
                .iter()
                .map(|edge| {
                    json!({
                        "id":               edge.id,
                        "source":           edge.source,
                        "target":           edge.target,
                        "bends":            edge.bends,
                        "detour_factor":    edge.detour_factor,
                        "crossings":        edge.crossings,
                        "quality_score":    edge.quality_score,
                        "flags":            edge.flags,
                        "port_side_source": edge.port_side_source,
                        "port_side_target": edge.port_side_target,
                    })
                })
                .collect();

            json!({
                "name": e.fixture_name,
                "svg":  e.annotated_svg,
                "edges": edges,
                "global_metrics": {
                    "avg_quality":    e.report.global_metrics.avg_quality_score,
                    "total_crossings":e.report.global_metrics.total_crossings,
                    "total_bends":    e.report.global_metrics.total_bends,
                    "flagged_edges":  e.report.global_metrics.flagged_edges,
                    "routed_edges":   e.report.global_metrics.routed_edges,
                },
            })
        })
        .collect();

    serde_json::to_string(&fixtures).unwrap_or_else(|_| "[]".to_string())
}

// --- Embedded HTML template ---------------------------------------------------
//
// __FIXTURES_JSON__ is replaced at runtime with the serialised fixture array.

const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Trellis Edge Review</title>
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: system-ui, -apple-system, sans-serif; font-size: 14px;
           background: #f5f5f5; color: #1a1a1a; }

    /* ── Layout ── */
    #app { display: flex; height: 100vh; overflow: hidden; }

    #sidebar {
      width: 320px; min-width: 320px;
      background: #fff; border-right: 1px solid #ddd;
      display: flex; flex-direction: column; overflow-y: auto;
      padding: 16px; gap: 12px;
    }
    #main { flex: 1; overflow: auto; padding: 24px; background: #f0f0f0; }

    /* ── Typography ── */
    h1 { font-size: 16px; font-weight: 700; color: #333; }
    h2 { font-size: 13px; font-weight: 600; color: #555; margin-bottom: 6px; }
    h3 { font-size: 13px; font-weight: 600; }
    hr { border: none; border-top: 1px solid #eee; }

    /* ── Fixture list ── */
    #fixture-list { display: flex; flex-direction: column; gap: 4px; }
    .fixture-btn {
      text-align: left; padding: 8px 10px;
      border: 1px solid #ddd; border-radius: 6px;
      background: #fafafa; cursor: pointer; font-size: 13px;
      transition: background 0.1s;
    }
    .fixture-btn:hover { background: #f0f0f0; }
    .fixture-btn.active { background: #3b82f6; color: #fff; border-color: #3b82f6; }
    .fixture-btn .badge {
      float: right; font-size: 11px; padding: 1px 6px;
      border-radius: 10px; background: rgba(255,255,255,0.3);
    }
    .fixture-btn:not(.active) .badge { background: #fee2e2; color: #991b1b; }

    /* ── Edge panel ── */
    #edge-panel {
      border: 1px solid #ddd; border-radius: 8px;
      padding: 12px; background: #fafafa;
    }
    #edge-title { margin-bottom: 8px; word-break: break-all; }
    .metric {
      display: flex; justify-content: space-between;
      padding: 3px 0; font-size: 12px; color: #555;
    }
    .metric span { color: #888; }
    .metric.flags b { color: #b45309; }
    label.checkbox-label {
      display: flex; align-items: center; gap: 6px;
      margin: 10px 0 6px; cursor: pointer; font-size: 13px;
    }
    textarea#edge-note {
      width: 100%; height: 80px; padding: 8px;
      border: 1px solid #ddd; border-radius: 6px;
      font-family: inherit; font-size: 12px; resize: vertical;
    }
    button.save-btn {
      margin-top: 8px; width: 100%; padding: 7px;
      background: #3b82f6; color: #fff;
      border: none; border-radius: 6px;
      cursor: pointer; font-size: 13px; font-weight: 600;
    }
    button.save-btn:hover { background: #2563eb; }

    /* ── Download button ── */
    #download-btn {
      width: 100%; padding: 9px;
      background: #22c55e; color: #fff;
      border: none; border-radius: 6px;
      cursor: pointer; font-size: 13px; font-weight: 600;
      margin-top: auto;
    }
    #download-btn:hover { background: #16a34a; }

    /* ── Status bar ── */
    #status-bar { font-size: 12px; color: #666; min-height: 18px; text-align: center; }

    /* ── SVG display ── */
    #svg-container svg {
      background: #fff; border-radius: 8px;
      box-shadow: 0 1px 4px rgba(0,0,0,0.1); display: block;
    }

    /* ── Edge overlays ── */
    #svg-container [data-edge-id] { transition: stroke-width 0.1s; }
    #svg-container [data-edge-id]:hover { stroke-width: 6 !important; opacity: 1 !important; }
    #svg-container [data-edge-id].selected-edge {
      stroke-width: 6 !important; opacity: 1 !important; stroke-dasharray: 8 4;
    }
    #svg-container [data-edge-id].edge-flagged { stroke-dasharray: 6 3; }

    /* ── Fixture header ── */
    .fixture-header { display: flex; align-items: center; gap: 12px; margin-bottom: 16px; }
    .fixture-header h2 { font-size: 18px; color: #333; }
    .quality-badge {
      padding: 3px 10px; border-radius: 12px;
      font-size: 12px; font-weight: 600;
    }
    .quality-good { background: #dcfce7; color: #166534; }
    .quality-ok   { background: #fef9c3; color: #854d0e; }
    .quality-bad  { background: #fee2e2; color: #991b1b; }

    /* ── Empty state ── */
    .empty-state { color: #aaa; font-style: italic; text-align: center; padding: 40px 0; }
  </style>
</head>
<body>
  <div id="app">

    <!-- ── Sidebar ── -->
    <div id="sidebar">
      <h1>Trellis Edge Review</h1>
      <div>
        <h2>Fixtures</h2>
        <div id="fixture-list"></div>
      </div>
      <hr>
      <div id="edge-panel" hidden>
        <h3 id="edge-title"></h3>
        <div id="edge-metrics"></div>
        <label class="checkbox-label">
          <input type="checkbox" id="edge-improvable">
          Mark as improvable
        </label>
        <textarea id="edge-note"
          placeholder="Describe the routing issue (e.g. 'should route left of node C instead')…"></textarea>
        <button class="save-btn" id="save-note">Save Note</button>
      </div>
      <hr>
      <button id="download-btn">&#x2B07; Download feedback.json</button>
      <div id="status-bar"></div>
    </div>

    <!-- ── Main view ── -->
    <div id="main">
      <div class="fixture-header" id="fixture-header" hidden>
        <h2 id="fixture-title"></h2>
        <span class="quality-badge" id="quality-badge"></span>
        <span id="flagged-count" style="font-size:12px;color:#666;"></span>
      </div>
      <div id="svg-container">
        <p class="empty-state">Select a fixture from the sidebar.</p>
      </div>
    </div>

  </div><!-- /#app -->

  <!-- Fixture data injected by trellis generate-review -->
  <script id="fixtures-data" type="application/json">__FIXTURES_JSON__</script>

  <script>
  (function () {
    'use strict';

    var STORAGE_KEY = 'trellis_review_feedback';

    // ── Load fixture data ──────────────────────────────────────────────────────
    var fixtures = [];
    try {
      fixtures = JSON.parse(document.getElementById('fixtures-data').textContent);
    } catch (e) { console.error('Failed to parse fixture data:', e); }

    // ── Load saved feedback from localStorage ─────────────────────────────────
    var feedback = {};
    try {
      var raw = localStorage.getItem(STORAGE_KEY);
      if (raw) feedback = JSON.parse(raw);
    } catch (e) {}

    // ── UI state ──────────────────────────────────────────────────────────────
    var currentFixtureIdx = -1;
    var currentEdgeId = null;

    // ── Elements ──────────────────────────────────────────────────────────────
    var fixtureListEl  = document.getElementById('fixture-list');
    var svgContainer   = document.getElementById('svg-container');
    var fixtureHeader  = document.getElementById('fixture-header');
    var fixtureTitleEl = document.getElementById('fixture-title');
    var qualityBadge   = document.getElementById('quality-badge');
    var flaggedCount   = document.getElementById('flagged-count');
    var edgePanel      = document.getElementById('edge-panel');
    var edgeTitleEl    = document.getElementById('edge-title');
    var edgeMetricsEl  = document.getElementById('edge-metrics');
    var edgeImprovable = document.getElementById('edge-improvable');
    var edgeNote       = document.getElementById('edge-note');
    var saveNoteBtn    = document.getElementById('save-note');
    var downloadBtn    = document.getElementById('download-btn');
    var statusBar      = document.getElementById('status-bar');

    // ── Build fixture buttons ─────────────────────────────────────────────────
    if (fixtures.length === 0) {
      fixtureListEl.innerHTML = '<p class="empty-state">No fixtures found.</p>';
    }

    fixtures.forEach(function (fixture, i) {
      var btn = document.createElement('button');
      btn.className = 'fixture-btn';

      var nameSpan = document.createElement('span');
      nameSpan.textContent = fixture.name;
      btn.appendChild(nameSpan);

      var flagged = countFlagged(fixture.name);
      if (flagged > 0) {
        var badge = document.createElement('span');
        badge.className = 'badge';
        badge.textContent = flagged + ' flagged';
        btn.appendChild(badge);
      }

      btn.addEventListener('click', function () { selectFixture(i); });
      fixtureListEl.appendChild(btn);
    });

    // ── Select a fixture ──────────────────────────────────────────────────────
    function selectFixture(idx) {
      currentFixtureIdx = idx;
      currentEdgeId = null;
      edgePanel.hidden = true;

      document.querySelectorAll('.fixture-btn').forEach(function (btn, j) {
        btn.classList.toggle('active', j === idx);
      });

      var fixture = fixtures[idx];

      // Header
      fixtureHeader.hidden = false;
      fixtureTitleEl.textContent = fixture.name;

      var q = fixture.global_metrics.avg_quality;
      qualityBadge.textContent = 'Quality: ' + (q * 100).toFixed(0) + '%';
      qualityBadge.className =
        'quality-badge ' +
        (q >= 0.8 ? 'quality-good' : q >= 0.5 ? 'quality-ok' : 'quality-bad');

      flaggedCount.textContent =
        fixture.global_metrics.routed_edges + ' edges  \u00B7  ' +
        fixture.global_metrics.flagged_edges + ' algorithm-flagged';

      // SVG
      svgContainer.innerHTML = fixture.svg;

      // Wire up click handlers on diagnostic overlay paths
      svgContainer.querySelectorAll('[data-edge-id]').forEach(function (el) {
        el.style.cursor = 'pointer';
        el.addEventListener('click', function (e) {
          e.stopPropagation();
          selectEdge(fixture.name, el.dataset.edgeId);
        });
      });

      updateEdgeHighlights(fixture.name);
    }

    // ── Select an edge ────────────────────────────────────────────────────────
    function selectEdge(fixtureName, edgeId) {
      currentEdgeId = edgeId;

      var fixture = fixtures[currentFixtureIdx];
      var edgeData = null;
      for (var i = 0; i < fixture.edges.length; i++) {
        if (fixture.edges[i].id === edgeId) { edgeData = fixture.edges[i]; break; }
      }
      if (!edgeData) return;

      edgePanel.hidden = false;
      edgeTitleEl.textContent = edgeId;

      var flagsHtml = edgeData.flags.length > 0
        ? '<div class="metric flags"><span>Flags:</span> <b>' +
          escHtml(edgeData.flags.join(', ')) + '</b></div>'
        : '';

      edgeMetricsEl.innerHTML =
        '<div class="metric"><span>Quality score:</span> <b>' +
          (edgeData.quality_score * 100).toFixed(0) + '%</b></div>' +
        '<div class="metric"><span>Detour factor:</span> <b>' +
          edgeData.detour_factor.toFixed(2) + 'x</b></div>' +
        '<div class="metric"><span>Bends:</span> <b>' + edgeData.bends + '</b></div>' +
        '<div class="metric"><span>Crossings:</span> <b>' + edgeData.crossings + '</b></div>' +
        '<div class="metric"><span>Source port:</span> <b>' +
          escHtml(edgeData.port_side_source) + '</b></div>' +
        '<div class="metric"><span>Target port:</span> <b>' +
          escHtml(edgeData.port_side_target) + '</b></div>' +
        flagsHtml;

      // Load any saved note
      var saved = getEdgeFeedback(fixtureName, edgeId);
      edgeImprovable.checked = saved.improvable || false;
      edgeNote.value = saved.note || '';

      // Highlight selected edge path
      svgContainer.querySelectorAll('[data-edge-id]').forEach(function (el) {
        el.classList.toggle('selected-edge', el.dataset.edgeId === edgeId);
      });
    }

    // ── Save note ─────────────────────────────────────────────────────────────
    saveNoteBtn.addEventListener('click', function () {
      if (currentFixtureIdx < 0 || !currentEdgeId) return;
      var fixtureName = fixtures[currentFixtureIdx].name;

      var note = edgeNote.value.trim();
      var improvable = edgeImprovable.checked;

      if (!feedback[fixtureName]) feedback[fixtureName] = { edges: {} };
      if (!feedback[fixtureName].edges) feedback[fixtureName].edges = {};

      if (improvable || note) {
        feedback[fixtureName].edges[currentEdgeId] = {
          improvable: improvable,
          note: note,
        };
      } else {
        delete feedback[fixtureName].edges[currentEdgeId];
        if (Object.keys(feedback[fixtureName].edges).length === 0) {
          delete feedback[fixtureName];
        }
      }

      saveFeedback();
      updateEdgeHighlights(fixtureName);
      refreshFixtureButton(currentFixtureIdx, fixtureName);
      showStatus('Saved.');
    });

    // ── Download feedback.json ────────────────────────────────────────────────
    downloadBtn.addEventListener('click', function () {
      var json = JSON.stringify(feedback, null, 2);
      var blob = new Blob([json], { type: 'application/json' });
      var url = URL.createObjectURL(blob);
      var a = document.createElement('a');
      a.href = url;
      a.download = 'feedback.json';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      showStatus('Downloaded.');
    });

    // ── Helpers ───────────────────────────────────────────────────────────────
    function saveFeedback() {
      try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(feedback));
      } catch (e) {
        showStatus('Save failed: ' + e.message);
      }
    }

    function getEdgeFeedback(fixtureName, edgeId) {
      return ((feedback[fixtureName] || {}).edges || {})[edgeId] || {};
    }

    function countFlagged(fixtureName) {
      var edges = ((feedback[fixtureName] || {}).edges) || {};
      var count = 0;
      Object.keys(edges).forEach(function (k) {
        if (edges[k].improvable) count++;
      });
      return count;
    }

    function updateEdgeHighlights(fixtureName) {
      var edges = ((feedback[fixtureName] || {}).edges) || {};
      svgContainer.querySelectorAll('[data-edge-id]').forEach(function (el) {
        var fb = edges[el.dataset.edgeId];
        el.classList.toggle('edge-flagged', !!(fb && fb.improvable));
      });
    }

    function refreshFixtureButton(idx, fixtureName) {
      var btns = document.querySelectorAll('.fixture-btn');
      var btn = btns[idx];
      if (!btn) return;

      var flagged = countFlagged(fixtureName);
      var badge = btn.querySelector('.badge');
      if (flagged > 0) {
        if (!badge) {
          badge = document.createElement('span');
          badge.className = 'badge';
          btn.appendChild(badge);
        }
        badge.textContent = flagged + ' flagged';
      } else if (badge) {
        badge.remove();
      }
    }

    function showStatus(msg) {
      statusBar.textContent = msg;
      setTimeout(function () { statusBar.textContent = ''; }, 2000);
    }

    function escHtml(s) {
      return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
    }

    // ── Auto-select first fixture ─────────────────────────────────────────────
    if (fixtures.length > 0) selectFixture(0);

  }());
  </script>
</body>
</html>
"#;

// --- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{DiagramReport, EdgeReport, GlobalMetrics};

    fn make_report(fixture: &str) -> DiagramReport {
        DiagramReport {
            fixture: fixture.to_string(),
            edges: vec![EdgeReport {
                id: "A-->B".to_string(),
                source: "A".to_string(),
                target: "B".to_string(),
                bends: 2,
                detour_factor: 1.5,
                crossings: 0,
                port_side_source: "South".to_string(),
                port_side_target: "North".to_string(),
                path_cells: vec![[0, 0], [1, 0]],
                quality_score: 0.75,
                flags: vec!["high_detour".to_string()],
            }],
            global_metrics: GlobalMetrics {
                total_edges: 1,
                routed_edges: 1,
                failed_edges: 0,
                total_crossings: 0,
                total_bends: 2,
                avg_quality_score: 0.75,
                avg_detour_factor: 1.5,
                flagged_edges: 1,
            },
        }
    }

    #[test]
    fn html_is_valid_utf8_and_contains_structure() {
        let report = make_report("b05.mmd");
        let entries = [ReviewEntry {
            fixture_name: "b05.mmd",
            annotated_svg: "<svg xmlns='http://www.w3.org/2000/svg'><g data-edge-id='A--&gt;B'/></svg>",
            report: &report,
        }];
        let html_bytes = generate_review_html(&entries);
        let html = std::str::from_utf8(&html_bytes).expect("output is not valid UTF-8");

        assert!(html.contains("<!DOCTYPE html>"), "missing doctype");
        assert!(html.contains("Trellis Edge Review"), "missing page title");
        assert!(html.contains("fixtures-data"), "missing data script tag");
        assert!(html.contains("feedback.json"), "missing download reference");
        assert!(html.contains("localStorage"), "missing localStorage usage");
    }

    #[test]
    fn fixtures_json_contains_fixture_name_and_edge() {
        let report = make_report("test.mmd");
        let entries = [ReviewEntry {
            fixture_name: "test.mmd",
            annotated_svg: "<svg/>",
            report: &report,
        }];
        let html = String::from_utf8(generate_review_html(&entries)).unwrap();

        assert!(html.contains("test.mmd"), "fixture name missing from JSON");
        assert!(html.contains("A-->B"), "edge id missing from JSON");
        assert!(html.contains("high_detour"), "flags missing from JSON");
    }

    #[test]
    fn empty_entries_renders_without_panic() {
        let html_bytes = generate_review_html(&[]);
        let html = std::str::from_utf8(&html_bytes).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        // fixtures-data block should contain an empty array
        assert!(html.contains("[]"));
    }

    #[test]
    fn multiple_fixtures_all_appear_in_output() {
        let r1 = make_report("b01.mmd");
        let r2 = make_report("b02.mmd");
        let entries = [
            ReviewEntry { fixture_name: "b01.mmd", annotated_svg: "<svg/>", report: &r1 },
            ReviewEntry { fixture_name: "b02.mmd", annotated_svg: "<svg/>", report: &r2 },
        ];
        let html = String::from_utf8(generate_review_html(&entries)).unwrap();

        assert!(html.contains("b01.mmd"));
        assert!(html.contains("b02.mmd"));
    }
}
