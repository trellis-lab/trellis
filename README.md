# Trellis

A fast Mermaid diagram renderer built in Rust.

## Project Structure

```
trellis/
├── crates/
│   ├── trellis-parser/    # Mermaid syntax parser
│   ├── trellis-core/      # Core rendering engine
│   ├── trellis-wasm/      # WASM bindings
│   └── trellis-cli/       # Command-line interface
├── tests/
│   └── benchmarks/
│       └── fixtures/      # Benchmark test files (b01-b12.mmd)
└── docs/                  # Documentation and specifications
```

## Status: M1 - Project Skeleton ✅

### Completed
- ✅ Cargo workspace setup
- ✅ trellis-parser crate with AST structures
- ✅ trellis-core crate with config and pipeline
- ✅ trellis-wasm crate with wasm-bindgen stubs
- ✅ trellis-cli crate with clap CLI
- ✅ Benchmark fixture files (b01-b12.mmd)

### Next Steps

1. **Install Rust** (if not already installed):
   ```bash
   # Visit https://rustup.rs/ or run:
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Verify the workspace**:
   ```bash
   # Test all crates compile
   cargo test --workspace

   # Test CLI version command
   cargo run -p trellis-cli -- --version

   # Test smoke test - render placeholder
   cargo run -p trellis-cli -- render tests/benchmarks/fixtures/b01.mmd -o test.svg
   ```

## Development

### Building
```bash
cargo build --workspace
```

### Testing
```bash
cargo test --workspace
```

### Running the CLI
```bash
# Render a diagram
cargo run -p trellis-cli -- render input.mmd -o output.svg

# Validate a diagram
cargo run -p trellis-cli -- validate input.mmd
```

## Milestones

- **M1**: Project skeleton ✅ (Current)
- **M2**: Mermaid parser (Flowchart)
- **M3**: Node placement (Sugiyama algorithm)
- **M4**: Grid building + port assignment
- **M5**: Edge routing (A* pathfinding)
- **M6**: SVG rendering (first visual output)
- ... (see docs/trellis-implementation-plan.md for full roadmap)

## License

MIT OR Apache-2.0
