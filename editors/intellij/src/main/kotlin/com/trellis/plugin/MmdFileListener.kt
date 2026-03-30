package com.trellis.plugin

import com.intellij.openapi.editor.EditorFactory
import com.intellij.openapi.editor.event.DocumentEvent
import com.intellij.openapi.editor.event.DocumentListener
import com.intellij.openapi.fileEditor.FileDocumentManager
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.FileEditorManagerListener
import com.intellij.openapi.project.Project
import com.intellij.openapi.startup.StartupActivity
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.vfs.VirtualFileManager
import com.intellij.openapi.vfs.newvfs.BulkFileListener
import com.intellij.openapi.vfs.newvfs.events.VFileEvent

/**
 * Registers listeners that keep the Trellis preview in sync with the editor.
 *
 * Implements [StartupActivity] so it is scoped to a project (via
 * `<postStartupActivity>` in plugin.xml) rather than the application.
 *
 * Two update triggers:
 *  - **VFS save events** – [BulkFileListener] fires when any .mmd file is saved to disk.
 *  - **Live typing** – a [DocumentListener] attached to .mmd documents updates the
 *    preview on every keystroke (respects the [TrellisSettings.autoPreview] flag).
 *  - **Editor switch** – [FileEditorManagerListener] re-renders when the user opens
 *    or switches to a .mmd file.
 */
class MmdFileListener : StartupActivity {

    override fun runActivity(project: Project) {
        val bus = project.messageBus

        // ── VFS save events ───────────────────────────────────────────────────
        bus.connect().subscribe(VirtualFileManager.VFS_CHANGES, object : BulkFileListener {
            override fun after(events: List<VFileEvent>) {
                if (!TrellisSettings.getInstance().state.autoPreview) return
                events
                    .filter { it.file?.isMmd() == true }
                    .forEach { event ->
                        val file = event.file ?: return@forEach
                        val doc = FileDocumentManager.getInstance().getDocument(file) ?: return@forEach
                        PreviewPanel.getInstance(project).update(doc.text)
                    }
            }
        })

        // ── Editor switch events ──────────────────────────────────────────────
        bus.connect().subscribe(
            FileEditorManagerListener.FILE_EDITOR_MANAGER,
            object : FileEditorManagerListener {
                override fun fileOpened(source: FileEditorManager, file: VirtualFile) {
                    if (!file.isMmd()) return
                    val doc = FileDocumentManager.getInstance().getDocument(file) ?: return
                    PreviewPanel.getInstance(project).update(doc.text)
                }
            },
        )

        // ── Live typing: document listener ────────────────────────────────────
        val disposable = PreviewPanel.getInstance(project)
        EditorFactory.getInstance().eventMulticaster.addDocumentListener(
            object : DocumentListener {
                override fun documentChanged(event: DocumentEvent) {
                    if (!TrellisSettings.getInstance().state.autoPreview) return
                    val file = FileDocumentManager.getInstance()
                        .getFile(event.document) ?: return
                    if (!file.isMmd()) return
                    // Only update if this document belongs to the current project
                    if (FileEditorManager.getInstance(project).openFiles.none { it == file }) return
                    PreviewPanel.getInstance(project).update(event.document.text)
                }
            },
            disposable,
        )
    }

    // ─────────────────────────────────────────────────────────────────────────

    private fun VirtualFile.isMmd(): Boolean =
        extension == "mmd" || extension == "mermaid"
}
