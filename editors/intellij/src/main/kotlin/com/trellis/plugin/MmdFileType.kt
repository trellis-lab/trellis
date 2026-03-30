package com.trellis.plugin

import com.intellij.openapi.fileTypes.LanguageFileType
import com.intellij.openapi.util.IconLoader
import javax.swing.Icon

class MmdFileType private constructor() : LanguageFileType(MmdLanguage.INSTANCE) {

    override fun getName(): String = "Mermaid Diagram"

    override fun getDescription(): String = "Mermaid diagram file (.mmd, .mermaid)"

    override fun getDefaultExtension(): String = "mmd"

    override fun getIcon(): Icon = IconLoader.getIcon("/icons/mmd.svg", MmdFileType::class.java)

    companion object {
        @JvmField
        val INSTANCE = MmdFileType()
    }
}
