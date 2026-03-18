---
paths: ["**/*.kt", "**/*.kts", "**/build.gradle.kts", "**/settings.gradle.kts"]
---

# Kotlin IntelliJ Plugin Development — Best Practices for AI Agents

> Instruction file for AI agents generating or reviewing Kotlin code.  
> Stack: **IntelliJ Platform SDK · Embedded WASM binary · Gradle (Kotlin DSL)**  
> Core principle: **small, focused files — strict separation of concerns — max ~150 lines per file.**

---

## 1. Project Structure

```
plugin-root/
├── build.gradle.kts
├── settings.gradle.kts
├── gradle/
│   └── libs.versions.toml            # Version catalog
└── src/
    └── main/
        ├── kotlin/
        │   └── com/example/plugin/
        │       ├── domain/
        │       │   ├── model/        # Pure Kotlin data types; zero SDK imports
        │       │   ├── port/         # Interfaces only (WasmRuntimePort, AnalysisPort…)
        │       │   └── usecase/      # One file per use-case
        │       ├── application/
        │       │   └── service/      # Orchestration; calls use-cases & ports
        │       ├── adapter/
        │       │   ├── wasm/         # WASM loader, bridge, memory helpers
        │       │   └── ide/          # IntelliJ extension point implementations
        │       └── infrastructure/
        │           ├── config/       # Plugin settings (PersistentStateComponent)
        │           └── di/           # Service locator / light-DI wiring
        └── resources/
            ├── META-INF/
            │   ├── plugin.xml        # Extension point declarations only
            │   └── actions.xml       # Action declarations (xi:included)
            └── wasm/
                └── engine.wasm       # Compiled WASM binary — never edit manually
```

**Dependency direction (strictly enforced):**
```
adapter/ide  ──►  application  ──►  domain
adapter/wasm ──►  application  ──►  domain
infrastructure              ──►  domain (config only)
```

- `domain/` has **zero** imports from `com.intellij.*`, `java.awt.*`, or any WASM runtime package.
- `adapter/wasm/` is the **only** layer allowed to touch raw WASM memory / JVM FFI.
- `adapter/ide/` is the **only** layer allowed to use `com.intellij.*` APIs.

---

## 2. Naming Conventions

| Element | Convention | Example |
|---|---|---|
| Classes / Objects | `PascalCase` | `WasmEngineAdapter` |
| Functions / variables | `camelCase` | `invokeWasmFunction` |
| Constants | `UPPER_SNAKE_CASE` | `WASM_PAGE_SIZE` |
| Packages | `lowercase` | `com.example.plugin.adapter.wasm` |
| Use-case classes | `<Verb><Noun>UseCase` | `RunAnalysisUseCase` |
| Port interfaces | `<Noun>Port` | `WasmRuntimePort` |
| WASM adapters | `<Noun>WasmAdapter` | `AnalysisWasmAdapter` |
| IDE actions | `<Verb><Noun>Action` | `RunAnalysisAction` |
| IDE inspections | `<Noun>Inspection` | `AnalysisInspection` |
| IDE tool windows | `<Noun>ToolWindow` | `AnalysisToolWindow` |
| Settings state | `<Noun>State` | `PluginSettingsState` |
| Platform services | `<Noun>Service` | `AnalysisService` |
| WASM pointer wrapper | `WasmPtr` | `WasmPtr` |

---

## 3. Domain Layer (`domain/`)

### 3.1 Models
- `data class`, fully immutable (`val` only). No framework or SDK annotations.
- Use `@JvmInline value class` for typed primitives — memory addresses, IDs, etc.

```kotlin
// domain/model/AnalysisId.kt
@JvmInline
value class AnalysisId(val value: String)

// domain/model/AnalysisResult.kt
data class AnalysisResult(
    val id: AnalysisId,
    val findings: List<Finding>,
    val durationMs: Long,
)
```

### 3.2 Ports (interfaces)
- One interface per file. Signatures express *what*, never *how*.
- No WASM types, no IntelliJ types in any signature.

