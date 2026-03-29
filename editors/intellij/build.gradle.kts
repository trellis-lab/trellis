// build.gradle.kts – Trellis IntelliJ Plugin
// Gradle IntelliJ Platform Plugin v2.2.1
// Targets: IntelliJ IDEA Community 2024.3 (build 243)

plugins {
    id("org.jetbrains.intellij.platform") version "2.2.1"
    kotlin("jvm") version "1.9.25"
}

group = "com.trellis"
version = "0.1.0"

kotlin {
    jvmToolchain(21)
}

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        intellijIdeaCommunity("2024.3")
    }
}

intellijPlatform {
    pluginConfiguration {
        id = "com.trellis.plugin"
        name = "Trellis Mermaid Preview"
        version = "0.1.0"
        description = """
            Live preview for Mermaid .mmd diagram files using the Trellis VLSI-routing renderer.
            Opens a tool window that re-renders the diagram on every file save.
        """.trimIndent()

        ideaVersion {
            sinceBuild = "243"
            untilBuild = provider { null }
        }
    }
}
