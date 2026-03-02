// editors/intellij/src/main/kotlin/com/trellis/plugin/PreviewPanel.kt
//
// JCEF-based preview tool window for Trellis .mmd diagrams.
//
// Architecture:
//   PreviewToolWindowFactory  – creates the tool window content on demand
//   PreviewPanel              – manages the singleton JCEF browser + JS bridge
//
// The JCEF browser loads preview.html (bundled in plugin resources/html/).
// Diagram rendering is done on the JVM side via WasmBridge (Chicory), and
// the resulting SVG string is pushed into the webview via executeJavaScript.

package com.trellis.plugin

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.diagnostic.thisLogger
import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.intellij.ui.jcef.JBCefBrowser
import com.intellij.ui.jcef.JBCefJSQuery
import org.cef.browser.CefBrowser
import org.cef.browser.CefFrame
import org.cef.handler.CefLoadHandlerAdapter
import javax.swing.JComponent
import javax.swing.JLabel
import javax.swing.JPanel
import java.awt.BorderLayout
import java.util.concurrent.atomic.AtomicBoolean

// ── Tool Window Factory ────────────────────────────────────────────────────────

/**
 * Registered in plugin.xml as `factoryClass` for the "Trellis Preview" tool window.
 * IntelliJ calls [createToolWindowContent] once per project when the tool window is
 * first opened.
 */
class PreviewToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val panel = PreviewPanel.getOrCreate(project)
        val content = ContentFactory.getInstance()
            .createContent(panel.component, "", false)
        toolWindow.contentManager.addContent(content)
    }

    override fun shouldBeAvailable(project: Project) = true
}

// ── Preview Panel ──────────────────────────────────────────────────────────────

/**
 * Singleton per-project preview panel backed by a JCEF browser.
 *
 * Call [update] from any thread to push a new Mermaid source string.
 */
class PreviewPanel private constructor(private val project: Project) {

    private val log = thisLogger()

    // ── JCEF browser ──────────────────────────────────────────────────────────

    private val browser: JBCefBrowser = JBCefBrowser()

    // JS query handler – used by the webview to send messages back to the JVM
    // (currently only used for error reporting; reserved for future use)
    private val jsQuery: JBCefJSQuery = JBCefJSQuery.create(browser)

    // Track whether the page has finished loading before we push SVG updates
    private val pageLoaded = AtomicBoolean(false)
    private var pendingSource: String? = null

    /** The Swing component to embed in the tool window. */
    val component: JComponent by lazy { buildComponent() }

    // ── Initialisation ────────────────────────────────────────────────────────

