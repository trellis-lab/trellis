// editors/intellij/src/main/kotlin/com/trellis/plugin/ExportAction.kt
//
// Export action stub – PNG / SVG export is deferred to milestone Mc
// (commercial features / licence enforcement).
//
// When Mc is implemented, this action will:
//   1. Retrieve the last-rendered SVG from the PreviewPanel.
//   2. Check the licence tier (Free → PNG only; Premium → SVG + PNG).
//   3. Write the file via a file-chooser dialog.

package com.trellis.plugin

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.ui.Messages

/**
 * "Export Trellis Diagram…" action registered in plugin.xml.
 *
 * Currently shows a "coming soon" notice.  Full implementation is planned
 * for milestone Mc (commercial features).
 */
class ExportAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        Messages.showInfoMessage(
            e.project,
            "PNG/SVG export will be available in a future release (milestone Mc).",
            "Trellis Export – Coming Soon",
        )
    }

    override fun update(e: AnActionEvent) {
        val file = e.project?.let {
            com.intellij.openapi.fileEditor.FileEditorManager.getInstance(it)
                .selectedFiles.firstOrNull()
        }
        e.presentation.isEnabled = file?.extension?.lowercase() in setOf("mmd", "mermaid")
    }
}
