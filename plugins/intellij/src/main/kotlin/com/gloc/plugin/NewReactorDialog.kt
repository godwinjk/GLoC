package com.gloc.plugin

import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.DialogWrapper
import com.intellij.openapi.ui.ValidationInfo
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.ui.JBColor
import com.intellij.ui.components.JBLabel
import com.intellij.ui.components.JBScrollPane
import com.intellij.ui.components.JBTextField
import com.intellij.util.ui.JBUI
import java.awt.BorderLayout
import java.awt.Font
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import javax.swing.*
import javax.swing.event.DocumentEvent
import javax.swing.event.DocumentListener

private val PASCAL_CASE = Regex("^[A-Z][a-zA-Z0-9]*$")

/// Dialog for creating a new GLoC reactor file.
class NewReactorDialog(
    project: Project,
    private val targetDir: VirtualFile,
) : DialogWrapper(project) {

    private val nameField = JBTextField(30)
    private val radioSimple = JRadioButton("Without Neutrons", true)
    private val radioNeutrons = JRadioButton("With Neutrons")
    private val previewArea = JTextArea().apply {
        isEditable = false
        font = Font(Font.MONOSPACED, Font.PLAIN, 12)
        background = JBColor(0x2b2b2b, 0x2b2b2b)
        foreground = JBColor(0xd4d4d4, 0xd4d4d4)
        border = JBUI.Borders.empty(8)
        lineWrap = false
        rows = 20
    }

    val reactorName: String get() = nameField.text.trim()
    val withNeutrons: Boolean get() = radioNeutrons.isSelected

    init {
        title = "New GLoC Reactor"
        init()
        updatePreview()

        val listener = object : DocumentListener {
            override fun insertUpdate(e: DocumentEvent?) = updatePreview()
            override fun removeUpdate(e: DocumentEvent?) = updatePreview()
            override fun changedUpdate(e: DocumentEvent?) = updatePreview()
        }
        nameField.document.addDocumentListener(listener)

        val typeListener = { _: Any -> updatePreview() }
        radioSimple.addActionListener(typeListener)
        radioNeutrons.addActionListener(typeListener)
    }

    override fun createCenterPanel(): JComponent {
        val group = ButtonGroup().apply {
            add(radioSimple)
            add(radioNeutrons)
        }

        val form = JPanel(GridBagLayout())
        val gc = GridBagConstraints().apply {
            fill = GridBagConstraints.HORIZONTAL
            insets = JBUI.insets(4, 0)
        }

        // Name row
        gc.gridx = 0; gc.gridy = 0; gc.weightx = 0.0
        form.add(JBLabel("Reactor Name:"), gc)
        gc.gridx = 1; gc.weightx = 1.0
        nameField.emptyText.text = "e.g. Counter"
        form.add(nameField, gc)

        // Type row
        gc.gridx = 0; gc.gridy = 1; gc.weightx = 0.0
        form.add(JBLabel("Reactor Type:"), gc)
        gc.gridx = 1; gc.weightx = 1.0
        val typePanel = JPanel().apply {
            layout = BoxLayout(this, BoxLayout.X_AXIS)
            isOpaque = false
            add(radioSimple)
            add(Box.createHorizontalStrut(16))
            add(radioNeutrons)
        }
        form.add(typePanel, gc)

        // Preview label + scroll pane
        gc.gridx = 0; gc.gridy = 2; gc.gridwidth = 2; gc.weightx = 1.0
        val previewLabel = JBLabel("Preview").apply {
            border = JBUI.Borders.emptyTop(8)
            font = font.deriveFont(Font.BOLD)
        }
        form.add(previewLabel, gc)

        gc.gridy = 3; gc.fill = GridBagConstraints.BOTH; gc.weighty = 1.0
        val scroll = JBScrollPane(previewArea).apply {
            border = BorderFactory.createLineBorder(JBColor(0x444444, 0x444444))
            preferredSize = JBUI.size(560, 320)
        }
        form.add(scroll, gc)

        val wrapper = JPanel(BorderLayout()).apply {
            border = JBUI.Borders.empty(8)
            add(form, BorderLayout.CENTER)
        }
        return wrapper
    }

    override fun doValidate(): ValidationInfo? {
        val name = nameField.text.trim()
        return when {
            name.isEmpty() -> ValidationInfo("Reactor name is required.", nameField)
            !PASCAL_CASE.matches(name) -> ValidationInfo(
                "Name must be PascalCase and start with an uppercase letter (e.g. Counter, CartItem).",
                nameField
            )
            targetDir.findChild("$name.rs") != null -> ValidationInfo(
                "A file named '$name.rs' already exists in the selected directory.",
                nameField
            )
            else -> null
        }
    }

    private fun updatePreview() {
        val name = nameField.text.trim().ifEmpty { "MyReactor" }
        previewArea.text = ReactorGenerator.generateContent(name, radioNeutrons.isSelected)
        previewArea.caretPosition = 0
    }
}
