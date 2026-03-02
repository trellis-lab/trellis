// editors/intellij/src/main/kotlin/com/trellis/plugin/WasmBridge.kt
//
// Pure-JVM WASM bridge using the Chicory runtime.
// Loads trellis_wasm_bg.wasm from the plugin resources and exposes
// `render(source)` / `renderWithMetrics(source, configJson)` methods.
//
// Chicory executes the WASM binary without any native JNI — fully portable
// across all JVM platforms (macOS, Linux, Windows) with no extra installation.

package com.trellis.plugin

import com.dylibso.chicory.runtime.ExportFunction
import com.dylibso.chicory.runtime.Instance
import com.dylibso.chicory.runtime.Module
import com.dylibso.chicory.wasm.Parser
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.thisLogger
import java.nio.charset.StandardCharsets

/**
 * Project-level service that owns the Chicory [Instance] lifetime.
 *
 * Use [WasmBridge.instance] to obtain the application-wide singleton.
 * Call [render] or [renderWithMetrics] from any thread.
 */
@Service
class WasmBridge {

    private val log = thisLogger()

    // ── Chicory instance (lazy – loaded on first use) ─────────────────────────

    @Volatile
    private var _instance: Instance? = null
    private val lock = Object()

    // Exported WASM function handles (initialised alongside _instance)
    private var fnAlloc: ExportFunction? = null
    private var fnDealloc: ExportFunction? = null
    private var fnRender: ExportFunction? = null
    private var fnRenderWithMetrics: ExportFunction? = null

    // ── Public API ────────────────────────────────────────────────────────────

    /**
     * Pre-warm: load and instantiate the WASM module.
     * Safe to call from a background thread.
     */
    fun warmUp() {
        getOrCreateInstance()
    }

    /**
     * Render a Mermaid [source] string and return SVG.
     *
     * @throws WasmRenderException on parse/render failure inside WASM.
     */
    fun render(source: String): String {
        return callWasm("render", source, null)
    }

    /**
     * Render with metrics; returns JSON: `{"svg":"…","metrics":{…}}`.
     *
     * @throws WasmRenderException on parse/render failure inside WASM.
     */
    fun renderWithMetrics(source: String, configJson: String? = null): String {
        return callWasm("render_with_metrics", source, configJson)
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    private fun getOrCreateInstance(): Instance {
        _instance?.let { return it }
        synchronized(lock) {
            _instance?.let { return it }

            log.info("Loading trellis_wasm_bg.wasm via Chicory…")
            val wasmBytes = loadWasmBytes()
            val module = Module.builder(Parser.parse(wasmBytes)).build()
            val inst = module.instantiate()

            // Resolve exported function handles
            fnAlloc = inst.export("__wbindgen_malloc")
            fnDealloc = inst.export("__wbindgen_free")
            fnRender = inst.export("render")
            fnRenderWithMetrics = inst.export("render_with_metrics")

            _instance = inst
            log.info("Chicory WASM instance ready.")
            return inst
        }
    }

    /**
     * Load the WASM binary from plugin resources.
     *
     * The file is placed at `src/main/resources/wasm/trellis_wasm_bg.wasm`
     * by `scripts/build-wasm.sh --bundler` and bundled into the plugin .zip.
     */
    private fun loadWasmBytes(): ByteArray {
        val resourcePath = "/wasm/trellis_wasm_bg.wasm"
        val stream = WasmBridge::class.java.getResourceAsStream(resourcePath)
            ?: error("Trellis: WASM binary not found at $resourcePath. Run scripts/build-wasm.sh first.")
        return stream.use { it.readBytes() }
    }

    /**
     * Call a wasm-bindgen–generated string-in / string-out exported function.
     *
     * wasm-bindgen's ABI for string arguments:
     *  1. Allocate memory in the WASM heap via `__wbindgen_malloc(len, 1)`.
     *  2. Write UTF-8 bytes into the returned pointer.
     *  3. Call the function with (ptr, len, [ptr2, len2], retptr).
     *  4. Read the result from `retptr` (two i32 words: ptr + len).
     *  5. Read the string, then free with `__wbindgen_free(ptr, len, 1)`.
     *
     * This implements the minimal subset needed for our two exported functions.
     */
    private fun callWasm(fnName: String, source: String, configJson: String?): String {
        val inst = getOrCreateInstance()
        val memory = inst.memory()

        val alloc = fnAlloc ?: error("__wbindgen_malloc not found")
        val dealloc = fnDealloc ?: error("__wbindgen_free not found")
        val fn = when (fnName) {
            "render" -> fnRender ?: error("render not exported")
            "render_with_metrics" -> fnRenderWithMetrics ?: error("render_with_metrics not exported")
            else -> error("Unknown function: $fnName")
        }

        val srcBytes = source.toByteArray(StandardCharsets.UTF_8)
        val cfgBytes = configJson?.toByteArray(StandardCharsets.UTF_8)

        // Allocate return-value slot (2 × i32 = 8 bytes) on the WASM heap
        val retPtr = alloc.apply(8L, 1L)[0].toInt()

        val srcPtr = allocString(inst, alloc, srcBytes)
        try {
            if (cfgBytes != null) {
                val cfgPtr = allocString(inst, alloc, cfgBytes)
                try {
                    fn.apply(retPtr.toLong(), srcPtr.toLong(), srcBytes.size.toLong(),
                              cfgPtr.toLong(), cfgBytes.size.toLong())
                } finally {
                    dealloc.apply(cfgPtr.toLong(), cfgBytes.size.toLong(), 1L)
                }
            } else {
                // Pass a null pointer + 0 length for the optional config
                fn.apply(retPtr.toLong(), srcPtr.toLong(), srcBytes.size.toLong(), 0L, 0L)
            }
        } finally {
            dealloc.apply(srcPtr.toLong(), srcBytes.size.toLong(), 1L)
        }

        // Read result (ptr, len) written by wasm-bindgen at retPtr
        val resultPtr = memory.readI32(retPtr)
        val resultLen = memory.readI32(retPtr + 4)
        val resultBytes = memory.readBytes(resultPtr, resultLen)
        // Free the result buffer and the retPtr slot
        dealloc.apply(resultPtr.toLong(), resultLen.toLong(), 1L)
        dealloc.apply(retPtr.toLong(), 8L, 1L)

        val result = String(resultBytes, StandardCharsets.UTF_8)

        // wasm-bindgen signals errors by returning JSON {"error":"…"}
        if (result.startsWith("{\"error\"")) {
            throw WasmRenderException(result)
        }
        return result
    }

    /** Allocate [bytes] on the WASM heap, write them, return the pointer. */
    private fun allocString(inst: Instance, alloc: ExportFunction, bytes: ByteArray): Int {
        val ptr = alloc.apply(bytes.size.toLong(), 1L)[0].toInt()
        inst.memory().write(ptr, bytes)
        return ptr
    }

    // ── Companion ─────────────────────────────────────────────────────────────

    companion object {
        /** Obtain the application-wide [WasmBridge] service instance. */
        fun instance(): WasmBridge =
            com.intellij.openapi.components.service<WasmBridge>()
    }
}

/** Thrown when the WASM module reports a parse or render error. */
class WasmRenderException(message: String) : RuntimeException(message)