    init {
        // Register a JS→JVM callback so the webview can signal errors
        jsQuery.addHandler { msg ->
            log.warn("Trellis webview reported: $msg")
            null
        }

        // Detect when the HTML page has finished loading
        browser.jbCefClient.addLoadHandler(object : CefLoadHandlerAdapter() {
            override fun onLoadEnd(b: CefBrowser?, frame: CefFrame?, httpStatusCode: Int) {
                if (frame?.isMain == true) {
                    pageLoaded.set(true)
                    // Inject any update that arrived before page load finished
                    pendingSource?.let { src ->
                        pendingSource = null
                        renderOnBrowser(src)
                    }
                }
            }
        }, browser.cefBrowser)

        // Load the bundled preview HTML
        val htmlUrl = PreviewPanel::class.java.getResource("/html/preview.html")
            ?: error("Trellis: /html/preview.html not found in plugin resources.")
        browser.loadURL(htmlUrl.toExternalForm())
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /**
     * Push a new Mermaid [source] string.  Safe to call from any thread.
     * Rendering is offloaded to a background thread; SVG is injected into
     * the JCEF page on the EDT.
     */
    fun update(source: String) {
        if (!pageLoaded.get()) {
            pendingSource = source
            return
        }
        renderOnBrowser(source)
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    private fun buildComponent(): JComponent {
        return if (JBCefBrowser.isSupported()) {
            browser.component
        } else {
            // Fallback when JCEF is not available (e.g. headless IDE)
            JPanel(BorderLayout()).also {
                it.add(JLabel("Trellis preview requires JCEF support. " +
                    "Please use an IDE with built-in JCEF (IntelliJ IDEA, etc.)."), BorderLayout.CENTER)
            }
        }
    }

    /**
     * Render [source] via [WasmBridge] on a background thread, then inject
     * the resulting SVG into the JCEF page via JavaScript.
     */
    private fun renderOnBrowser(source: String) {
        ApplicationManager.getApplication().executeOnPooledThread {
            val js = try {
                val bridge = WasmBridge.instance()
                val resultJson = bridge.renderWithMetrics(source)
                // Parse the JSON minimally – we only need the "svg" field
                val svg = extractSvgFromJson(resultJson)
                val escaped = svg.escapeForJs()
                "window.trellisSetSvg('$escaped');"
            } catch (ex: WasmRenderException) {
                val msg = ex.message.orEmpty().escapeForJs()
                "window.trellisSetError('$msg');"
            } catch (ex: Exception) {
                log.error("Trellis render failed", ex)
                val msg = ex.message.orEmpty().escapeForJs()
                "window.trellisSetError('$msg');"
            }

            // Execute on the JCEF browser thread (can be called from any thread)
            browser.cefBrowser.executeJavaScript(js, browser.cefBrowser.url, 0)
        }
    }

    /** Minimal JSON field extraction – avoids pulling in a JSON library. */
    private fun extractSvgFromJson(json: String): String {
        // The trellis WASM output is: {"svg":"…","metrics":{…}}
        val prefix = "\"svg\":\""
        val start = json.indexOf(prefix)
        if (start < 0) return json          // fallback: treat entire string as SVG
        var i = start + prefix.length
        val sb = StringBuilder()
        while (i < json.length) {
            when {
                json[i] == '\\' && i + 1 < json.length -> {
                    when (json[i + 1]) {
                        '"' -> sb.append('"')
                        '\\' -> sb.append('\\')
                        'n' -> sb.append('\n')
                        'r' -> sb.append('\r')
                        't' -> sb.append('\t')
                        else -> sb.append(json[i + 1])
                    }
                    i += 2
                }
                json[i] == '"' -> break     // closing quote of the svg value
                else -> { sb.append(json[i]); i++ }
            }
        }
        return sb.toString()
    }

    /** Escape a string for safe embedding inside a JS single-quoted string literal. */
    private fun String.escapeForJs(): String =
        replace("\\", "\\\\")
            .replace("'", "\\'")
            .replace("\n", "\\n")
            .replace("\r", "\\r")

    // ── Companion ─────────────────────────────────────────────────────────────

    companion object {
        private val instances = mutableMapOf<Project, PreviewPanel>()

        fun getOrCreate(project: Project): PreviewPanel =
            instances.getOrPut(project) { PreviewPanel(project) }

        /** Called when a project closes; releases JCEF resources. */
        fun dispose(project: Project) {
            instances.remove(project)?.browser?.dispose()
        }

        /** Push new source to the panel for [project], if one exists. */
        fun update(project: Project, source: String) {
            instances[project]?.update(source)
        }
    }
}

// ── Open Preview Action ────────────────────────────────────────────────────────

/**
 * Action registered in plugin.xml.
 * Opens / reveals the Trellis Preview tool window and renders the active file.
 */
class OpenPreviewAction : com.intellij.openapi.actionSystem.AnAction() {
    override fun actionPerformed(e: com.intellij.openapi.actionSystem.AnActionEvent) {
        val project = e.project ?: return
        val editor = com.intellij.openapi.fileEditor.FileEditorManager.getInstance(project)
            .selectedTextEditor ?: return
        val source = editor.document.text
        // Reveal the tool window
        val twm = com.intellij.openapi.wm.ToolWindowManager.getInstance(project)
        twm.getToolWindow("Trellis Preview")?.apply {
            show()
            activate(null)
        }
        PreviewPanel.update(project, source)
    }

    override fun update(e: com.intellij.openapi.actionSystem.AnActionEvent) {
        val editor = e.project?.let {
            com.intellij.openapi.fileEditor.FileEditorManager.getInstance(it).selectedTextEditor
        }
        val file = editor?.document?.let {
            com.intellij.openapi.fileEditor.FileDocumentManager.getInstance().getFile(it)
        }
        e.presentation.isEnabled = file?.extension?.lowercase() in setOf("mmd", "mermaid")
    }
}