```kotlin
// domain/port/WasmRuntimePort.kt
interface WasmRuntimePort {
    fun invoke(function: String, payload: ByteArray): ByteArray
    fun isReady(): Boolean
}

// domain/port/AnalysisPort.kt
interface AnalysisPort {
    suspend fun analyse(input: AnalysisInput): AnalysisResult
}
```

### 3.3 Use-cases
- One public entry point per file (`operator fun invoke` or `execute`).
- No coroutine scope ownership — callers provide the scope.
- Return `Result<T>` or a sealed `Outcome<T>`; never swallow exceptions.

```kotlin
// domain/usecase/RunAnalysisUseCase.kt
class RunAnalysisUseCase(private val analysis: AnalysisPort) {
    suspend operator fun invoke(input: AnalysisInput): Result<AnalysisResult> =
        runCatching { analysis.analyse(input) }
}
```

---

## 4. WASM Adapter Layer (`adapter/wasm/`)

> This is the most sensitive layer. Each file must have exactly one responsibility.

### 4.1 File responsibilities

| File | Sole responsibility |
|---|---|
| `WasmLoader.kt` | Locate, validate, and load `engine.wasm` from plugin resources |
| `WasmPtr.kt` | Typed wrapper for WASM linear memory addresses |
| `WasmMemoryBridge.kt` | Read/write linear memory; JVM ↔ WASM type conversion |
| `WasmFunctionRegistry.kt` | Map Kotlin function names → WASM exported function handles |
| `WasmRuntimeAdapter.kt` | Implements `WasmRuntimePort`; composes the above |

### 4.2 Loading the binary

```kotlin
// adapter/wasm/WasmLoader.kt
object WasmLoader {
    private const val RESOURCE_PATH = "/wasm/engine.wasm"

    fun loadBytes(): ByteArray =
        WasmLoader::class.java.getResourceAsStream(RESOURCE_PATH)
            ?.readBytes()
            ?: error("WASM binary not found at $RESOURCE_PATH")
}
```

### 4.3 Memory bridge rules
- Raw `Int` addresses must never be exposed outside `WasmMemoryBridge`.
- Model pointers exclusively as `WasmPtr`.
- Always release WASM-allocated memory in a `try/finally` block.

```kotlin
// adapter/wasm/WasmPtr.kt
@JvmInline
value class WasmPtr(val address: Int)

// adapter/wasm/WasmMemoryBridge.kt
class WasmMemoryBridge(private val memory: Any /* runtime-specific memory handle */) {
    fun writeBytes(data: ByteArray): WasmPtr { /* allocate + write; return ptr */ }
    fun readBytes(ptr: WasmPtr, length: Int): ByteArray { /* read from linear memory */ }
    fun free(ptr: WasmPtr) { /* deallocate */ }
}
```

### 4.4 Threading discipline
- WASM execution is **blocking** — always dispatch on `Dispatchers.Default` or a dedicated `ExecutorCoroutineDispatcher`. Never on the EDT.
- The `WasmRuntimeAdapter` is the single place where the thread switch is enforced.

```kotlin
// adapter/wasm/WasmRuntimeAdapter.kt
class WasmRuntimeAdapter(
    private val bridge: WasmMemoryBridge,
    private val registry: WasmFunctionRegistry,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default,
) : WasmRuntimePort {

    override fun invoke(function: String, payload: ByteArray): ByteArray =
        runBlocking(dispatcher) {
            val fn = registry.resolve(function)
            val ptr = bridge.writeBytes(payload)
            try {
                val resultPtr = fn.call(ptr)
                bridge.readBytes(resultPtr, fn.lastResultLength())
            } finally {
                bridge.free(ptr)
            }
        }

    override fun isReady(): Boolean = registry.isInitialised()
}
```

### 4.5 WASM runtime library choice
- Prefer a pure-JVM runtime (e.g. **Chicory**, **GraalWasm** in native-image-free mode) to avoid native agent requirements.
- Pin the runtime version in `libs.versions.toml`. Never use a range version.
- Isolate runtime-specific types inside `adapter/wasm/` — if the runtime is swapped, no other package changes.

---

## 5. IDE Adapter Layer (`adapter/ide/`)

### 5.1 File responsibilities

