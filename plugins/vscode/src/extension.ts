import * as vscode from 'vscode';
import { ReactorDialog } from './reactorDialog';

export function activate(context: vscode.ExtensionContext): void {
  const disposable = vscode.commands.registerCommand(
    'gloc.newReactor',
    async (folderUri?: vscode.Uri) => {
      const targetUri = folderUri ?? vscode.workspace.workspaceFolders?.[0]?.uri;
      if (!targetUri) {
        vscode.window.showErrorMessage('GLoC: No folder selected.');
        return;
      }
      const dialog = new ReactorDialog(context, targetUri);
      dialog.show();
    }
  );
  context.subscriptions.push(disposable);
}

export function deactivate(): void {}
