package com.trellis.plugin

import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.jcef.JBCefApp
import com.intellij.ui.jcef.JBCefBrowser
import com.intellij.ui.jcef.JBCefBrowserBase
import com.intellij.ui.jcef.JBCefJSQuery
import org.cef.browser.CefBrowser
import org.cef.browser.CefFrame
import org.cef.handler.CefLoadHandlerAdapter
import java.io.IOException
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import javax.swing.JComponent
import javax.swing.JLabel
import javax.swing.SwingConstants

private val log = logger<PreviewPanel>()

/**
 * Project-level service that owns the JCEF browser used for diagram preview.
 *
 * JCEF (JetBrains Chromium Embedded Framework) runs a full Chromium renderer
 * which supports ES modules and the WebAssembly API, allowing the wasm-bindgen
 * generated `trellis_wasm.js` glue to work unmodified inside the browser page.
 *
 * Lifecycle:
 *  1. [PreviewToolWindowFactory] retrieves this service and embeds [component] in the tool window.
 *  2. [MmdFileListener] calls [update] whenever a .mmd document changes.
 *  3. On first [update] call, WASM resources are extracted and the browser is initialised.
 */
@Service(Service.Level.PROJECT)
class PreviewPanel(private val project: Project) : Disposable {

    // ── Component exposed to the tool window ──────────────────────────────────

    val component: JComponent = buildComponent()

    private var browser: JBCefBrowser? = null
    private var wasmReady = false
    private var pendingSource: String? = null
    private var pendingConfig: String? = null

    // ─────────────────────────────────────────────────────────────────────────

    private fun buildComponent(): JComponent {
        if (!JBCefApp.isSupported()) {
            log.warn("Trellis: JCEF is not available in this IDE build; preview disabled.")
            return JLabel(
                "<html><center>Trellis Preview requires JCEF.<br>" +
                        "Please use a JetBrains Runtime (JBR) based IDE.</center></html>",
                SwingConstants.CENTER,
            )
        }

        val b = JBCefBrowser()
        browser = b

        // After every page load (including the initial load of preview.html),
        // inject the WASM module URIs so the page can initialise the renderer.
        b.jbCefClient.addLoadHandler(object : CefLoadHandlerAdapter() {
            override fun onLoadEnd(browser: CefBrowser, frame: CefFrame, httpStatusCode: Int) {
                if (!frame.isMain) return
                initWasm()
            }
        }, b.cefBrowser)

        return b.component
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /**
     * Render a new Mermaid source string in the preview.
     *
     * If the browser page is not yet ready (WASM not initialised) the source is
     * queued and rendered as soon as initialisation completes.
     */
    fun update(source: String) {
        val configJson = WasmBridge.buildConfigJson()

        // Ensure the page is loaded
        ensurePageLoaded()

        if (wasmReady) {
            pushUpdate(source, configJson)
        } else {
            pendingSource = source
            pendingConfig = configJson
        }
    }

    override fun dispose() {
        browser?.dispose()
        browser = null
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    private var pageLoaded = false

    private fun ensurePageLoaded() {
        if (pageLoaded) return
        val b = browser ?: return

        val previewPath = extractPreviewHtml() ?: run {
            log.warn("Trellis: Could not extract preview.html; preview unavailable.")
            return
        }
        b.loadURL(previewPath.toUri().toString())
        pageLoaded = true
    }

    /** Send the `trellisInit` call to the page with the WASM file URIs. */
    private fun initWasm() {
        val b = browser ?: return
        val moduleUri = WasmBridge.wasmModuleUri()
        val binaryUri = WasmBridge.wasmBinaryUri()

        if (moduleUri == null || binaryUri == null) {
            // Resources not yet extracted – try now (blocking, but on a render thread)
            ApplicationManager.getApplication().executeOnPooledThread {
                WasmBridge.prepare()
                b.cefBrowser.executeJavaScript(
                    initJs(WasmBridge.wasmModuleUri() ?: return@executeOnPooledThread,
                           WasmBridge.wasmBinaryUri() ?: return@executeOnPooledThread),
                    b.cefBrowser.url, 0,
                )
                wasmReady = true
                flushPending()
            }
            return
        }

        b.cefBrowser.executeJavaScript(initJs(moduleUri, binaryUri), b.cefBrowser.url, 0)
        wasmReady = true
        flushPending()
    }

    private fun flushPending() {
        val src = pendingSource ?: return
        pushUpdate(src, pendingConfig ?: "")
        pendingSource = null
        pendingConfig = null
    }

    private fun pushUpdate(source: String, configJson: String) {
        val b = browser ?: return
        val escapedSource = source
            .replace("\\", "\\\\")
            .replace("'", "\\'")
            .replace("\n", "\\n")
            .replace("\r", "")
        val escapedConfig = configJson
            .replace("\\", "\\\\")
            .replace("'", "\\'")
        b.cefBrowser.executeJavaScript(
            "window.trellisUpdate('$escapedSource', '$escapedConfig');",
            b.cefBrowser.url, 0,
        )
    }

    private fun initJs(moduleUri: String, binaryUri: String): String =
        "window.trellisInit('$moduleUri', '$binaryUri');"

    /**
     * Extract `html/preview.html` from the plugin resources to the same temp
     * directory as the WASM files so that relative ES module imports resolve.
     */
    private fun extractPreviewHtml(): Path? {
        val wasmDir = WasmBridge.prepare() ?: return null
        val dest = wasmDir.resolve("preview.html")
        if (dest.toFile().exists()) return dest

        val stream = PreviewPanel::class.java.getResourceAsStream("/html/preview.html")
            ?: return null
        return try {
            stream.use { Files.copy(it, dest, StandardCopyOption.REPLACE_EXISTING) }
            dest
        } catch (e: IOException) {
            log.error("Trellis: Failed to extract preview.html", e)
            null
        }
    }

    companion object {
        fun getInstance(project: Project): PreviewPanel =
            project.getService(PreviewPanel::class.java)
    }
}

// ── Tool window factory ───────────────────────────────────────────────────────

/**
 * Registered in plugin.xml as the factory for the "Trellis Preview" tool window.
 * Called once per project when the tool window is first shown.
 */
class PreviewToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val panel = PreviewPanel.getInstance(project)
        val content = ContentFactory.getInstance()
            .createContent(panel.component, "", false)
        toolWindow.contentManager.addContent(content)
    }

    override fun shouldBeAvailable(project: Project): Boolean = true
}
