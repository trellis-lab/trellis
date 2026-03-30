package com.trellis.plugin

import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage

/**
 * Application-level persistent settings for the Trellis plugin.
 *
 * Stored in: `~/.config/JetBrains/<IDE>/options/TrellisSettings.xml`
 */
@Service(Service.Level.APP)
@State(name = "TrellisSettings", storages = [Storage("TrellisSettings.xml")])
class TrellisSettings : PersistentStateComponent<TrellisSettings.State> {

    data class State(
        /** Automatically re-render whenever a .mmd file is saved. */
        var autoPreview: Boolean = true,
        /** Default layout direction for flowcharts ("TB", "LR", "BT", "RL"). */
        var defaultDirection: String = "TB",
        /** Colour theme ("default", "dark", "neutral"). */
        var defaultTheme: String = "default",
    )

    private var myState = State()

    override fun getState(): State = myState

    override fun loadState(state: State) {
        myState = state
    }

    companion object {
        fun getInstance(): TrellisSettings =
            com.intellij.openapi.application.ApplicationManager
                .getApplication()
                .getService(TrellisSettings::class.java)
    }
}
