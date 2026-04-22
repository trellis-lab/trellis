// Landing.jsx — Marketing sections
const LandingSection = () => {
  const features = [
    {
      icon: '⬡',
      title: 'VLSI Maze Routing',
      desc: 'A* pathfinding borrowed from chip design. Edges route on an orthogonal grid — overlap-free by construction, not by post-processing.',
    },
    {
      icon: '→',
      title: 'Orthogonal Edges',
      desc: 'Every edge is routed with right-angle turns. No diagonal spaghetti. Rip-up & reroute resolves deadlocks automatically.',
    },
    {
      icon: '◈',
      title: 'Four Diagram Types',
      desc: 'Flowchart, Class, ER, and C4 diagrams supported. Each type uses a specialized layout algorithm — Sugiyama, hybrid, or force-directed.',
    },
    {
      icon: '▶',
      title: 'WASM-First',
      desc: 'The renderer compiles to a tiny WebAssembly module. Drop it in any browser or Node.js project. No server-side rendering needed.',
    },
    {
      icon: '≋',
      title: 'Editor Extensions',
      desc: 'Live preview in VS Code and IntelliJ. Re-renders on every keystroke. Ships as a standard .vsix or .zip plugin.',
    },
    {
      icon: '⊞',
      title: 'Quality Scoring',
      desc: 'Every rendered diagram ships with a per-edge quality report. Detour factor, crossing count, bend count — all machine-readable JSON.',
    },
  ];

  const steps = [
    { step: '01', label: 'Write Mermaid', code: 'flowchart LR\n  A --> B --> C' },
    { step: '02', label: 'Call render()', code: 'const svg = render(src, null);\ndiv.innerHTML = svg;' },
    { step: '03', label: 'Get clean SVG', code: '→ 3 nodes, 2 edges\n→ Quality: 0.98 ✓' },
  ];

  const useCases = [
    { label: 'AI-generated diagrams', desc: 'LLMs emit Mermaid natively. Trellis makes the output presentable — no manual cleanup.' },
    { label: 'Architecture documentation', desc: 'System diagrams that stay in version control and look professional in pull requests.' },
    { label: 'Client deliverables', desc: 'Export production-quality SVGs from your Markdown without reaching for draw.io.' },
    { label: 'Pandoc pipelines', desc: 'Drop the Lua filter into your document build. Mermaid blocks render inline, automatically.' },
  ];

  const S = {
    section: { padding: '96px 24px', maxWidth: 1200, margin: '0 auto' },
    label: { fontFamily: "'JetBrains Mono', monospace", fontSize: 11, letterSpacing: '0.15em', textTransform: 'uppercase', color: '#4A8AB5', marginBottom: 12, display: 'block' },
    h2: { fontFamily: "'Courier Prime', monospace", fontSize: 'clamp(1.75rem, 4vw, 2.75rem)', fontWeight: 700, letterSpacing: '-0.02em', color: '#E8F4FB', lineHeight: 1.15 },
    p: { color: 'rgba(232,244,251,0.6)', fontSize: 15, lineHeight: 1.7, marginTop: 12 },
  };

  return (
    <>
      {/* Features grid */}
      <section style={{ background: 'var(--blueprint)', borderTop: '1px solid rgba(74,138,181,0.2)', borderBottom: '1px solid rgba(74,138,181,0.2)' }}>
        <div style={S.section}>
          <div style={{ textAlign: 'center', marginBottom: 56 }}>
            <span style={S.label}>Why Trellis</span>
            <h2 style={S.h2}>Routing that <span style={{ color: '#4A8AB5' }}>actually works.</span></h2>
            <p style={{ ...S.p, maxWidth: 540, margin: '12px auto 0' }}>Dagre hasn't had a meaningful update since 2018. Trellis is built from first principles using algorithms from VLSI chip design.</p>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: 16 }}>
            {features.map((f, i) => (
              <div key={i} style={{
                background: 'rgba(0,49,83,0.45)', border: '1px solid rgba(74,138,181,0.2)',
                borderRadius: 8, padding: '20px 22px',
                transition: 'border-color 150ms, background 150ms',
              }}
              onMouseEnter={e => { e.currentTarget.style.borderColor = '#4A8AB5'; e.currentTarget.style.background = 'rgba(0,49,83,0.7)'; }}
              onMouseLeave={e => { e.currentTarget.style.borderColor = 'rgba(74,138,181,0.2)'; e.currentTarget.style.background = 'rgba(0,49,83,0.45)'; }}>
                <div style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 20, color: '#4A8AB5', marginBottom: 10 }}>{f.icon}</div>
                <div style={{ fontSize: 14, fontWeight: 600, color: '#E8F4FB', marginBottom: 6 }}>{f.title}</div>
                <div style={{ fontSize: 13, color: 'rgba(232,244,251,0.55)', lineHeight: 1.65 }}>{f.desc}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* How it works */}
      <section style={{ backgroundImage: 'var(--grid)', backgroundSize: '32px 32px', borderBottom: '1px solid rgba(74,138,181,0.15)' }}>
        <div style={S.section}>
          <div style={{ textAlign: 'center', marginBottom: 48 }}>
            <span style={S.label}>How it works</span>
            <h2 style={S.h2}>Three lines to clean diagrams.</h2>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: 16 }}>
            {steps.map((s, i) => (
              <div key={i} style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                  <span style={{ fontFamily: "'JetBrains Mono',monospace", fontSize: 11, color: '#4A8AB5', letterSpacing: '0.1em' }}>{s.step}</span>
                  <div style={{ flex: 1, height: 1, background: 'rgba(74,138,181,0.2)' }} />
                </div>
                <div style={{ fontSize: 15, fontWeight: 600, color: '#E8F4FB' }}>{s.label}</div>
                <pre style={{
                  background: '#071523', border: '1px solid rgba(74,138,181,0.2)', borderRadius: 6,
                  padding: '12px 14px', fontFamily: "'JetBrains Mono',monospace", fontSize: 12,
                  color: '#7dd3fc', lineHeight: 1.7, whiteSpace: 'pre-wrap',
                }}>{s.code}</pre>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Use cases */}
      <section style={{ background: 'var(--blueprint)', borderBottom: '1px solid rgba(74,138,181,0.2)' }}>
        <div style={{ ...S.section, display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 48, alignItems: 'center' }}>
          <div>
            <span style={S.label}>Use cases</span>
            <h2 style={S.h2}>Built for the places where Dagre breaks.</h2>
            <p style={S.p}>Real software architectures are rarely planar graphs. By Kuratowski's theorem, crossings are mathematically inevitable with straight edges — Trellis embraces that with orthogonal routing instead of hiding it.</p>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            {useCases.map((u, i) => (
              <div key={i} style={{ background: 'rgba(0,49,83,0.45)', border: '1px solid rgba(74,138,181,0.18)', borderRadius: 8, padding: '14px 18px' }}>
                <div style={{ fontSize: 13, fontWeight: 600, color: '#E8F4FB', marginBottom: 3 }}>{u.label}</div>
                <div style={{ fontSize: 12, color: 'rgba(232,244,251,0.5)', lineHeight: 1.6 }}>{u.desc}</div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Install / CTA */}
      <section style={{ backgroundImage: 'var(--grid)', backgroundSize: '32px 32px', borderBottom: '1px solid rgba(74,138,181,0.15)' }}>
        <div style={{ ...S.section, textAlign: 'center' }}>
          <span style={S.label}>Get started</span>
          <h2 style={S.h2}>Zero dependencies.<br />Drop-in Mermaid syntax.</h2>
          <p style={{ ...S.p, maxWidth: 480, margin: '12px auto 24px' }}>Pre-built binaries for Linux, macOS, and Windows. WASM package for the browser. Docker image for CI pipelines.</p>
          <div style={{ display: 'flex', gap: 10, justifyContent: 'center', flexWrap: 'wrap' }}>
            {[
              ['Download CLI', '#4A8AB5', '#003153'],
              ['npm install trellis-wasm', 'transparent', '#4A8AB5'],
              ['docker pull ghcr.io/trellis-mermaid/trellis', 'transparent', 'rgba(232,244,251,0.5)'],
            ].map(([label, bg, color], i) => (
              <code key={i} style={{
                display: 'inline-block', padding: i === 0 ? '9px 22px' : '9px 16px',
                background: bg, color, border: `1px solid ${i === 0 ? 'transparent' : 'rgba(74,138,181,0.3)'}`,
                borderRadius: 6, fontFamily: i === 0 ? 'var(--font-b)' : "'JetBrains Mono',monospace",
                fontWeight: i === 0 ? 600 : 400, fontSize: i === 0 ? 14 : 12, cursor: 'pointer',
              }}>{label}</code>
            ))}
          </div>
        </div>
      </section>
    </>
  );
};
Object.assign(window, { LandingSection });
