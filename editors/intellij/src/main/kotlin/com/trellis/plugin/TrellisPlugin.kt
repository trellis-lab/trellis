// editors/intellij/src/main/kotlin/com/trellis/plugin/TrellisPlugin.kt
//
// Application lifecycle listener – performs one-time startup work
// (e.g. warming up the WasmBridge on a background thread).

package com.trellis.plugin

import com.intellij.ide.AppLifecycleListener
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.diagnostic.thisLogger

/**
 * Entry point registered in plugin.xml as an [AppLifecycleListener].
 *
 * Responsible for:
 * - Logging plugin startup.
 * - (Optionally) pre-loading the WASM module on a background thread so that
 *   the first preview opens without delay.
 */
class TrellisPlugin : AppLifecycleListener {

    private val log = thisLogger()

    override fun appStarted() {
        log.info("Trellis plugin started (v${PLUGIN_VERSION})")

        // Pre-warm the WASM bridge on a pooled background thread.
        // If WASM loading fails here, individual preview panels will retry.
        ApplicationManager.getApplication().executeOnPooledThread {
            try {
                WasmBridge.instance().warmUp()
                log.info("Trellis WASM bridge pre-warmed successfully.")
            } catch (ex: Exception) {
                log.warn("Trellis WASM bridge pre-warm failed (will retry on first preview): ${ex.message}")
            }
        }
    }

    companion object {
        const val PLUGIN_VERSION = "0.1.0"
        const val PLUGIN_ID = "com.trellis.plugin"
    }
}
