buildscript {
    repositories { mavenCentral() }
    dependencies { classpath("org.commonmark:commonmark:0.21.0") }
}

plugins {
    id("org.jetbrains.intellij.platform") version "2.2.0"
    kotlin("jvm") version "2.0.21"
}

group = providers.gradleProperty("pluginGroup").get()
version = providers.gradleProperty("pluginVersion").get()

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencies {
    intellijPlatform {
        rustRover(providers.gradleProperty("rustRoverVersion"))
        bundledPlugin("com.jetbrains.rust")
        pluginVerifier()
        instrumentationTools()
    }
    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.2")
    testImplementation("org.junit.jupiter:junit-jupiter-params:5.10.2")
}

fun markdownToHtml(file: File): String {
    val parser = org.commonmark.parser.Parser.builder().build()
    val renderer = org.commonmark.renderer.html.HtmlRenderer.builder().build()
    return renderer.render(parser.parse(file.readText()))
}

intellijPlatform {
    pluginConfiguration {
        name = providers.gradleProperty("pluginName")
        version = providers.gradleProperty("pluginVersion")

        description = providers.provider {
            markdownToHtml(layout.projectDirectory.file("README.md").asFile)
        }
        changeNotes = providers.provider {
            markdownToHtml(layout.projectDirectory.file("CHANGELOG.md").asFile)
        }

        vendor {
            name = providers.gradleProperty("pluginVendorName")
            email = providers.gradleProperty("pluginVendorEmail")
            url = providers.gradleProperty("pluginVendorUrl")
        }

        ideaVersion {
            sinceBuild = providers.gradleProperty("pluginSinceBuild")
            untilBuild = providers.gradleProperty("pluginUntilBuild")
        }
    }

    instrumentCode = true
}

kotlin {
    jvmToolchain(17)
}

tasks {
    test {
        useJUnitPlatform()
    }
}
