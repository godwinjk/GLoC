package com.gloc.plugin

import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.vfs.VfsUtil

/// Generates GLoC reactor source content and handles file creation on disk.
object ReactorGenerator {

    /// Returns the Rust source for a reactor with the given name and type.
    fun generateContent(name: String, withNeutrons: Boolean): String {
        return if (withNeutrons) {
            """use gloc::prelude::*;

#[reactor_state]
pub struct ${name}State {
    // TODO: add state fields
}

impl Default for ${name}State {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub enum ${name}Neutron {
    // TODO: add neutron variants
}

#[reactor(state = ${name}State, neutrons = ${name}Neutron)]
pub struct ${name}Reactor {}

impl ${name}Reactor {
    fn on_event(&mut self, neutron: ${name}Neutron) {
        match neutron {
            // TODO: handle neutron variants
        }
    }
}
"""
        } else {
            """use gloc::prelude::*;

#[reactor_state]
pub struct ${name}State {
    // TODO: add state fields
}

impl Default for ${name}State {
    fn default() -> Self {
        Self {}
    }
}

#[reactor(state = ${name}State)]
pub struct ${name}Reactor {}

impl ${name}Reactor {
    // TODO: add methods that call self.emit(...)
}
"""
        }
    }

    /// Creates the reactor file in [dir] and opens it in the editor.
    fun createFile(project: Project, dir: VirtualFile, name: String, withNeutrons: Boolean) {
        WriteCommandAction.runWriteCommandAction(project) {
            try {
                val file = dir.createChildData(this, "$name.rs")
                VfsUtil.saveText(file, generateContent(name, withNeutrons))
                FileEditorManager.getInstance(project).openFile(file, true)
            } catch (ex: Exception) {
                Messages.showErrorDialog(project, "Failed to create $name.rs: ${ex.message}", "GLoC Error")
            }
        }
    }
}