| File | Sole responsibility |
|---|---|
| `RunAnalysisAction.kt` | `AnAction` — extracts IDE context, delegates to service |
| `AnalysisInspection.kt` | `LocalInspectionTool` — delegates logic to use-case |
| `AnalysisToolWindowFactory.kt` | `ToolWindowFactory` — creates and wires the tool window panel |
| `AnalysisToolWindowPanel.kt` | The Swing/UI panel; no business logic |
| `PluginServiceLocator.kt` | Retrieves app/project services from `ServiceManager` |

### 5.2 Actions — no logic, only delegation

```kotlin
// adapter/ide/RunAnalysisAction.kt
class RunAnalysisAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        PluginServiceLocator.analysisService(project).runInBackground(project)
    }

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.project != null
    }
}
```

### 5.3 Services — lifecycle-aware coroutine scope

```kotlin
// adapter/ide/AnalysisService.kt  (project-level light service)
@Service(Service.Level.PROJECT)
class AnalysisService(private val project: Project) : Disposable {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    // useCase and port are wired in infrastructure/di/PluginWiring.kt
    private val useCase: RunAnalysisUseCase by lazy {
        PluginWiring.buildRunAnalysisUseCase()
    }

    fun runInBackground(project: Project) {
        scope.launch {
            useCase(AnalysisInput.fromProject(project))
                .onSuccess { showNotification(project, it) }
                .onFailure { reportError(project, it) }
        }
    }

    override fun dispose() = scope.cancel()
}
```

### 5.4 EDT rules
| Operation | Thread | Mechanism |
|---|---|---|
| Read PSI / VFS | Any | `ReadAction.compute { }` |
| Write PSI | EDT only | `WriteCommandAction.runWriteCommandAction(...)` |
| WASM calls | Background | `Dispatchers.Default` via `WasmRuntimeAdapter` |
| Update UI | EDT | `ApplicationManager.getApplication().invokeLater { }` |

---

## 6. Infrastructure Layer (`infrastructure/`)

### 6.1 Settings — split state from component

```kotlin
// infrastructure/config/PluginSettingsState.kt
data class PluginSettingsState(
    @Tag var wasmTimeoutMs: Long = 5_000L,
    @Tag var enableVerboseLogging: Boolean = false,
)

// infrastructure/config/PluginSettings.kt
@State(name = "PluginSettings", storages = [Storage("pluginSettings.xml")])
@Service(Service.Level.APP)
class PluginSettings : PersistentStateComponent<PluginSettingsState> {
    private var state = PluginSettingsState()
    override fun getState(): PluginSettingsState = state
    override fun loadState(s: PluginSettingsState) { state = s }
}
```

### 6.2 Dependency wiring

```kotlin
// infrastructure/di/PluginWiring.kt
object PluginWiring {
    fun buildRunAnalysisUseCase(): RunAnalysisUseCase {
        val loader  = WasmLoader
        val bridge  = WasmMemoryBridge(buildMemoryHandle(loader.loadBytes()))
        val registry = WasmFunctionRegistry(bridge)
        val runtime  = WasmRuntimeAdapter(bridge, registry)
        val port     = AnalysisWasmAdapter(runtime)
        return RunAnalysisUseCase(port)
    }
}
```

- Domain/application code uses constructor injection; never calls `PluginWiring` directly.
- `PluginServiceLocator` and `PluginWiring` live exclusively in `infrastructure/`.

---

## 7. Gradle Setup

### `build.gradle.kts`

```kotlin
plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.intellijPlatform)
}

kotlin {
    jvmToolchain(17)               // IntelliJ 2024+ requires JVM 17
    compilerOptions {
        allWarningsAsErrors = true
        freeCompilerArgs.add("-Xjvm-default=all")
    }
}

intellijPlatform {
    pluginConfiguration {
        ideaVersion { sinceBuild = "241" }
    }
}

dependencies {
    intellijPlatform {
        intellijIdeaCommunity(libs.versions.idea.get())
        pluginVerifier()
        zipSigner()
        instrumentationTools()
    }
    implementation(libs.wasmRuntime)
    testImplementation(libs.kotlin.test)
    testImplementation(libs.junit)
}

// Embed the WASM binary into plugin resources
tasks.processResources {
    from("wasm/engine.wasm") { into("wasm") }
}
```

