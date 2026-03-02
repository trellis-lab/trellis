// editors/intellij/build.gradle.kts
//
// IntelliJ Platform Plugin for Trellis – Mermaid diagram renderer.
// Requires the Gradle IntelliJ Plugin (org.jetbrains.intellij.platform).
//
// Build:   ./gradlew buildPlugin
// Run IDE: ./gradlew runIde
// Verify:  ./gradlew verifyPlugin

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.3.10"
    id("org.jetbrains.intellij.platform") version "2.11.0"
}

group = "com.trellis"
version = "0.1.0"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    // ── IntelliJ Platform ───────────────────────────────────────────────────
    intellijPlatform {
        // Target the latest stable IntelliJ IDEA Community (adjust as needed)
        intellijIdeaCommunity("2024.3")
        bundledPlugin("com.intellij.java")
        pluginVerifier()
        zipSigner()
    }

    // ── Chicory – pure-JVM WebAssembly runtime ──────────────────────────────
    // Used to execute the trellis_wasm.wasm binary without native code.
    implementation("com.dylibso.chicory:runtime:0.0.12")

    // ── Kotlin standard library ─────────────────────────────────────────────
    implementation(kotlin("stdlib"))
}

// ── Plugin configuration ────────────────────────────────────────────────────
intellijPlatform {
    pluginConfiguration {
        id = "com.trellis.plugin"
        name = "Trellis Mermaid Preview"
        version = "0.1.0"
        description = "Live preview for .mmd / .mermaid files using the Trellis renderer."
        changeNotes = "Initial release – IntelliJ JCEF preview with Chicory WASM bridge."

        ideaVersion {
            sinceBuild = "243"
            untilBuild = provider { null }
        }

        vendor {
            name = "Trellis"
            url = "https://github.com/trellis/trellis"
        }
    }

    signing {
        // Signing credentials are supplied via environment variables at release time:
        //   CERTIFICATE_CHAIN, PRIVATE_KEY, PRIVATE_KEY_PASSWORD
        certificateChain = providers.environmentVariable("CERTIFICATE_CHAIN")
        privateKey = providers.environmentVariable("PRIVATE_KEY")
        password = providers.environmentVariable("PRIVATE_KEY_PASSWORD")
    }

    publishing {
        token = providers.environmentVariable("PUBLISH_TOKEN")
    }
}

// ── Kotlin compiler options ─────────────────────────────────────────────────
kotlin {
    jvmToolchain(17)
}

tasks {
    // Copy the WASM artefacts built by scripts/build-wasm.sh into the plugin
    // resources so they are bundled inside the .zip distribution.
    processResources {
        // The WASM files land in src/main/resources/wasm/ after build-wasm.sh
        // runs; nothing extra to do here – they are already on the resource path.
    }
}
