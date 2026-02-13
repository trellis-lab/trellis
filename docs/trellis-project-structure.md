# Trellis – Projekt szerkezet

## Monorepo struktúra (Rust/WASM + IDE pluginok)

**Dátum:** 2026-02-08
**Kapcsolódó dokumentum:** trellis-spec.md (v0.3)

---

## Tartalomjegyzék

1. [Áttekintés](#1-áttekintés)
2. [Könyvtárstruktúra](#2-könyvtárstruktúra)
3. [Core crate-ek](#3-core-crate-ek)
4. [WASM build](#4-wasm-build)
5. [CLI alkalmazás](#5-cli-alkalmazás)
6. [VS Code extension](#6-vs-code-extension)
7. [IntelliJ plugin](#7-intellij-plugin)
8. [Docker](#8-docker)
9. [Build pipeline](#9-build-pipeline)
10. [Fejlesztői workflow](#10-fejlesztői-workflow)

---

## 1. Áttekintés

```
┌──────────────────────────────────────────────────────────┐
│                    trellis (monorepo)                    │
│                                                           │
│  ┌────────────────────────────────────┐                   │
│  │         crates/ (Rust)             │                   │
│  │                                    │                   │
│  │  trellis-core     algoritmus     │                   │
│  │  trellis-parser   Mermaid parse  │                   │
│  │  trellis-wasm     WASM binding   │                   │
│  │  trellis-cli      CLI bináris    │                   │
│  └──────────┬────────────┬────────────┘                   │
│             │            │                                │
│        ┌────▼────┐  ┌────▼────┐                           │
│        │  WASM   │  │  CLI    │                           │
│        │  .wasm  │  │  bin    │                           │
│        └────┬────┘  └────┬────┘                           │
│    ┌────────┼────┐       │                                │
│ ┌──▼─────┐ ┌──▼──────┐  │                                │
│ │editors/│ │editors/  │  │                                │
│ │vscode/ │ │intellij/ │  │                                │
│ │(TS)    │ │(Kotlin)  │  │                                │
│ └────────┘ └──────────┘  │                                │
│                     ┌────▼─────┐                          │
│                     │ Docker / │                          │
│                     │ CI/CD    │                          │
│                     └──────────┘                          │
└──────────────────────────────────────────────────────────┘
```

---

## 2. Könyvtárstruktúra

```
trellis/
├── Cargo.toml                    # Workspace root
├── LICENSE
├── README.md
│
├── crates/
│   ├── trellis-parser/         # Mermaid parser (Fázis 1)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tokenizer.rs      # Lexer
│   │       ├── flowchart.rs      # Flowchart szintaxis
│   │       ├── class_diagram.rs  # Class diagram szintaxis
│   │       ├── er_diagram.rs     # ER diagram szintaxis
│   │       └── ast.rs            # Közös AST struktúrák (Graph, Node, Edge, Subgraph)
│   │
│   ├── trellis-core/           # Algoritmus (Fázis 2-9)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs          # Közös típusok (Grid, Cell, Port, Path, BoundingBox)
│   │       ├── config.rs         # TrellisConfig (dekompozíció, küszöbök)
│   │       │
│   │       ├── placement/        # Fázis 2 – Csomópont-elhelyezés
│   │       │   ├── mod.rs
│   │       │   ├── sugiyama.rs   # Flowchart (rétegezés, barycenter)
│   │       │   ├── class.rs      # Class diagram (hibrid)
│   │       │   ├── er.rs         # ER diagram (force-directed)
│   │       │   ├── subgraph.rs   # Subgraph fa + rekurzív elhelyezés
│   │       │   └── snap.rs       # Grid-re kerekítés
│   │       │
│   │       ├── grid/             # Fázis 3 – Grid felépítés
│   │       │   ├── mod.rs
│   │       │   ├── params.rs     # Cellaméret, kiterjedés számítás
│   │       │   └── builder.rs    # Grid konstrukció, blokkolás
│   │       │
│   │       ├── ports/            # Fázis 4 – Port-kiosztás
│   │       │   ├── mod.rs
│   │       │   ├── assignment.rs # Oldal-hozzárendelés, szögszámítás
│   │       │   ├── overflow.rs   # Túlcsordulás kezelés
│   │       │   └── positions.rs  # Port pozíciók kiszámítása
│   │       │
│   │       ├── routing/          # Fázis 5-6 – Prioritás + A*
│   │       │   ├── mod.rs
│   │       │   ├── priority.rs   # Routing sorrend (hibrid pontozás)
│   │       │   ├── astar.rs      # A* pathfinding
│   │       │   ├── cost.rs       # Költségfüggvény (BASE, BEND, ADJACENT, CROSSING)
│   │       │   ├── multi_edge.rs # Többszörös élek (detektálás + routing)
│   │       │   └── commit.rs     # Út lefoglalás
│   │       │
│   │       ├── deadlock/         # Fázis 7 – Zsákutca kezelés
│   │       │   ├── mod.rs
│   │       │   ├── rip_up.rs     # Rip-up and reroute
│   │       │   ├── expand.rs     # Rács növelés
│   │       │   └── fallback.rs   # Keresztezéses fallback
│   │       │
│   │       ├── labels/           # Fázis 8 – Él címke elhelyezés
│   │       │   ├── mod.rs
│   │       │   ├── placement.rs  # Szegmens kiválasztás, pozíció-jelöltek
│   │       │   ├── collision.rs  # Ütközés-detektálás
│   │       │   └── slide.rs      # Eltolás a szegmens mentén
│   │       │
│   │       ├── render/           # Fázis 9 – SVG rendering
│   │       │   ├── mod.rs
│   │       │   ├── svg.rs        # SVG generálás
│   │       │   ├── nodes.rs      # Csomópont renderelés (rect, diamond, stb.)
│   │       │   ├── edges.rs      # Él renderelés (polyline, lekerekítés)
│   │       │   ├── crossing.rs   # Keresztezési híd
│   │       │   ├── subgraph.rs   # Subgraph keret renderelés
│   │       │   └── png.rs        # SVG → PNG konverzió
│   │       │
│   │       ├── decomposition/    # Fázis 13 – Dekompozíció
│   │       │   ├── mod.rs
│   │       │   ├── clustering.rs # Louvain + hibrid klaszterezés
│   │       │   ├── single.rs     # SINGLE mód (kétszintű routing)
│   │       │   └── multi.rs      # MULTI mód (több diagram)
│   │       │
│   │       └── pipeline.rs       # Teljes pipeline orchestráció
│   │
│   └── trellis-wasm/           # WASM interface
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs            # wasm-bindgen API
│
│   └── trellis-cli/            # CLI alkalmazás
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # Belépési pont, clap CLI
│           ├── commands/
│           │   ├── mod.rs
│           │   ├── render.rs     # `render` parancs
│           │   ├── render_batch.rs # `render-batch` parancs
│           │   ├── validate.rs   # `validate` parancs
│           │   └── license.rs    # `license` parancs (aktiválás, státusz)
│           ├── license/
│           │   ├── mod.rs
│           │   ├── state.rs      # Licencállapot (~/.trellis/license.json)
│           │   └── lemon.rs      # Lemon Squeezy API kliens
│           └── output/
│               ├── mod.rs
│               ├── svg_writer.rs # SVG fájl írás
│               └── png_writer.rs # PNG fájl írás (resvg)
│
├── editors/
│   ├── vscode/                   # VS Code Extension
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   ├── src/
│   │   │   ├── extension.ts      # Extension belépési pont
│   │   │   ├── preview.ts        # Diagram preview panel
│   │   │   ├── wasm-bridge.ts    # WASM betöltés és hívás
│   │   │   ├── license.ts        # Freemium/Premium kezelés
│   │   │   └── export.ts         # PNG/SVG/draw.io export
│   │   ├── media/
│   │   │   └── preview.html      # Preview webview template
│   │   └── wasm/                 # Build output (gitignored)
│   │       └── trellis_wasm_bg.wasm
│   │
│   └── intellij/                 # IntelliJ Plugin
│       ├── build.gradle.kts
│       ├── settings.gradle.kts
│       ├── src/main/kotlin/
│       │   ├── TrellisPlugin.kt      # Plugin belépési pont
│       │   ├── PreviewPanel.kt         # Diagram preview tool window
│       │   ├── WasmBridge.kt           # WASM betöltés (chicory/GraalWasm)
│       │   ├── LicenseManager.kt       # Freemium/Premium kezelés
│       │   └── ExportAction.kt         # PNG/SVG/draw.io export
│       ├── src/main/resources/
│       │   ├── META-INF/plugin.xml
│       │   └── wasm/                   # Build output (gitignored)
│       │       └── trellis_wasm_bg.wasm
│       └── src/test/kotlin/
│
├── tests/
│   ├── unit/                     # Rust unit tesztek (cargo test)
│   │   └── (a crate-eken belüli #[cfg(test)] modulok)
│   │
│   ├── integration/              # Integrációs tesztek
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── invariants.rs     # 7 invariáns ellenőrzés
│   │       └── pipeline_tests.rs # Teljes pipeline tesztek
│   │
│   └── benchmarks/               # Benchmark diagramok (B1-B12)
│       ├── Cargo.toml
│       ├── src/
│       │   └── bench.rs          # Criterion benchmark runner
│       └── fixtures/
│           ├── b01_linear_chain.mmd
│           ├── b02_wide_branch.mmd
│           ├── b03_k33_bipartite.mmd
│           ├── b04_diamond.mmd
│           ├── b05_star.mmd
│           ├── b06_multi_edge.mmd
│           ├── b07_cycle.mmd
│           ├── b08_nested_subgraph.mmd
│           ├── b09_subgraph_edges.mmd
│           ├── b10_50node_flowchart.mmd
│           ├── b11_100node_er.mmd
│           └── b12_class_hierarchy.mmd
│
├── docs/
│   ├── trellis-spec.md         # Algoritmus specifikáció
│   └── trellis-project-structure.md  # Ez a dokumentum
│
├── docker/
│   ├── Dockerfile                # CLI multi-stage build
│   └── docker-compose.yml        # Fejlesztői környezet (opcionális)
│
└── scripts/
    ├── build-wasm.sh             # Rust → WASM build
    ├── build-cli.sh              # CLI cross-compile (Linux/macOS/Windows)
    ├── build-docker.sh           # Docker image build
    ├── package-vscode.sh         # VS Code .vsix csomag
    └── package-intellij.sh       # IntelliJ .zip csomag
```

---

## 3. Core crate-ek

### Függőségi gráf

```
trellis-parser  (0 külső dep.)
       │
       ▼
trellis-core    (dep: trellis-parser)
       │
  ┌────┼────────────┐
  │    │             │
  ▼    ▼             ▼
CLI  WASM         (natív lib)
(clap,       (wasm-bindgen)
 resvg,
 reqwest)
```

### trellis-parser

Saját Mermaid parser, PEG-alapú.

```toml
# crates/trellis-parser/Cargo.toml
[package]
name = "trellis-parser"
version = "0.1.0"
edition = "2021"

[dependencies]
# Minimális: csak ha PEG generátor kell
# pest = "2"       # opcionális, kézi parser is lehet
```

**Felelőssége:** Mermaid szöveg → `Graph` AST (Fázis 1)

### trellis-core

A teljes algoritmus implementáció.

```toml
# crates/trellis-core/Cargo.toml
[package]
name = "trellis-core"
version = "0.1.0"
edition = "2021"

[dependencies]
trellis-parser = { path = "../trellis-parser" }

[dev-dependencies]
criterion = "0.5"   # benchmark
```

**Felelőssége:** Fázis 2-9 + dekompozíció + pipeline

### trellis-wasm

WASM binding réteg – a core-t hívja, de a plugin-specifikus logikát (licenc, export formátumok) nem tartalmazza.

```toml
# crates/trellis-wasm/Cargo.toml
[package]
name = "trellis-wasm"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
trellis-core = { path = "../trellis-core" }
wasm-bindgen = "0.2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### WASM API felület

```rust
// crates/trellis-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use trellis_core::{pipeline, config::TrellisConfig};

/// Fő belépési pont: Mermaid szöveg → SVG string
#[wasm_bindgen]
pub fn render(mermaid_source: &str, config_json: &str) -> Result<String, JsValue> {
    let config: TrellisConfig = serde_json::from_str(config_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let result = pipeline::render(mermaid_source, &config)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    Ok(result.svg)
}

/// Render + metrikák (benchmark mérésekhez)
#[wasm_bindgen]
pub fn render_with_metrics(mermaid_source: &str, config_json: &str) -> Result<String, JsValue> {
    // ... JSON-ban adja vissza az SVG-t + crossing count, bend count, render time, stb.
}
```

---

## 4. WASM build

```bash
# scripts/build-wasm.sh

#!/bin/bash
set -e

# wasm-pack build
cd crates/trellis-wasm
wasm-pack build --target web --out-dir ../../editors/vscode/wasm
wasm-pack build --target bundler --out-dir ../../editors/intellij/src/main/resources/wasm

echo "WASM build kész."
```

A `--target web` a VS Code-hoz (natív ES module), a `--target bundler` az IntelliJ-hez.

---

## 5. CLI alkalmazás

### trellis-cli crate

```toml
# crates/trellis-cli/Cargo.toml
[package]
name = "trellis-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "trellis"
path = "src/main.rs"

[dependencies]
trellis-core = { path = "../trellis-core" }
trellis-parser = { path = "../trellis-parser" }
clap = { version = "4", features = ["derive"] }     # CLI argumentumok
serde = { version = "1", features = ["derive"] }
serde_json = "1"
resvg = "0.44"                                       # SVG → PNG konverzió
reqwest = { version = "0.12", features = ["blocking", "json"] }  # Lemon Squeezy API
dirs = "5"                                           # ~/.trellis/ elérés
toml = "0.8"                                         # config.toml
```

### Belépési pont

```rust
// crates/trellis-cli/src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "trellis", version, about = "Mermaid diagram renderer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Mermaid fájl renderelése SVG/PNG kimenetté
    Render {
        /// Bemeneti fájl (vagy "-" stdin-hez)
        input: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(short, long, default_value = "svg")]
        format: String,
        #[arg(long, default_value = "single")]
        decomposition: String,
        #[arg(long, default_value = "50")]
        decomposition_threshold: usize,
        #[arg(long, default_value = "false")]
        metrics: bool,
        #[arg(long, default_value = "false")]
        quiet: bool,
    },
    /// Könyvtár összes .mmd fájljának renderelése
    RenderBatch {
        input_dir: String,
        #[arg(short, long)]
        output: String,
        #[arg(short, long, default_value = "svg")]
        format: String,
    },
    /// Mermaid szintaxis ellenőrzés renderelés nélkül
    Validate {
        input: String,
    },
    /// Licenc kezelés
    License {
        #[arg(long)]
        status: bool,
        #[arg(long)]
        activate: Option<String>,
        #[arg(long)]
        deactivate: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Render { .. } => commands::render::execute(..),
        Commands::RenderBatch { .. } => commands::render_batch::execute(..),
        Commands::Validate { .. } => commands::validate::execute(..),
        Commands::License { .. } => commands::license::execute(..),
    }
}
```

### Licencállapot tárolás

```rust
// crates/trellis-cli/src/license/state.rs

// Lokális tárolás: ~/.trellis/license.json
// {
//   "key": "38b1460a-...",
//   "tier": "premium",
//   "expires_at": "2027-02-08T00:00:00Z",
//   "instance_id": "cli-abc123",
//   "last_validated": "2026-02-08T10:00:00Z"
// }

pub fn license_path() -> PathBuf {
    dirs::home_dir().unwrap()
        .join(".trellis")
        .join("license.json")
}

pub fn check_license() -> LicenseTier {
    let state = load_license_state();
    match state {
        None => LicenseTier::Free,
        Some(s) if s.is_expired() => LicenseTier::Free,
        Some(s) if s.needs_revalidation() => {
            // Online revalidáció (max 30 nap grace period)
            match revalidate_online(&s) {
                Ok(valid) if valid => LicenseTier::Premium,
                _ if s.within_grace_period() => LicenseTier::Premium,
                _ => LicenseTier::Free,
            }
        }
        Some(_) => LicenseTier::Premium,
    }
}
```

### Freemium korlátok a CLI-ben

```rust
// crates/trellis-cli/src/commands/render.rs

pub fn execute(args: RenderArgs) -> Result<()> {
    let tier = license::check_license();
    let source = read_input(&args.input)?;
    let graph = trellis_parser::parse(&source)?;

    // Free tier korlátok
    if tier == LicenseTier::Free {
        if graph.nodes.len() > 10 {
            eprintln!("Free verzió: max 10 csomópont ({} található). \
                       Premium: trellis license --activate <kulcs>",
                       graph.nodes.len());
            std::process::exit(1);
        }
        if args.format == "svg" {
            eprintln!("Free verzió: csak PNG kimenet. \
                       SVG-hez Premium szükséges.");
            std::process::exit(1);
        }
    }

    let config = build_config(&args);
    let result = trellis_core::pipeline::render(&source, &config)?;

    // Vízjel (free tier)
    let svg = if tier == LicenseTier::Free {
        add_watermark(&result.svg)
    } else {
        result.svg
    };

    // Metrikák (mindkét tier-ben elérhető)
    if args.metrics {
        let metrics = serde_json::json!({
            "crossings": result.crossings,
            "bends": result.bends,
            "render_ms": result.render_time.as_millis(),
            "grid_utilization": result.grid_utilization,
        });
        eprintln!("{}", metrics);
    }

    write_output(&svg, &args)?;
    Ok(())
}
```

---

## 6. VS Code Extension

### Működés

```
┌─────────────────────────────────────────┐
│ VS Code                                 │
│                                         │
│  ┌─────────────┐     ┌───────────────┐  │
│  │  Editor     │────>│  Extension    │  │
│  │  .mmd fájl  │     │  (TS)        │  │
│  └─────────────┘     │              │  │
│                      │  wasm-bridge │  │
│                      │      │       │  │
│                      │  ┌───▼─────┐ │  │
│                      │  │  WASM   │ │  │
│                      │  │  core   │ │  │
│                      │  └───┬─────┘ │  │
│                      │      │       │  │
│  ┌─────────────┐     │  ┌───▼─────┐ │  │
│  │  Preview    │<────│  │   SVG   │ │  │
│  │  Panel      │     │  └─────────┘ │  │
│  └─────────────┘     └───────────────┘  │
└─────────────────────────────────────────┘
```

### Freemium korlátok helye

```typescript
// editors/vscode/src/license.ts

export interface LicenseState {
    tier: "free" | "premium";
    // Premium esetén: licenckulcs validálás
}

export function applyLimits(
    mermaidSource: string,
    state: LicenseState
): { source: string; warnings: string[] } {
    if (state.tier === "free") {
        const nodeCount = countNodes(mermaidSource);
        if (nodeCount > 10) {
            return {
                source: mermaidSource,
                warnings: ["Free verzió: max 10 csomópont. Premium verzióra van szükség."]
            };
        }
    }
    return { source: mermaidSource, warnings: [] };
}

export function shouldAddWatermark(state: LicenseState): boolean {
    return state.tier === "free";
}

export function getAllowedExportFormats(state: LicenseState): string[] {
    if (state.tier === "free") return ["png"];
    return ["png", "svg", "drawio"];
}
```

---

## 7. IntelliJ Plugin

### Működés

Hasonló a VS Code-hoz, de Kotlin-ból hívja a WASM-et:

```
┌─────────────────────────────────────────┐
│ IntelliJ                                │
│                                         │
│  ┌─────────────┐     ┌───────────────┐  │
│  │  Editor     │────>│   Plugin      │  │
│  │  .mmd fájl  │     │   (Kotlin)    │  │
│  └─────────────┘     │              │  │
│                      │  WasmBridge  │  │
│                      │      │       │  │
│                      │  ┌───▼─────┐ │  │
│                      │  │  WASM   │ │  │
│                      │  │(chicory)│ │  │
│                      │  └───┬─────┘ │  │
│                      │      │       │  │
│  ┌─────────────┐     │  ┌───▼─────┐ │  │
│  │  Tool       │<────│  │   SVG   │ │  │
│  │  Window     │     │  └─────────┘ │  │
│  └─────────────┘     └───────────────┘  │
└─────────────────────────────────────────┘
```

### WASM runtime opciók JVM-en

| Runtime | Előny | Hátrány |
|---|---|---|
| **Chicory** | Pure Java, nincs natív dep. | Lassabb |
| **GraalWasm** | Gyors (JIT) | GraalVM kell |
| **Wasmer-java** | Natív sebesség | Platform-specifikus natív lib |

Javasolt: **Chicory** (egyszerűség), majd később GraalWasm ha kell a sebesség.

---

## 8. Docker

### Dockerfile

```dockerfile
# docker/Dockerfile
# Multi-stage build: csak a CLI bináris kerül a végső image-be

# ── Build stage ──
FROM rust:1.82 AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --package trellis-cli

# ── Runtime stage ──
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/trellis /usr/local/bin/trellis
ENTRYPOINT ["trellis"]
```

### Image méret

| Stage | Méret (becsült) |
|---|---|
| Builder | ~2 GB (Rust toolchain + deps) |
| Runtime | ~30-50 MB (slim Debian + bináris) |

### Build és publish

```bash
# scripts/build-docker.sh
#!/bin/bash
set -e

VERSION=$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name=="trellis-cli") | .version')

docker build -f docker/Dockerfile -t ghcr.io/trellis/trellis:${VERSION} .
docker tag ghcr.io/trellis/trellis:${VERSION} ghcr.io/trellis/trellis:latest

echo "Docker image kész: ghcr.io/trellis/trellis:${VERSION}"
```

---

## 9. Build pipeline

### Fejlesztői build

```bash
# 1. Core tesztek
cargo test --workspace

# 2. CLI build + teszt
cargo build --package trellis-cli
cargo test --package trellis-cli

# 3. WASM build
./scripts/build-wasm.sh

# 4. VS Code extension
cd editors/vscode
npm install
npm run compile

# 5. IntelliJ plugin
cd editors/intellij
./gradlew build
```

### Release build

```bash
# 1. Core tesztek + benchmarkok
cargo test --workspace
cargo bench --package trellis-core

# 2. CLI bináris (cross-compile)
./scripts/build-cli.sh
# → target/release/trellis          (Linux x86_64)
# → target/x86_64-apple-darwin/       (macOS x86_64)
# → target/aarch64-apple-darwin/      (macOS ARM)
# → target/x86_64-pc-windows-msvc/   (Windows)

# 3. Docker image
./scripts/build-docker.sh

# 4. WASM build (optimalizált)
cd crates/trellis-wasm
wasm-pack build --release --target web --out-dir ../../editors/vscode/wasm
wasm-pack build --release --target bundler --out-dir ../../editors/intellij/src/main/resources/wasm

# 5. wasm-opt optimalizáció
wasm-opt -O3 editors/vscode/wasm/trellis_wasm_bg.wasm -o editors/vscode/wasm/trellis_wasm_bg.wasm

# 6. VS Code package
cd editors/vscode
vsce package  # → trellis-x.y.z.vsix

# 7. IntelliJ package
cd editors/intellij
./gradlew buildPlugin  # → build/distributions/trellis-x.y.z.zip
```

### CI (GitHub Actions)

```
┌──────────────────────────────────────────────────────────────┐
│  PR / push to main                                           │
│                                                              │
│  ┌──────────┐  ┌───────────┐  ┌────────────────────────────┐│
│  │cargo test│─>│cargo bench│─>│ build-wasm + build-cli     ││
│  │          │  │(B1-B12)   │  │                            ││
│  └──────────┘  └───────────┘  └─────────┬──────────────────┘│
│                                         │                    │
│                    ┌────────────────┬────┼──────────┐        │
│               ┌────▼───┐      ┌────▼──┐ │     ┌────▼─────┐  │
│               │ vscode │      │ ij    │ │     │  Docker  │  │
│               │ build  │      │ build │ │     │  build   │  │
│               └────┬───┘      └───┬───┘ │     └────┬─────┘  │
│                    │              │      │          │         │
│               ┌────▼──────────────▼──────▼──────────▼──────┐ │
│               │          Artifact upload                   │ │
│               │  .vsix + .zip + CLI binaries + Docker push │ │
│               └────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

---

## 10. Fejlesztői workflow

### Napi munka

| Feladat | Parancs |
|---|---|
| Core fejlesztés + teszt | `cargo test -p trellis-core` |
| Parser fejlesztés + teszt | `cargo test -p trellis-parser` |
| CLI fejlesztés + teszt | `cargo test -p trellis-cli` |
| CLI futtatás (gyors teszt) | `cargo run -p trellis-cli -- render test.mmd -o test.svg` |
| Benchmark futtatás | `cargo bench -p trellis-core` |
| WASM újrafordítás | `./scripts/build-wasm.sh` |
| Docker build | `./scripts/build-docker.sh` |
| VS Code extension debug | `cd editors/vscode && code . → F5` |
| IntelliJ plugin debug | `cd editors/intellij → Run Plugin (Gradle task)` |

### Új fázis hozzáadása

1. Rust modul hozzáadása `crates/trellis-core/src/` alá
2. Unit tesztek a modulon belül (`#[cfg(test)]`)
3. Integrálás `pipeline.rs`-be
4. Integrációs tesztek `tests/integration/` alatt
5. CLI-vel manuális teszt: `cargo run -p trellis-cli -- render fixture.mmd --metrics`
6. WASM API bővítés ha kell (`trellis-wasm`)
7. `./scripts/build-wasm.sh` → plugin tesztje

### Benchmark hozzáadása

1. `.mmd` fájl a `tests/benchmarks/fixtures/` alá
2. Criterion bench regisztrálása `tests/benchmarks/src/bench.rs`-ben
3. `cargo bench` → eredmények a `target/criterion/` alatt
4. CLI-vel ellenőrzés: `cargo run -p trellis-cli -- render fixtures/bXX.mmd --metrics`
