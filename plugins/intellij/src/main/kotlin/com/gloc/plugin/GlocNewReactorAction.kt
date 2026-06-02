package com.gloc.plugin

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys

/// Right-click action that opens the New GLoC Reactor dialog.
class GlocNewReactorAction : AnAction() {

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val clicked = e.getData(CommonDataKeys.VIRTUAL_FILE) ?: return
        val targetDir = if (clicked.isDirectory) clicked else clicked.parent ?: return

        val dialog = NewReactorDialog(project, targetDir)
        if (dialog.showAndGet()) {
            ReactorGenerator.createFile(project, targetDir, dialog.reactorName, dialog.withNeutrons)
        }
    }

    override fun update(e: AnActionEvent) {
        // Only enable when a file or directory is selected in the project view.
        e.presentation.isEnabledAndVisible = e.getData(CommonDataKeys.VIRTUAL_FILE) != null
    }
}
