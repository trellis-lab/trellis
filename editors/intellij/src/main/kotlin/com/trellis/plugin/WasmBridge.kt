package com.trellis.plugin

import com.intellij.openapi.diagnostic.logger
import java.io.IOException
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption

/**
 * WasmBridge manages the lifecycle of the Trellis WASM resources.
 *
 * The plugin bundles the wasm-pack `--target web` output (trellis_wasm.js +
 * trellis_wasm_bg.wasm) inside the plugin JAR under `/wasm/`.  On first use,
 * those resources are extracted to a temporary directory so JCEF's Chromium
 * renderer can load them as local `file://` URIs.
 *
 * Architecture:
 *   Plugin JAR (resources/wasm/) ──extract──> temp dir ──file://──> JCEF page
 *
 * JCEF runs a full Chromium engine that supports ES modules and the
 * WebAssembly API, so the wasm-bindgen generated JS glue works without any
 * JVM-side WASM interpreter.
 */
object WasmBridge {

    private val log = logger<WasmBridge>()

    /** Files that must be present in the plugin's `/wasm/` resource directory. */
    private val WASM_RESOURCES = listOf(
        "trellis_wasm.js",
        "trellis_wasm_bg.wasm",
    )

    @Volatile
    private var wasmDir: Path? = null

    /**
     * Extract WASM resources to a temp directory (idempotent – safe to call repeatedly).
     *
     * Returns the temp directory path, or `null` if the resources are missing
     * (e.g. `build-wasm.sh` has not been run yet).
     */
    fun prepare(): Path? {
        wasmDir?.let { return it }

        return synchronized(this) {
            wasmDir?.let { return it }

            try {
                val tmpDir = Files.createTempDirectory("trellis-wasm")
                tmpDir.toFile().deleteOnExit()

                for (name in WASM_RESOURCES) {
                    val stream = WasmBridge::class.java.getResourceAsStream("/wasm/$name")
                    if (stream == null) {
                        log.warn("Trellis: WASM resource /wasm/$name not found. " +
                                "Run ./scripts/build-wasm.sh to build the WASM binary.")
                        return null
                    }
                    stream.use {
                        Files.copy(it, tmpDir.resolve(name), StandardCopyOption.REPLACE_EXISTING)
                    }
                }

                log.info("Trellis: WASM resources extracted to $tmpDir")
                wasmDir = tmpDir
                tmpDir
            } catch (e: IOException) {
                log.error("Trellis: Failed to extract WASM resources", e)
                null
            }
        }
    }

    /** `file://` URI for the wasm-bindgen ES module entry point, or `null` if not ready. */
    fun wasmModuleUri(): String? = wasmDir?.resolve("trellis_wasm.js")?.toUri()?.toString()

    /** `file://` URI for the raw WASM binary, or `null` if not ready. */
    fun wasmBinaryUri(): String? = wasmDir?.resolve("trellis_wasm_bg.wasm")?.toUri()?.toString()

    /** Build a minimal TrellisConfig JSON from settings (empty string = use Rust defaults). */
    fun buildConfigJson(): String {
        val settings = TrellisSettings.getInstance().state
        val parts = mutableListOf<String>()
        if (settings.defaultDirection != "TB") {
            parts += "\"direction\":\"${settings.defaultDirection}\""
        }
        return if (parts.isEmpty()) "" else "{${parts.joinToString(",")}}"
    }
}
