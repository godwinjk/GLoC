package com.gloc.plugin

import com.intellij.openapi.editor.EditorFactory
import com.intellij.openapi.fileTypes.FileTypeManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.DialogWrapper
import com.intellij.openapi.ui.ValidationInfo
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.ui.EditorTextField
import com.intellij.ui.dsl.builder.AlignX
import com.intellij.ui.dsl.builder.bind
import com.intellij.ui.dsl.builder.bindText
import com.intellij.ui.dsl.builder.panel
import javax.swing.JComponent

/// Dialog for creating a new GLoC reactor file using JetBrains UI DSL.
class NewReactorDialog(
    private val project: Project,
    private val targetDir: VirtualFile,
) : DialogWrapper(project) {

    private var name: String = ""
    private var withNeutrons: Boolean = false

    private val previewEditor = EditorTextField(
        EditorFactory.getInstance().createDocument(""),
        project,
        FileTypeManager.getInstance().getFileTypeByExtension("rs"),
        true,
        false,
    ).apply {
        preferredSize = java.awt.Dimension(600, 380)
    }

    val reactorName: String get() = name
    val includeNeutrons: Boolean get() = withNeutrons

    init {
        title = "New GLoC Reactor"
        init()
    }

    override fun createCenterPanel(): JComponent = panel {
        row("Reactor Name:") {
            textField()
                .bindText(::name)
                .applyToComponent { emptyText.text = "e.g. Counter" }
                .align(AlignX.FILL)
                .focused()
                .onChanged { name = it.text; updatePreview() }
        }

        buttonsGroup("Reactor Type:") {
            row {
                radioButton("Without Neutrons", false)
                    .applyToComponent { addActionListener { withNeutrons = false; updatePreview() } }
            }
            row {
                radioButton("With Neutrons", true)
                    .applyToComponent { addActionListener { withNeutrons = true; updatePreview() } }
            }
        }.bind(
            getter = { withNeutrons },
            setter = { withNeutrons = it },
        )

        group("Preview") {
            row {
                cell(previewEditor).align(AlignX.FILL)
            }
        }

        updatePreview()
    }

    override fun doValidate(): ValidationInfo? {
        val error = validateReactorName(name)
        if (error != null) return ValidationInfo(error)
        val fileName = ReactorGenerator.toFileName(name)
        if (targetDir.findChild("$fileName.rs") != null) {
            return ValidationInfo("A file named '$fileName.rs' already exists in the selected directory.")
        }
        return null
    }

    private fun updatePreview() {
        val displayName = name.ifEmpty { "MyReactor" }
        previewEditor.text = ReactorGenerator.generateContent(displayName, withNeutrons)
    }
}
