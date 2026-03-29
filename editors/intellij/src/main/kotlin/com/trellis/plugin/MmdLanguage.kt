package com.trellis.plugin

import com.intellij.lang.Language

class MmdLanguage private constructor() : Language("Mermaid") {
    companion object {
        @JvmField
        val INSTANCE = MmdLanguage()
    }
}
