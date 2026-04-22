// Editor.jsx — Split-pane Mermaid editor
const SAMPLE_DIAGRAMS = {
  flowchart: `flowchart LR
  Client([Client App])
  API[API Gateway]
  Auth[Auth Service]
  DB[(Database)]
  Cache[(Redis Cache)]
  Queue[[Job Queue]]
  Worker[Worker Service]
  Storage[/Object Storage/]

  Client --> API
  API --> Auth
  Auth --> DB
  API --> Cache
  API --> Queue
  Queue --> Worker
  Worker --> DB
  Worker --> Storage`,

  classDiagram: `classDiagram
  class DiagramRenderer {
    +String input
    +TrellisConfig config
    +render() SVG
    +renderWithMetrics() JSON
  }
  class TrellisConfig {
    +String direction
    +String theme
    +Int cellSize
  }
  class Graph {
    +Node[] nodes
    +Edge[] edges
    +Subgraph[] subgraphs
  }
  class Node {
    +String id
    +String label
    +String shape
    +Int width
    +Int height
  }
  DiagramRenderer --> TrellisConfig
  DiagramRenderer --> Graph
  Graph --> Node`,

  erDiagram: `erDiagram
  USER {
    int id PK
    string email
    string name
    timestamp created_at
  }
  PROJECT {
    int id PK
    string title
    string status
    int owner_id FK
  }
  DIAGRAM {
    int id PK
    string source
    string svg_output
    int project_id FK
    float quality_score
  }
  USER ||--o{ PROJECT : owns
  PROJECT ||--o{ DIAGRAM : contains`,
};

const METRICS_MAP = {
  flowchart: { nodes: 8, edges: 8, quality: '0.94', time: '4.1ms', crossings: 0 },
  classDiagram: { nodes: 4, edges: 3, quality: '0.91', time: '2.8ms', crossings: 0 },
  erDiagram: { nodes: 3, edges: 3, quality: '0.96', time: '3.3ms', crossings: 0 },
};

