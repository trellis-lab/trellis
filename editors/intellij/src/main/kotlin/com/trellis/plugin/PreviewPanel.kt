package com.trellis.plugin

import com.intellij.openapi.Disposable
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.logger
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.jcef.JBCefApp
import com.intellij.ui.jcef.JBCefBrowser
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
    private var pageReady = false
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

        // After the page finishes loading, flush any pending diagram update.
        // The page auto-initializes WASM via relative ES module imports served
        // over HTTP — no file:// URIs needed.
        b.jbCefClient.addLoadHandler(object : CefLoadHandlerAdapter() {
            override fun onLoadEnd(browser: CefBrowser, frame: CefFrame, httpStatusCode: Int) {
                if (!frame.isMain) return
                pageReady = true
                flushPending()
            }
        }, b.cefBrowser)

        return b.component
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /**
     * Render a new Mermaid source string in the preview.
     *
     * If the browser page is not yet ready the source is queued and rendered
     * as soon as the page finishes loading. The page itself queues the update
     * internally until its WASM module has initialised.
     */
    fun update(source: String) {
        val configJson = WasmBridge.buildConfigJson()
        ensurePageLoaded()

        if (pageReady) {
            pushUpdate(source, configJson)
        } else {
            pendingSource = source
            pendingConfig = configJson
        }
    }

    override fun dispose() {
        browser?.dispose()
        browser = null
        WasmBridge.shutdown()
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    private var pageLoaded = false

    private fun ensurePageLoaded() {
        if (pageLoaded) return
        val b = browser ?: return

        extractPreviewHtml() ?: run {
            log.warn("Trellis: Could not extract preview.html; preview unavailable.")
            return
        }
        val baseUrl = WasmBridge.serverBaseUrl() ?: run {
            log.warn("Trellis: Local resource server not available.")
            return
        }
        b.loadURL("$baseUrl/preview.html")
        pageLoaded = true
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

    /**
     * Extract `html/preview.html` from the plugin resources to the same temp
     * directory as the WASM files so that the local HTTP server can serve it.
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
