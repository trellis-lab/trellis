// editors/intellij/src/main/kotlin/com/trellis/plugin/MmdFileListener.kt
//
// VFS (Virtual File System) listener that watches for changes to .mmd /
// .mermaid files and triggers a preview update.
//
// Registered in plugin.xml as a projectListener for BulkFileListener.
// IntelliJ fires this on every VFS event (save, external change, etc.).

package com.trellis.plugin

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.diagnostic.thisLogger
import com.intellij.openapi.editor.EditorFactory
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.newvfs.BulkFileListener
import com.intellij.openapi.vfs.newvfs.events.VFileContentChangeEvent
import com.intellij.openapi.vfs.newvfs.events.VFileEvent

/**
 * Listens for VFS content-change events and forwards Mermaid file changes
 * to [PreviewPanel].
 *
 * The listener is registered per-project via plugin.xml so `project` is
 * injected at construction time.
 */
@Service
class MmdFileListener(private val project: Project) : BulkFileListener {

    private val log = thisLogger()

    override fun after(events: List<VFileEvent>) {
        for (event in events) {
            if (event !is VFileContentChangeEvent) continue
            val file = event.file
            if (!isMermaidFile(file.extension)) continue

            log.debug("Trellis: detected change in ${file.name}")

            // Read the document content on the EDT (required by IntelliJ API)
            ApplicationManager.getApplication().invokeLater {
                val document = FileDocumentManager.getInstance().getDocument(file) ?: return@invokeLater
                val source = document.text
                PreviewPanel.update(project, source)
            }
        }
    }

    // ── Editor document listener (catches unsaved in-memory edits) ────────────

    /**
     * Also watch the currently-active editor document for live (unsaved)
     * changes so the preview updates as the user types.
     *
     * Called from [TrellisPlugin.appStarted] after the project opens.
     */
    fun installDocumentListener() {
        val editorManager = FileEditorManager.getInstance(project)
        EditorFactory.getInstance().eventMulticaster.addDocumentListener(
            object : com.intellij.openapi.editor.event.DocumentListener {
                override fun documentChanged(event: com.intellij.openapi.editor.event.DocumentChangeEvent) {
                    val file = FileDocumentManager.getInstance()
                        .getFile(event.document) ?: return
                    if (!isMermaidFile(file.extension)) return
                    // Debounce: only update if this file is actually open in an editor
                    val isOpen = editorManager.isFileOpen(file)
                    if (!isOpen) return
                    PreviewPanel.update(project, event.document.text)
                }
            },
            // Dispose together with the project service
            project,
        )
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    companion object {
        private val MERMAID_EXTENSIONS = setOf("mmd", "mermaid")

        fun isMermaidFile(extension: String?): Boolean =
            extension?.lowercase() in MERMAID_EXTENSIONS
    }
}
