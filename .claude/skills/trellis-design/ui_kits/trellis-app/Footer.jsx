// Footer.jsx — Trellis site footer
const TrellisFooter = () => (
  <footer style={{
    background: '#00203d',
    borderTop: '1px solid rgba(74,138,181,0.15)',
    padding: '40px 32px',
  }}>
    <div style={{ maxWidth: 1200, margin: '0 auto', display: 'grid', gridTemplateColumns: '1fr auto', alignItems: 'start', gap: 32 }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 10 }}>
          <img src="../../assets/trellis-logo-narrow.svg" height={20} alt="Trellis" style={{ filter: 'invert(1) brightness(1.8)' }} />
          <span style={{ fontFamily: "'Courier Prime',monospace", fontWeight: 700, fontSize: 15, color: '#E8F4FB' }}>Trellis</span>
        </div>
        <p style={{ fontSize: 12, color: 'rgba(232,244,251,0.4)', lineHeight: 1.65, maxWidth: 380 }}>
          A fast, overlap-free Mermaid-compatible diagram renderer built in Rust.
          VLSI maze routing. Orthogonal edges. Zero JavaScript required.
        </p>
        <p style={{ marginTop: 16, fontSize: 11, color: 'rgba(232,244,251,0.25)', fontFamily: "'JetBrains Mono',monospace" }}>
          © 2026 Trellis · MIT OR Apache-2.0 · <a href="https://trellislabs.net" style={{ color: '#4A8AB5' }}>trellislabs.net</a>
        </p>
      </div>
      <div style={{ display: 'flex', gap: 32 }}>
        {[
          { heading: 'Product', links: ['Editor', 'CLI', 'WASM', 'VS Code', 'IntelliJ'] },
          { heading: 'Resources', links: ['GitHub', 'Docs', 'Releases', 'Changelog'] },
        ].map(col => (
          <div key={col.heading}>
            <div style={{ fontSize: 10, textTransform: 'uppercase', letterSpacing: '0.12em', color: 'rgba(232,244,251,0.3)', marginBottom: 10, fontFamily: "'JetBrains Mono',monospace" }}>{col.heading}</div>
            {col.links.map(l => (
              <div key={l} style={{ marginBottom: 6 }}>
                <a href="#" style={{ fontSize: 12, color: 'rgba(232,244,251,0.45)', transition: 'color 150ms' }}
                  onMouseEnter={e => e.target.style.color = '#E8F4FB'}
                  onMouseLeave={e => e.target.style.color = 'rgba(232,244,251,0.45)'}>{l}</a>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  </footer>
);
Object.assign(window, { TrellisFooter });
