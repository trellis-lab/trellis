package com.trellis.plugin

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.ui.Messages

/**
 * Stub action for PNG/SVG export.
 *
 * Full implementation is deferred to the commercial milestone (M19).
 * For now, the action is registered so the menu item is visible and shows
 * a "coming soon" notice rather than being absent or crashing.
 */
class ExportAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        Messages.showInfoMessage(
            e.project,
            "Diagram export (PNG/SVG) will be available in a future release.",
            "Trellis: Export Coming Soon",
        )
    }

    override fun update(e: AnActionEvent) {
        // Only show the action when a .mmd file is active
        val file = e.project?.let {
            com.intellij.openapi.fileEditor.FileEditorManager.getInstance(it).selectedFiles.firstOrNull()
        }
        e.presentation.isEnabledAndVisible = file?.extension == "mmd" || file?.extension == "mermaid"
    }
}
