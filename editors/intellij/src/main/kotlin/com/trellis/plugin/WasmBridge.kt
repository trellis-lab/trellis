package com.trellis.plugin

import com.intellij.openapi.diagnostic.logger
import com.sun.net.httpserver.HttpServer
import java.io.IOException
import java.net.InetAddress
import java.net.InetSocketAddress
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption

/**
 * WasmBridge manages the lifecycle of the Trellis WASM resources.
 *
 * The plugin bundles the wasm-pack `--target web` output (trellis_wasm.js +
 * trellis_wasm_bg.wasm) inside the plugin JAR under `/wasm/`.  On first use,
 * those resources are extracted to a temporary directory and served via a
 * local HTTP server so JCEF's Chromium can load ES modules and WASM binaries
 * without `file://` CORS restrictions.
 *
 * Architecture:
 *   Plugin JAR (resources/wasm/) ──extract──> temp dir ──http://127.0.0.1──> JCEF page
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

    private var server: HttpServer? = null

    @Volatile
    private var serverPort: Int = 0

    /**
     * Extract WASM resources to a temp directory and start a local HTTP server
     * (idempotent – safe to call repeatedly).
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
                startServer(tmpDir)
                wasmDir = tmpDir
                tmpDir
            } catch (e: IOException) {
                log.error("Trellis: Failed to extract WASM resources", e)
                null
            }
        }
    }

    /** Base URL of the local HTTP server, or `null` if not started. */
    fun serverBaseUrl(): String? =
        if (serverPort > 0) "http://127.0.0.1:$serverPort" else null

    /** Build a minimal TrellisConfig JSON from settings (empty string = use Rust defaults). */
    fun buildConfigJson(): String {
        val settings = TrellisSettings.getInstance().state
        val parts = mutableListOf<String>()
        if (settings.defaultDirection != "TB") {
            parts += "\"direction\":\"${settings.defaultDirection}\""
        }
        return if (parts.isEmpty()) "" else "{${parts.joinToString(",")}}"
    }

    /** Stop the local HTTP server. */
    fun shutdown() {
        server?.stop(0)
        server = null
        serverPort = 0
    }

    private fun startServer(dir: Path) {
        if (server != null) return
        val httpServer = HttpServer.create(
            InetSocketAddress(InetAddress.getLoopbackAddress(), 0), 0,
        )
        httpServer.createContext("/") { exchange ->
            val path = exchange.requestURI.path.trimStart('/')
            val file = dir.resolve(path)
            if (path.isEmpty() || !file.startsWith(dir) || !file.toFile().exists()) {
                exchange.sendResponseHeaders(404, -1)
                exchange.close()
                return@createContext
            }
            val bytes = Files.readAllBytes(file)
            exchange.responseHeaders["Content-Type"] = listOf(mimeType(path))
            exchange.sendResponseHeaders(200, bytes.size.toLong())
            exchange.responseBody.use { it.write(bytes) }
        }
        httpServer.start()
        server = httpServer
        serverPort = httpServer.address.port
        log.info("Trellis: local resource server started on port $serverPort")
    }

    private fun mimeType(path: String): String = when {
        path.endsWith(".html") -> "text/html; charset=utf-8"
        path.endsWith(".js") -> "application/javascript"
        path.endsWith(".wasm") -> "application/wasm"
        path.endsWith(".json") -> "application/json"
        else -> "application/octet-stream"
    }
}
