# Trellis

A fast Mermaid diagram renderer built in Rust, using VLSI maze routing (A* pathfinding) to produce overlap-free orthogonal edge layouts.

Supports: **flowchart**, **class**, **ER**, and **C4** diagram types.

## Project Structure

```
trellis/
├── crates/
│   ├── trellis-parser/    # Mermaid syntax parser (nom-based)
│   ├── trellis-core/      # Core rendering pipeline (placement → grid → routing → SVG)
│   ├── trellis-wasm/      # WASM bindings (wasm-bindgen)
│   └── trellis-cli/       # Command-line interface
├── docker/
│   ├── Dockerfile.wasm-builder      # Builds WASM (Rust + wasm-pack)
│   ├── wasm-entrypoint.sh
│   ├── Dockerfile.vscode-builder    # Builds the VS Code .vsix (Node.js + vsce)
│   ├── vscode-entrypoint.sh
│   ├── Dockerfile.intellij-builder  # Builds the IntelliJ .zip (JDK 17 + Gradle)
│   └── intellij-entrypoint.sh
├── editors/
│   ├── vscode/            # VS Code extension
│   └── intellij/          # IntelliJ platform plugin
├── tests/
│   └── benchmarks/
│       └── fixtures/      # Benchmark test files (b01-b12.mmd)
└── docs/                  # Specifications and implementation plan
```

## CLI

### Build

```bash
cargo build --workspace
```

### Usage

```bash
# Render a diagram to SVG
cargo run -p trellis-cli -- render input.mmd -o output.svg

# Validate a diagram (parse only, no render)
cargo run -p trellis-cli -- validate input.mmd

# Batch render a directory of .mmd files
cargo run -p trellis-cli -- render-batch ./diagrams -o ./output

# Render from stdin
echo "flowchart LR\n  A --> B" | cargo run -p trellis-cli -- render - -o out.svg
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run benchmarks
cargo bench -p benchmarks

# Static analysis
cargo clippy --workspace
```

---

## Building WASM (required by both editor extensions)

The VS Code extension and IntelliJ plugin both need the compiled WASM binary.
There are two ways to build it:

### Option A — Docker (recommended, no Rust/wasm-pack needed locally)

Requires only [Docker Desktop](https://www.docker.com/products/docker-desktop/) (Windows, macOS, Linux).

```bash
./scripts/build-wasm-docker.sh           # dev build
./scripts/build-wasm-docker.sh --release # optimised release build
```

The first run downloads the Rust toolchain and wasm-pack inside the image (~1–2 min); subsequent runs use Docker's layer cache and are fast. Output is written directly to both editor trees:

```
editors/vscode/wasm/
editors/intellij/src/main/resources/wasm/
```

### Option B — Native (requires Rust + wasm-pack installed)

```bash
# Install the wasm32 target once
rustup target add wasm32-unknown-unknown

# Run the build script
./scripts/build-wasm.sh           # debug
./scripts/build-wasm.sh --release # optimised
```

> **Windows note:** run from Git Bash or WSL with wasm-pack installed natively
> (`curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh`).

---

## VS Code Extension

### Build (Docker — no Node.js needed locally)

```bash
# Step 1: build WASM (once, or after Rust changes)
./scripts/build-wasm-docker.sh

# Step 2: build the .vsix package
./scripts/build-vscode-docker.sh
# → dist/trellis-preview-0.1.0.vsix
```

### Build (native — requires Node.js 18+ and WASM already built)

```bash
./scripts/build-wasm-docker.sh   # or ./scripts/build-wasm.sh
cd editors/vscode
npm install
npm run compile
npm run package
# → editors/vscode/trellis-preview-0.1.0.vsix
```

### Install

```bash
code --install-extension dist/trellis-preview-0.1.0.vsix
```

### Run in development (VS Code F5)

1. Open the repo root in VS Code.
2. Press **F5** — launches an Extension Development Host.
3. Open any `.mmd` file.
4. Press **Ctrl+Shift+V** (macOS: **Cmd+Shift+V**) to open the preview panel.

---

## IntelliJ Plugin

### Build (Docker — no JDK/Gradle needed locally)

```bash
# Step 1: build WASM (once, or after Rust changes)
./scripts/build-wasm-docker.sh

# Step 2: build the plugin ZIP
./scripts/build-intellij-docker.sh
# → dist/trellis-intellij-0.1.0.zip
```

Gradle and the IntelliJ sandbox (~600 MB) are cached in a named Docker volume
(`trellis-gradle-cache`) so the second run is much faster.

### Build (native — requires JDK 17+ and WASM already built)

```bash
./scripts/build-wasm-docker.sh   # or ./scripts/build-wasm.sh
cd editors/intellij
./gradlew buildPlugin
# → editors/intellij/build/distributions/trellis-intellij-0.1.0.zip
```

### Install

1. In IntelliJ IDEA: **Settings → Plugins → ⚙ → Install Plugin from Disk…**
2. Select `dist/trellis-intellij-0.1.0.zip`.
3. Restart the IDE — the **Trellis Preview** tool window appears on the right.

### Run in development

```bash
cd editors/intellij
./gradlew runIde   # downloads a sandboxed IDEA instance on first run
```

Open any `.mmd` file — the **Trellis Preview** tool window appears on the right.

### Settings

**Settings → Tools → Trellis** (or search "Trellis" in settings):

| Setting | Default | Description |
|---|---|---|
| Auto Preview | `true` | Re-render on every keystroke / file save |
| Default Direction | `TB` | Flowchart layout direction (`TB`, `LR`, `BT`, `RL`) |
| Default Theme | `default` | Colour theme (`default`, `dark`, `neutral`) |

---

## Milestones

| Milestone | Status | Description |
|---|---|---|
| M1 | ✅ | Project skeleton |
| M2 | ✅ | Flowchart parser |
| M3 | ✅ | Sugiyama placement |
| M4 | ✅ | Grid + ports |
| M5 | ✅ | A* edge routing |
| M6 | ✅ | SVG output |
| M7 | ✅ | Deadlock handling (rip-up & reroute) |
| M8 | ✅ | Edge labels |
| M9 | ✅ | Subgraph visualisation |
| M10 | ✅ | Class diagram |
| M11 | ✅ | ER diagram |
| M12 | ✅ | C4 diagram types |
| M13 | ✅ | Benchmarks (B01–B12) |
| M14 | ✅ | CLI (render-batch, validate, stdin, Pandoc filter) |
| M15 | ✅ | WASM + VS Code extension |
| M16 | ✅ | IntelliJ plugin |
| M17 | ⬜ | Docker + CI/CD |
| M18 | ⬜ | Decomposition (50+ node graphs) |
| M19 | ⬜ | Commercial features (licensing, freemium) |

See [docs/trellis-implementation-plan.md](docs/trellis-implementation-plan.md) for the full roadmap.

## License

MIT OR Apache-2.0
