---
globs: ["**/*.ts", "**/*.tsx", "**/tsconfig*.json"]
---

# Agent Instructions: TypeScript VS Code Extension with Rust/WASM

> This file governs all AI-assisted development on this project.  
> Follow every rule here unless explicitly overridden in a narrower-scope instruction file.

---

## 1. Project Philosophy

- **Small files.** Every source file has a single, clear responsibility. If a file exceeds ~150 lines, it is a signal to split it.
- **Separation of concerns.** The extension host (TypeScript), the UI layer (VS Code API), and the business logic (WASM/Rust) are strictly decoupled — they never cross-import directly; they communicate through defined interfaces.
- **No logic in glue code.** Files that wire pieces together (registrations, loaders, entry points) must contain zero business logic.
- **Explicit over implicit.** Prefer named exports, explicit return types, and clear function signatures over inference magic.

---

## 2. Project Structure

```
my-extension/
├── src/
│   ├── extension.ts          # Entry point — ONLY activates & registers, no logic
│   ├── commands/             # One file per VS Code command
│   │   └── myCommand.ts
│   ├── providers/            # Language providers, tree views, webview providers
│   │   └── myProvider.ts
│   ├── wasm/
│   │   ├── loader.ts         # WASM module loading & lifecycle only
│   │   ├── bridge.ts         # TypeScript ↔ WASM type-safe interface
│   │   └── types.ts          # Shared types mirroring Rust structs
│   ├── services/             # Pure TS business logic that does NOT touch VS Code API
│   │   └── myService.ts
│   ├── utils/                # Pure, stateless helper functions
│   │   └── strings.ts
│   └── types/                # Global shared TypeScript types & interfaces
│       └── index.ts
├── wasm-src/                 # Rust source (compiled separately)
│   └── src/
│       └── lib.rs
├── resources/
│   └── my_extension_bg.wasm  # Compiled WASM binary (do not edit)
├── package.json
├── tsconfig.json
└── AGENTS.md                 # ← This file
```

### Directory rules

| Directory | Allowed imports | Forbidden imports |
|---|---|---|
| `commands/` | `services/`, `wasm/bridge`, `types/`, VS Code API | Direct WASM, other commands |
| `providers/` | `services/`, `wasm/bridge`, `types/`, VS Code API | Direct WASM, other providers |
| `services/` | `types/`, `utils/` | VS Code API, WASM loader |
| `wasm/bridge.ts` | `wasm/types.ts`, `wasm/loader.ts` | VS Code API, services |
| `wasm/loader.ts` | Node `fs`, VS Code `Uri` | Business logic, services |
| `utils/` | Nothing project-internal | Everything else |

---

## 3. TypeScript Configuration

Use a **balanced strict** `tsconfig.json`:

```jsonc
{
  "compilerOptions": {
    // Targeting
    "target": "ES2020",
    "module": "Node16",
    "moduleResolution": "Node16",
    "lib": ["ES2020"],

    // Strictness — balanced
    "strict": true,                  // enables: strictNullChecks, noImplicitAny, etc.
    "noUncheckedIndexedAccess": true, // array[i] returns T | undefined
    "exactOptionalPropertyTypes": false, // relaxed: allows undefined assignment
    "noPropertyAccessFromIndexSignature": false, // relaxed for record-style objects

    // Quality
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "forceConsistentCasingInFileNames": true,

    // Output
    "outDir": "out",
    "rootDir": "src",
    "sourceMap": true,
    "declaration": true,

    // Interop
    "esModuleInterop": true,
    "skipLibCheck": true
  },
  "include": ["src"],
  "exclude": ["node_modules", "out", "wasm-src"]
}
```

---

## 4. TypeScript Patterns

### 4.1 File & naming conventions

- One export concept per file. A file exporting `MyService` should not also export unrelated helpers.
- File names: `camelCase.ts` for modules, `PascalCase.ts` for class-primary files.
- Interfaces over `type` aliases for object shapes; `type` for unions, intersections, and utility types.

### 4.2 Always annotate public API return types

```typescript
// ✅ Good
export function parseResult(raw: string): ParsedResult {
  ...
}

// ❌ Bad — relies on inference, breaks AI-readable contracts
export function parseResult(raw: string) {
  ...
}
```

### 4.3 Avoid classes unless statefulness is required

Prefer plain functions and objects. Use classes only when you need instance state or VS Code requires a class (e.g., `TreeDataProvider`).

```typescript
// ✅ Prefer
export function createDiagnosticService(config: Config): DiagnosticService { ... }

// ⚠️ Only when needed
export class MyTreeProvider implements vscode.TreeDataProvider<Item> { ... }
```

### 4.4 Error handling

- Never `throw` raw strings. Always throw typed `Error` subclasses or use `Result`-style returns.
- Async functions must handle rejections — never leave a floating `Promise`.

```typescript
// Result-style (preferred for WASM boundary)
type Result<T> = { ok: true; value: T } | { ok: false; error: string };

export async function invokeWasm(input: Input): Promise<Result<Output>> {
  try {
    const value = await bridge.process(input);
    return { ok: true, value };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}
```

### 4.5 Avoid barrel re-exports (`index.ts`) in deep directories

Barrel files hide dependency graphs. Use them only at the top-level `types/index.ts`. Import directly everywhere else:

```typescript
// ✅
import { parseResult } from '../services/parserService';

// ❌
import { parseResult } from '../services';
```

### 4.6 Immutability

Prefer `readonly` on all interfaces and function parameters that should not be mutated:

