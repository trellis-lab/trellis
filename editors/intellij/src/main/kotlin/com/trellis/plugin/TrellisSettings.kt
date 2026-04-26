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
        /** Display labels on diagram edges. */
        var showEdgeLabels: Boolean = true,
        /** Display the diagram title above the diagram. */
        var showTitle: Boolean = true,
        /** How edges are drawn when they cross: None, Arc, Rectangular, Skip. */
        var crossingStyle: String = "Skip",
        /** Corner roundness of node shapes in pixels (0 = sharp corners). */
        var cornerRadius: Double = 8.0,
        /** Font size in pixels for edge labels. */
        var labelFontSize: Double = 10.0,
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