const EditorSection = ({ tweaks }) => {
  const { useState, useEffect, useRef, useCallback } = React;
  const [diagramType, setDiagramType] = useState(() => localStorage.getItem('trellis-dtype') || 'flowchart');
  const [code, setCode] = useState(() => localStorage.getItem('trellis-code') || SAMPLE_DIAGRAMS.flowchart);
  const [renderKey, setRenderKey] = useState(0);
  const [error, setError] = useState(null);
  const [copied, setCopied] = useState(false);
  const previewRef = useRef(null);
  const debounceRef = useRef(null);

  const layoutCols = tweaks?.layout === '40-60' ? '40% 1fr' : tweaks?.layout === '60-40' ? '60% 1fr' : '1fr 1fr';
  const showGrid = tweaks?.showGrid !== false;
  const showMetrics = tweaks?.showMetrics !== false;
  const metrics = METRICS_MAP[diagramType] || METRICS_MAP.flowchart;

  const renderDiagram = useCallback(async (src) => {
    if (!previewRef.current) return;
    try {
      mermaid.initialize({ startOnLoad: false, theme: tweaks?.theme || 'default', securityLevel: 'loose' });
      const id = 'diagram-' + Date.now();
      const { svg } = await mermaid.render(id, src);
      if (previewRef.current) {
        previewRef.current.innerHTML = svg;
        // Style the SVG for blueprint context
        const svgEl = previewRef.current.querySelector('svg');
        if (svgEl) {
          svgEl.style.maxWidth = '100%';
          svgEl.style.height = 'auto';
          svgEl.style.display = 'block';
          svgEl.style.margin = '0 auto';
        }
        setError(null);
      }
    } catch (err) {
      if (previewRef.current) {
        previewRef.current.innerHTML = '';
      }
      setError(err.message || 'Parse error');
    }
  }, [tweaks?.theme]);

  useEffect(() => {
    clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => renderDiagram(code), 400);
    return () => clearTimeout(debounceRef.current);
  }, [code, renderDiagram]);

  const switchType = (type) => {
    setDiagramType(type);
    const next = SAMPLE_DIAGRAMS[type];
    setCode(next);
    localStorage.setItem('trellis-dtype', type);
    localStorage.setItem('trellis-code', next);
  };

  const handleCode = (e) => {
    const val = e.target.value;
    setCode(val);
    localStorage.setItem('trellis-code', val);
  };

  const copyCode = () => {
    navigator.clipboard.writeText(code).then(() => { setCopied(true); setTimeout(() => setCopied(false), 1500); });
  };

  const downloadSVG = () => {
    const svg = previewRef.current?.querySelector('svg');
    if (!svg) return;
    const blob = new Blob([svg.outerHTML], { type: 'image/svg+xml' });
    const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = 'diagram.svg'; a.click();
  };

  return (
    <section id="editor" style={{
      minHeight: '100vh', display: 'flex', flexDirection: 'column',
      background: 'var(--prussian)',
      backgroundImage: showGrid ? 'var(--grid)' : 'none',
      backgroundSize: '32px 32px',
    }}>
      {/* Hero blurb */}
      <div style={{ textAlign: 'center', padding: '48px 24px 32px' }}>
        <div style={{ display: 'inline-block', padding: '4px 14px', borderRadius: 999, border: '1px solid rgba(74,138,181,0.35)', background: 'rgba(74,138,181,0.08)', marginBottom: 16 }}>
          <span style={{ fontFamily: 'var(--font-m)', fontSize: 11, letterSpacing: '0.15em', textTransform: 'uppercase', color: '#4A8AB5' }}>Now in beta</span>
        </div>
        <h1 style={{ fontFamily: "'Courier Prime', monospace", fontSize: 'clamp(2rem,5vw,3.25rem)', fontWeight: 700, letterSpacing: '-0.02em', color: '#E8F4FB', lineHeight: 1.15 }}>
          Diagrams that <span style={{ color: '#4A8AB5' }}>actually route</span>.
        </h1>
        <p style={{ marginTop: 14, color: 'rgba(232,244,251,0.6)', fontSize: 15, maxWidth: 520, margin: '14px auto 0', lineHeight: 1.65 }}>
          Trellis uses VLSI maze routing to render overlap-free Mermaid diagrams. Drop-in compatible. Zero JavaScript required.
        </p>
      </div>

      {/* Diagram type tabs */}
      <div style={{ display: 'flex', justifyContent: 'center', gap: 6, padding: '0 24px 16px' }}>
        {[['flowchart','Flowchart'], ['classDiagram','Class'], ['erDiagram','ER Diagram']].map(([type, label]) => (
          <button key={type} onClick={() => switchType(type)} style={{
            padding: '6px 16px', borderRadius: 6, fontSize: 12, fontWeight: 500, cursor: 'pointer',
            border: '1px solid ' + (diagramType === type ? '#4A8AB5' : 'rgba(74,138,181,0.25)'),
            background: diagramType === type ? 'rgba(74,138,181,0.18)' : 'transparent',
            color: diagramType === type ? '#E8F4FB' : 'rgba(232,244,251,0.5)',
            transition: 'all 150ms',
          }}>{label}</button>
        ))}
      </div>

      {/* Split pane */}
      <div style={{
        flex: 1, display: 'grid', gridTemplateColumns: layoutCols,
        gap: 1, background: 'rgba(74,138,181,0.15)',
        margin: '0 24px', borderRadius: 10, overflow: 'hidden',
        border: '1px solid rgba(74,138,181,0.2)',
        minHeight: 480,
      }}>
        {/* Code pane */}
        <div style={{ display: 'flex', flexDirection: 'column', background: 'var(--code-bg)', overflow: 'hidden' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 14px', borderBottom: '1px solid rgba(74,138,181,0.15)' }}>
            <span style={{ fontFamily: 'var(--font-m)', fontSize: 11, color: 'rgba(232,244,251,0.4)', textTransform: 'uppercase', letterSpacing: '0.1em' }}>Mermaid source</span>
            <button onClick={copyCode} style={{
              fontFamily: 'var(--font-m)', fontSize: 11, padding: '3px 10px', borderRadius: 4,
              background: 'transparent', border: '1px solid rgba(74,138,181,0.25)',
              color: copied ? '#3fb950' : 'rgba(232,244,251,0.5)', cursor: 'pointer',
            }}>{copied ? '✓ Copied' : 'Copy'}</button>
          </div>
          <textarea
            value={code}
            onChange={handleCode}
            spellCheck={false}
            style={{
              flex: 1, background: 'transparent', border: 'none', outline: 'none', resize: 'none',
              fontFamily: "'JetBrains Mono', monospace", fontSize: 13, lineHeight: 1.75,
              color: '#E8F4FB', padding: '16px', tabSize: 2,
            }}
          />
          {error && (
            <div style={{ padding: '8px 14px', background: 'rgba(248,81,73,0.1)', borderTop: '1px solid rgba(248,81,73,0.25)', fontSize: 11, color: '#f85149', fontFamily: 'var(--font-m)' }}>
              ✗ {error.split('\n')[0].slice(0, 80)}
            </div>
          )}
        </div>

        {/* Preview pane */}
        <div style={{ display: 'flex', flexDirection: 'column', background: '#0d1e30', overflow: 'hidden' }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '8px 14px', borderBottom: '1px solid rgba(74,138,181,0.15)' }}>
            <span style={{ fontFamily: 'var(--font-m)', fontSize: 11, color: 'rgba(232,244,251,0.4)', textTransform: 'uppercase', letterSpacing: '0.1em' }}>Preview</span>
            <button onClick={downloadSVG} style={{
              fontFamily: 'var(--font-m)', fontSize: 11, padding: '3px 10px', borderRadius: 4,
              background: 'transparent', border: '1px solid rgba(74,138,181,0.25)',
              color: 'rgba(232,244,251,0.5)', cursor: 'pointer',
            }}>Export SVG</button>
          </div>
          <div ref={previewRef} style={{
            flex: 1, padding: 24, overflow: 'auto', display: 'flex', alignItems: 'center', justifyContent: 'center',
          }} />
        </div>
      </div>

      {/* Metrics bar */}
      {showMetrics && (
        <div style={{ display: 'flex', justifyContent: 'center', gap: 6, padding: '12px 24px 32px', flexWrap: 'wrap' }}>
          {[
            ['NODES', metrics.nodes],
            ['EDGES', metrics.edges],
            ['QUALITY', metrics.quality],
            ['RENDER', metrics.time],
            ['CROSSINGS', metrics.crossings],
          ].map(([k, v]) => (
            <div key={k} style={{ background: 'rgba(27,63,110,0.6)', border: '1px solid rgba(74,138,181,0.2)', borderRadius: 6, padding: '6px 16px', textAlign: 'center' }}>
              <div style={{ fontFamily: 'var(--font-m)', fontSize: 9, color: 'rgba(232,244,251,0.4)', letterSpacing: '0.15em', textTransform: 'uppercase' }}>{k}</div>
              <div style={{ fontFamily: 'var(--font-m)', fontSize: 16, fontWeight: 700, color: k === 'QUALITY' ? '#3fb950' : '#4A8AB5', marginTop: 2 }}>{v}</div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
};
Object.assign(window, { EditorSection });
