package com.trellis.plugin

import com.intellij.ide.AppLifecycleListener
import com.intellij.openapi.application.ApplicationManager

/**
 * AppLifecycleListener that pre-warms the WASM bridge on IDE startup.
 *
 * Extracting the WASM binary to a temp directory on a pooled thread means the
 * first preview render completes without any extra latency.
 */
class TrellisPlugin : AppLifecycleListener {
    override fun appStarted() {
        ApplicationManager.getApplication().executeOnPooledThread {
            WasmBridge.prepare()
        }
    }
}