```typescript
interface Config {
  readonly endpoint: string;
  readonly timeoutMs: number;
}
```

---

## 5. VS Code Extension API

### 5.1 `extension.ts` is a wiring file only

```typescript
// extension.ts — ONLY this pattern is allowed here
import * as vscode from 'vscode';
import { registerMyCommand } from './commands/myCommand';
import { MyProvider } from './providers/myProvider';
import { loadWasm } from './wasm/loader';

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await loadWasm(context.extensionUri);
  registerMyCommand(context);
  context.subscriptions.push(new MyProvider());
}

export function deactivate(): void {}
```

### 5.2 One file per command

```typescript
// commands/myCommand.ts
import * as vscode from 'vscode';
import { myService } from '../services/myService';

export function registerMyCommand(context: vscode.ExtensionContext): void {
  const disposable = vscode.commands.registerCommand(
    'myExtension.myCommand',
    async () => {
      const result = await myService.doSomething();
      if (!result.ok) {
        vscode.window.showErrorMessage(result.error);
        return;
      }
      vscode.window.showInformationMessage(`Done: ${result.value}`);
    }
  );
  context.subscriptions.push(disposable);
}
```

### 5.3 Always push disposables to `context.subscriptions`

Every `vscode.*` registration must be disposed on deactivation. Never skip this.

### 5.4 Never access `vscode` from `services/` or `wasm/`

Services and WASM bridge must remain framework-agnostic for testability.

---

## 6. WASM Integration (Rust via `wasm-pack`)

### 6.1 Loader — single responsibility: load & cache

```typescript
// wasm/loader.ts
import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';

let wasmModule: typeof import('../../resources/pkg') | undefined;

export async function loadWasm(extensionUri: vscode.Uri): Promise<void> {
  if (wasmModule) return; // already loaded
  const wasmPath = vscode.Uri.joinPath(extensionUri, 'resources', 'pkg', 'my_extension_bg.wasm');
  const bytes = await fs.readFile(wasmPath.fsPath);
  // wasm-pack generated init
  const mod = await import('../../resources/pkg');
  await mod.default(bytes);
  wasmModule = mod;
}

export function getWasm(): typeof import('../../resources/pkg') {
  if (!wasmModule) throw new Error('WASM not loaded. Call loadWasm() first.');
  return wasmModule;
}
```

### 6.2 Bridge — single responsibility: typed interface

The bridge translates between TypeScript types and WASM-exported functions. It contains no logic beyond translation and error wrapping.

```typescript
// wasm/bridge.ts
import { getWasm } from './loader';
import type { WasmInput, WasmOutput } from './types';
import type { Result } from '../types';

export async function processInput(input: WasmInput): Promise<Result<WasmOutput>> {
  try {
    const wasm = getWasm();
    const raw = wasm.process(JSON.stringify(input));
    const output: WasmOutput = JSON.parse(raw);
    return { ok: true, value: output };
  } catch (e) {
    return { ok: false, error: String(e) };
  }
}
```

### 6.3 WASM types mirror Rust structs exactly

Keep `wasm/types.ts` in sync with your Rust public structs. Use `serde_json` in Rust and JSON as the serialization boundary — avoid raw memory passing unless performance-critical.

```typescript
// wasm/types.ts
export interface WasmInput {
  readonly source: string;
  readonly options: ProcessOptions;
}

export interface ProcessOptions {
  readonly maxDepth: number;
  readonly verbose: boolean;
}

export interface WasmOutput {
  readonly result: string;
  readonly diagnostics: readonly Diagnostic[];
}

export interface Diagnostic {
  readonly severity: 'error' | 'warning' | 'info';
  readonly message: string;
  readonly line: number;
}
```

### 6.4 WASM is loaded once, at activation

Never call `loadWasm()` inside a command or provider. It must be called exactly once in `extension.ts` `activate()`.

### 6.5 Rust/wasm-pack conventions

- Expose only the minimum public API from Rust (`#[wasm_bindgen]` only on boundary functions).
- Use `console_error_panic_hook` in Rust for readable WASM panics during development.
- Build with `wasm-pack build --target nodejs` for VS Code extensions (Node.js environment).
- Keep the compiled `pkg/` output in `resources/` and commit the `.wasm` binary.

---

## 7. File Size & Splitting Rules

| Trigger | Action |
|---|---|
| File > 150 lines | Split by extracting a focused sub-concern into a new file |
| Function > 30 lines | Extract helper functions; each function does one thing |
| More than 3 imports from same sibling directory | Consider a shared sub-module |
| Mixed VS Code API + pure logic in same file | Separate into `provider` + `service` |

---

## 8. Testing Conventions

- `services/` and `utils/` must be unit-testable with zero VS Code dependencies.
- `wasm/bridge.ts` is tested with a mock `loader.ts`.
- Commands and providers are integration-tested using `@vscode/test-electron`.
- Test files live next to source: `myService.test.ts` beside `myService.ts`.

---

## 9. What AI Agents Must Not Do

- Do not add business logic to `extension.ts`, command files, or provider files.
- Do not import `vscode` in `services/`, `utils/`, or `wasm/`.
- Do not create files longer than 150 lines without splitting.
- Do not use `any` — use `unknown` and narrow with type guards.
- Do not use `!` non-null assertions on WASM or VS Code API return values — handle `undefined` explicitly.
- Do not combine multiple commands or providers into one file.
- Do not skip `context.subscriptions.push(...)` for any disposable.
- Do not call `loadWasm()` more than once or outside `activate()`.
