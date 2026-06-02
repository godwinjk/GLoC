plugins {
    id("org.jetbrains.intellij") version "1.17.2"
    kotlin("jvm") version "1.9.22"
}

group = "com.gloc"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
}

intellij {
    version.set("2023.3")
    type.set("IC")
}

tasks {
    patchPluginXml {
        sinceBuild.set("233")
        untilBuild.set("243.*")
    }

    compileKotlin {
        kotlinOptions.jvmTarget = "17"
    }

    compileTestKotlin {
        kotlinOptions.jvmTarget = "17"
    }

    test {
        useJUnitPlatform()
    }
}
