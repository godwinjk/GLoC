import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

type IncomingMessage =
  | { type: 'validate'; name: string }
  | { type: 'create'; name: string; withNeutrons: boolean }
  | { type: 'cancel' };

export class ReactorDialog {
  private panel: vscode.WebviewPanel | undefined;

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly folderUri: vscode.Uri
  ) {}

  show(): void {
    this.panel = vscode.window.createWebviewPanel(
      'glocNewReactor',
      'New GLoC Reactor',
      vscode.ViewColumn.Active,
      { enableScripts: true, retainContextWhenHidden: false }
    );

    this.panel.webview.html = this.buildHtml(this.panel.webview);

    this.panel.webview.onDidReceiveMessage((msg: IncomingMessage) => {
      switch (msg.type) {
        case 'validate':
          this.handleValidate(msg.name);
          break;
        case 'create':
          this.handleCreate(msg.name, msg.withNeutrons);
          break;
        case 'cancel':
          this.panel?.dispose();
          break;
      }
    });
  }

  private handleValidate(name: string): void {
    const filePath = path.join(this.folderUri.fsPath, `${name}.rs`);
    const exists = fs.existsSync(filePath);
    this.panel?.webview.postMessage({ type: 'validationResult', exists });
  }

  private async handleCreate(name: string, withNeutrons: boolean): Promise<void> {
    const filePath = path.join(this.folderUri.fsPath, `${name}.rs`);
    const content = generateContent(name, withNeutrons);
    try {
      fs.writeFileSync(filePath, content, 'utf-8');
      this.panel?.dispose();
      const doc = await vscode.workspace.openTextDocument(filePath);
      await vscode.window.showTextDocument(doc);
    } catch (err) {
      vscode.window.showErrorMessage(`GLoC: Failed to create file — ${err}`);
    }
  }

  private buildHtml(webview: vscode.Webview): string {
    const nonce = getNonce();
    const csp = `default-src 'none'; style-src 'nonce-${nonce}'; script-src 'nonce-${nonce}';`;
    const folderPath = this.folderUri.fsPath;

    return /* html */`<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="${csp}">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>New GLoC Reactor</title>
  <style nonce="${nonce}">
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      font-family: var(--vscode-font-family, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif);
      font-size: 13px;
      background: var(--vscode-editor-background, #1e1e1e);
      color: var(--vscode-foreground, #cccccc);
      padding: 24px;
      min-height: 100vh;
    }

    h2 {
      font-size: 16px;
      font-weight: 600;
      margin-bottom: 20px;
      color: var(--vscode-titleBar-activeForeground, #fff);
    }

    .field { margin-bottom: 16px; }

    label {
      display: block;
      margin-bottom: 6px;
      font-weight: 500;
      color: var(--vscode-foreground, #ccc);
    }

    input[type="text"] {
      width: 100%;
      padding: 6px 10px;
      background: var(--vscode-input-background, #3c3c3c);
      color: var(--vscode-input-foreground, #ccc);
      border: 1px solid var(--vscode-input-border, #555);
      border-radius: 3px;
      font-size: 13px;
      outline: none;
      transition: border-color 0.15s;
    }
    input[type="text"]:focus { border-color: var(--vscode-focusBorder, #007fd4); }
    input[type="text"].error { border-color: #f44747; }

    .error-msg {
      margin-top: 5px;
      font-size: 11px;
      color: #f44747;
      min-height: 15px;
    }

    .radio-group {
      display: flex;
      gap: 20px;
      margin-top: 4px;
    }
    .radio-group label {
      display: flex;
      align-items: center;
      gap: 7px;
      cursor: pointer;
      font-weight: 400;
      margin-bottom: 0;
    }
    input[type="radio"] { accent-color: var(--vscode-focusBorder, #007fd4); cursor: pointer; }

    .preview-label {
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--vscode-descriptionForeground, #999);
      margin-bottom: 6px;
    }

    pre {
      background: var(--vscode-textCodeBlock-background, #2d2d2d);
      border: 1px solid var(--vscode-panel-border, #444);
      border-radius: 4px;
      padding: 14px;
      font-family: var(--vscode-editor-font-family, 'Cascadia Code', Menlo, monospace);
      font-size: 12px;
      line-height: 1.6;
      overflow: auto;
      max-height: 340px;
      white-space: pre;
      color: #d4d4d4;
    }

    /* Syntax colours */
    .kw  { color: #569cd6; }   /* keywords */
    .ty  { color: #4ec9b0; }   /* types */
    .mac { color: #dcdcaa; }   /* macros */
    .cm  { color: #6a9955; font-style: italic; } /* comments */
    .attr{ color: #9cdcfe; }   /* attributes */
    .str { color: #ce9178; }   /* strings */

    .actions {
      display: flex;
      justify-content: flex-end;
      gap: 10px;
      margin-top: 20px;
    }

    button {
      padding: 6px 18px;
      font-size: 13px;
      border-radius: 3px;
      border: none;
      cursor: pointer;
      transition: opacity 0.15s;
    }
    button:disabled { opacity: 0.4; cursor: not-allowed; }

    #btnCreate {
      background: var(--vscode-button-background, #0e639c);
      color: var(--vscode-button-foreground, #fff);
    }
    #btnCreate:not(:disabled):hover { background: var(--vscode-button-hoverBackground, #1177bb); }

    #btnCancel {
      background: var(--vscode-button-secondaryBackground, #3a3d41);
      color: var(--vscode-button-secondaryForeground, #ccc);
    }
    #btnCancel:hover { background: var(--vscode-button-secondaryHoverBackground, #45494e); }

    hr { border: none; border-top: 1px solid var(--vscode-panel-border, #444); margin: 18px 0; }
  </style>
</head>
<body>
  <h2>New GLoC Reactor</h2>

  <div class="field">
    <label for="reactorName">Reactor Name</label>
    <input type="text" id="reactorName" placeholder="e.g. Counter" autocomplete="off" spellcheck="false">
    <div class="error-msg" id="nameError"></div>
  </div>

  <div class="field">
    <label>Reactor Type</label>
    <div class="radio-group">
      <label><input type="radio" name="reactorType" value="simple" checked> Without Neutrons</label>
      <label><input type="radio" name="reactorType" value="neutrons"> With Neutrons</label>
    </div>
  </div>

  <hr>

  <div class="preview-label">Preview</div>
  <pre id="preview"><span class="cm">// Enter a name above to see the generated file.</span></pre>

  <div class="actions">
    <button id="btnCancel">Cancel</button>
    <button id="btnCreate" disabled>Create</button>
  </div>

  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    const nameInput   = document.getElementById('reactorName');
    const nameError   = document.getElementById('nameError');
    const preview     = document.getElementById('preview');
    const btnCreate   = document.getElementById('btnCreate');
    const radios      = document.querySelectorAll('input[name="reactorType"]');

    const PASCAL = /^[A-Z][a-zA-Z0-9]*$/;
    const FOLDER = ${JSON.stringify(folderPath)};

    let validName    = false;
    let fileExists   = false;
    let pendingCheck = null;

    function withNeutrons() {
      return document.querySelector('input[name="reactorType"]:checked').value === 'neutrons';
    }

    function generatePreview(name, neutrons) {
      if (!name || !PASCAL.test(name)) {
        return '<span class="cm">// Enter a valid PascalCase name above.</span>';
      }
      const n = name;
      if (neutrons) {
        return [
          '<span class="kw">use</span> gloc::prelude::*;',
          '',
          '<span class="attr">#[reactor_state]</span>',
          '<span class="kw">pub struct</span> <span class="ty">' + n + 'State</span> {',
          '    <span class="cm">// TODO: add state fields</span>',
          '}',
          '',
          '<span class="kw">impl</span> <span class="ty">Default</span> <span class="kw">for</span> <span class="ty">' + n + 'State</span> {',
          '    <span class="kw">fn</span> <span class="mac">default</span>() -> <span class="ty">Self</span> {',
          '        <span class="ty">Self</span> {}',
          '    }',
          '}',
          '',
          '<span class="attr">#[derive(Debug)]</span>',
          '<span class="kw">pub enum</span> <span class="ty">' + n + 'Neutron</span> {',
          '    <span class="cm">// TODO: add neutron variants</span>',
          '}',
          '',
          '<span class="attr">#[reactor(state = <span class="ty">' + n + 'State</span>, neutrons = <span class="ty">' + n + 'Neutron</span>)]</span>',
          '<span class="kw">pub struct</span> <span class="ty">' + n + 'Reactor</span> {}',
          '',
          '<span class="kw">impl</span> <span class="ty">' + n + 'Reactor</span> {',
          '    <span class="kw">fn</span> <span class="mac">on_event</span>(&<span class="kw">mut</span> <span class="kw">self</span>, neutron: <span class="ty">' + n + 'Neutron</span>) {',
          '        <span class="kw">match</span> neutron {',
          '            <span class="cm">// TODO: handle neutron variants</span>',
          '        }',
          '    }',
          '}',
        ].join('\\n');
      } else {
        return [
          '<span class="kw">use</span> gloc::prelude::*;',
          '',
          '<span class="attr">#[reactor_state]</span>',
          '<span class="kw">pub struct</span> <span class="ty">' + n + 'State</span> {',
          '    <span class="cm">// TODO: add state fields</span>',
          '}',
          '',
          '<span class="kw">impl</span> <span class="ty">Default</span> <span class="kw">for</span> <span class="ty">' + n + 'State</span> {',
          '    <span class="kw">fn</span> <span class="mac">default</span>() -> <span class="ty">Self</span> {',
          '        <span class="ty">Self</span> {}',
          '    }',
          '}',
          '',
          '<span class="attr">#[reactor(state = <span class="ty">' + n + 'State</span>)]</span>',
          '<span class="kw">pub struct</span> <span class="ty">' + n + 'Reactor</span> {}',
          '',
          '<span class="kw">impl</span> <span class="ty">' + n + 'Reactor</span> {',
          '    <span class="cm">// TODO: add methods that call self.emit(...)</span>',
          '}',
        ].join('\\n');
      }
    }

    function validate() {
      const name = nameInput.value.trim();
      preview.innerHTML = generatePreview(name, withNeutrons());

      if (!name) {
        setError('Name is required.');
        return;
      }
      if (!PASCAL.test(name)) {
        setError('Must be PascalCase (e.g. Counter, CartItem).');
        return;
      }

      // Async file-existence check — debounced
      clearTimeout(pendingCheck);
      pendingCheck = setTimeout(() => {
        vscode.postMessage({ type: 'validate', name });
      }, 200);

      // Optimistically clear format error while we wait
      setError('');
    }

    function setError(msg) {
      nameError.textContent = msg;
      const hasError = msg.length > 0;
      nameInput.classList.toggle('error', hasError);
      btnCreate.disabled = hasError || fileExists;
      validName = !hasError;
    }

    nameInput.addEventListener('input', validate);
    radios.forEach(r => r.addEventListener('change', validate));

    btnCreate.addEventListener('click', () => {
      const name = nameInput.value.trim();
      if (!validName || fileExists) return;
      vscode.postMessage({ type: 'create', name, withNeutrons: withNeutrons() });
    });

    btnCancel.addEventListener('click', () => {
      vscode.postMessage({ type: 'cancel' });
    });

    // Message from extension
    window.addEventListener('message', (event) => {
      const msg = event.data;
      if (msg.type === 'validationResult') {
        fileExists = msg.exists;
        if (fileExists) {
          setError('A file with this name already exists in the selected folder.');
        } else {
          setError('');
        }
      }
    });

    nameInput.focus();
  </script>
</body>
</html>`;
  }
}

function generateContent(name: string, withNeutrons: boolean): string {
  if (withNeutrons) {
    return `use gloc::prelude::*;

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
`;
  }

  return `use gloc::prelude::*;

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
`;
}

function getNonce(): string {
  let text = '';
  const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}
