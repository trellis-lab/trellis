// Header.jsx — Trellis top navigation
const TrellisHeader = () => {
  const [scrolled, setScrolled] = React.useState(false);
  React.useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    window.addEventListener('scroll', onScroll);
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  return (
    <header style={{
      position: 'sticky', top: 0, zIndex: 100,
      background: scrolled ? 'rgba(0,49,83,0.96)' : 'transparent',
      backdropFilter: scrolled ? 'blur(12px)' : 'none',
      borderBottom: scrolled ? '1px solid rgba(74,138,181,0.2)' : '1px solid transparent',
      transition: 'all 200ms ease',
      padding: '0 32px',
    }}>
      <div style={{ maxWidth: 1200, margin: '0 auto', height: 56, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        {/* Logo */}
        <a href="#" style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <img src="../../assets/trellis-logo-narrow.svg" height={28} alt="Trellis" style={{ filter: 'invert(1) brightness(2)' }} />
          <span style={{ fontFamily: "'Courier Prime', monospace", fontWeight: 700, fontSize: 18, color: '#E8F4FB', letterSpacing: '-0.01em' }}>Trellis</span>
        </a>

        {/* Nav links */}
        <nav style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
          {[
            { label: 'Editor', href: '#editor' },
            { label: 'Docs', href: '#' },
            { label: 'GitHub', href: 'https://github.com/trellis-mermaid/trellis' },
          ].map(item => (
            <a key={item.label} href={item.href} style={{
              padding: '6px 12px', borderRadius: 6, fontSize: 13, fontWeight: 500,
              color: 'rgba(232,244,251,0.75)',
              transition: 'color 150ms, background 150ms',
            }}
            onMouseEnter={e => { e.target.style.color = '#E8F4FB'; e.target.style.background = 'rgba(74,138,181,0.1)'; }}
            onMouseLeave={e => { e.target.style.color = 'rgba(232,244,251,0.75)'; e.target.style.background = 'transparent'; }}>
              {item.label}
            </a>
          ))}
          <a href="https://github.com/trellis-mermaid/trellis/releases" style={{
            marginLeft: 8, padding: '6px 14px', borderRadius: 6, fontSize: 13, fontWeight: 600,
            background: '#4A8AB5', color: '#003153', border: 'none', cursor: 'pointer',
            transition: 'background 150ms',
          }}
          onMouseEnter={e => e.target.style.background = '#5a9bc5'}
          onMouseLeave={e => e.target.style.background = '#4A8AB5'}>
            Download
          </a>
        </nav>
      </div>
    </header>
  );
};
Object.assign(window, { TrellisHeader });