### `gradle/libs.versions.toml`

```toml
[versions]
kotlin            = "2.0.21"
intellijPlatform  = "2.1.0"
idea              = "2024.1"
wasmRuntime       = "1.1.0"      # pin exact version — no ranges

[libraries]
kotlin-test    = { module = "org.jetbrains.kotlin:kotlin-test" }
junit          = { module = "junit:junit", version = "4.13.2" }
wasmRuntime    = { module = "io.github.chicory:runtime", version.ref = "wasmRuntime" }

[plugins]
kotlin-jvm          = { id = "org.jetbrains.kotlin.jvm",        version.ref = "kotlin" }
intellijPlatform    = { id = "org.jetbrains.intellij.platform", version.ref = "intellijPlatform" }
```

---

## 8. Kotlin Language Rules

### 8.1 Immutability first
- Default to `val`. Any `var` must have a comment explaining why mutation is needed.
- Use immutable collections (`listOf`, `mapOf`) at all API boundaries.

### 8.2 Null safety
- No `!!` in production code. Use `?: error(...)`, `requireNotNull`, or `?.let`.
- Express optionality as `T?` or `Result<T>`; never use sentinel values (`-1`, `""`, etc.).

### 8.3 Coroutines
- Every `CoroutineScope` is tied to a lifecycle and cancelled on disposal.
- Use `supervisorScope` for parallel fan-out to isolate child failures.
- `GlobalScope` is forbidden.

### 8.4 Error handling
- Use-cases return `Result<T>` or a sealed outcome type.
- Adapters catch exceptions at the boundary and convert to domain outcomes — exceptions never leak into domain or application layers.

### 8.5 Extension functions
- Keep in a file named `<ReceiverType>Extensions.kt` in the same package as the receiver.
- Top-level utilities go in `<Topic>Utils.kt` — one topic per file.

---

## 9. `plugin.xml` Rules

- Declarations only — zero logic.
- Every `<service>` references the implementation class (interface where possible).
- Keep under 80 lines; split `<actions>` into `actions.xml` using `<xi:include>`.

```xml
<!-- resources/META-INF/plugin.xml -->
<idea-plugin>
    <id>com.example.plugin</id>
    <name>My WASM Plugin</name>
    <vendor>Example Corp</vendor>
    <depends>com.intellij.modules.platform</depends>

    <extensions defaultExtensionNs="com.intellij">
        <applicationService
            serviceImplementation="com.example.plugin.infrastructure.config.PluginSettings"/>
        <projectService
            serviceImplementation="com.example.plugin.adapter.ide.AnalysisService"/>
    </extensions>

    <xi:include href="actions.xml" xpointer="xpointer(/idea-plugin/*)"/>
</idea-plugin>
```

---

## 10. Testing Strategy

| Layer | Tool | Scope |
|---|---|---|
| `domain/` | `kotlin-test` | Pure unit tests; no mocks needed |
| `adapter/wasm/` | `kotlin-test` + real binary | `WasmLoader`, `WasmMemoryBridge` in isolation |
| `adapter/ide/` | IntelliJ test framework | Lightweight platform tests; mock use-cases at port boundary |
| Integration | IntelliJ test framework | One integration test per user-facing feature |

- Mock at **port boundaries** only — never inside the domain.
- Do not mock domain data classes — construct them directly.
- Test naming: `given_<context>_when_<action>_then_<expectation>`.

---

## 11. File Size & Decomposition Triggers

An agent **must** split a file when **any** of the following is true:

| Trigger | Required action |
|---|---|
| File exceeds **150 lines** | Extract cohesive block into a new named file |
| Class has more than **one public responsibility** | Create a sibling file for the second responsibility |
| Function body exceeds **30 lines** | Extract private functions or a helper object |
| WASM memory ops mixed with business logic | Move memory ops to `WasmMemoryBridge` |
| `com.intellij.*` import outside `adapter/ide/` | Move declaration to the correct IDE adapter file |
| WASM runtime type leaks outside `adapter/wasm/` | Wrap in a domain-safe type; keep runtime type internal |

---

*End of instruction file. AI agents must follow all rules above without exception unless the user explicitly overrides a specific section.*
