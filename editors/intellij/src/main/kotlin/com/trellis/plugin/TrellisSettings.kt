// editors/intellij/src/main/kotlin/com/trellis/plugin/TrellisSettings.kt
//
// Application-level persistent settings for the Trellis plugin.
// Stored in the IDE's standard settings directory (XML via PersistentStateComponent).

package com.trellis.plugin

import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.RoamingType
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.openapi.components.service
import com.intellij.util.xmlb.XmlSerializerUtil

/**
 * Persisted settings for the Trellis plugin.
 *
 * Accessed via:
 *   `TrellisSettings.instance().autoPreview`
 */
@State(
    name = "TrellisSettings",
    storages = [Storage("trellis.xml", roamingType = RoamingType.DISABLED)],
)
class TrellisSettings : PersistentStateComponent<TrellisSettings> {

    /** Auto-update the preview panel whenever a .mmd file is saved / edited. */
    var autoPreview: Boolean = true

    /** Default diagram direction (TB / BT / LR / RL). */
    var defaultDirection: String = "TB"

    /** Default theme colour scheme (mermaid compatible names). */
    var defaultTheme: String = "default"

    override fun getState(): TrellisSettings = this

    override fun loadState(state: TrellisSettings) {
        XmlSerializerUtil.copyBean(state, this)
    }

    companion object {
        fun instance(): TrellisSettings = service()
    }
}
