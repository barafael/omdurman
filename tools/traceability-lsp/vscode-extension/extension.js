// Self-contained LSP client for traceability-lsp: spawns the server binary and
// speaks JSON-RPC over stdio directly (no vscode-languageclient dependency).
// Handles diagnostics + hover/definition/references/implementation/codeLens.

const vscode = require('vscode');
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

function activate(context) {
  const client = new TraceabilityClient(context);
  context.subscriptions.push(
    client,
    vscode.commands.registerCommand('traceabilityLsp.setServerPath', () => client.pickServerPath()),
    vscode.commands.registerCommand('traceabilityLsp.restart', () => client.restart()),
  );
  return client.start();
}

class TraceabilityClient {
  constructor(context) {
    this.context = context;
    this.proc = null;
    this.buf = Buffer.alloc(0);
    this.nextId = 1;
    this.pending = new Map();
    this.ready = null;
    this.diags = vscode.languages.createDiagnosticCollection('traceability');
    this.relevantLangs = ['toml', 'markdown', 'rust'];
  }

  async start() {
    const bin = this.findServer();
    if (!bin) {
      const pick = await vscode.window.showWarningMessage(
        'traceability-lsp: server binary not found. Set the path or pick a location.',
        'Select server binary…',
        'Open Settings',
      );
      if (pick === 'Select server binary…') {
        await this.pickServerPath();
      } else if (pick === 'Open Settings') {
        await vscode.commands.executeCommand('workbench.action.openSettings', 'traceabilityLsp.serverPath');
      }
      return;
    }
    const root = this.workspaceRoot();
    this.proc = spawn(bin, [], { cwd: root });
    this.proc.stdout.on('data', (d) => this.onData(d));
    this.proc.stderr.on('data', (d) => console.error('[traceability-lsp]', d.toString().trim()));
    this.proc.on('exit', (code) => {
      console.log(`[traceability-lsp] server exited: ${code}`);
      this.proc = null;
    });

    this.ready = this.handshake(root);

    // Notify the server about documents as they open / change / save / close.
    this.context.subscriptions.push(
      vscode.workspace.onDidOpenTextDocument((doc) => this.sendOpen(doc)),
      vscode.workspace.onDidChangeTextDocument((e) => this.sendChange(e)),
      vscode.workspace.onDidSaveTextDocument((doc) => this.sendSave(doc)),
      vscode.workspace.onDidCloseTextDocument((doc) => this.sendClose(doc)),
    );
    for (const doc of vscode.workspace.textDocuments) {
      this.sendOpen(doc);
    }

    this.registerProviders();
  }

  workspaceRoot() {
    return (vscode.workspace.workspaceFolders && vscode.workspace.workspaceFolders[0])
      ? vscode.workspace.workspaceFolders[0].uri.fsPath
      : '';
  }

  // Candidates for the server binary, in order of preference.
  serverCandidates() {
    const names = ['target/release/traceability-lsp', 'target/debug/traceability-lsp'];
    const roots = [
      this.workspaceRoot(),
      path.join(this.context.extensionPath, '..', '..'),
      process.env.HOME || '',
    ];
    const out = [];
    for (const root of roots) {
      if (!root) continue;
      for (const name of names) {
        const p = path.join(root, name);
        if (!out.includes(p)) out.push(p);
      }
    }
    return out;
  }

  findServer() {
    const configured = vscode.workspace.getConfiguration('traceabilityLsp').get('serverPath', '');
    if (configured) return configured;
    return this.serverCandidates().find((p) => fs.existsSync(p)) || null;
  }

  // Command: let the user point the extension at the binary.
  async pickServerPath() {
    const candidates = this.serverCandidates().filter((p) => fs.existsSync(p));
    const pick = await vscode.window.showQuickPick(
      [
        ...candidates.map((p) => ({ label: `$(file-binary) ${p}`, description: 'detected', value: p })),
        { label: '$(folder) Browse…', description: 'pick the binary manually', value: '' },
      ],
      { placeHolder: 'Where is the traceability-lsp binary?' },
    );
    let chosen = pick && pick.value;
    if (chosen === undefined) return;
    if (chosen === '') {
      const uri = await vscode.window.showOpenDialog({
        canSelectFiles: true,
        canSelectMany: false,
        openLabel: 'Select server binary',
      });
      if (!uri || !uri.length) return;
      chosen = uri[0].fsPath;
    }
    await vscode.workspace.getConfiguration('traceabilityLsp').update('serverPath', chosen, vscode.ConfigurationTarget.Workspace);
    vscode.window.showInformationMessage(`traceability-lsp: server path set to ${chosen}. Restart to apply.`);
  }

  async restart() {
    if (this.proc) {
      this.notify('exit', {});
      try { this.proc.kill(); } catch (_) { /* already dead */ }
      this.proc = null;
    }
    this.diags.clear();
    await this.start();
  }

  async handshake(root) {
    const init = this.request('initialize', {
      capabilities: {},
      rootUri: root ? vscode.Uri.file(root).toString() : null,
      processId: process.pid,
    });
    await init;
    this.notify('initialized', {});
    this.notify('workspace/didChangeWatchedFiles', { changes: [] });
  }

  async request(method, params) {
    await this.waitForProc();
    const id = this.nextId++;
    const msg = { jsonrpc: '2.0', id, method, params };
    this.proc.stdin.write(this.frame(msg));
    return new Promise((resolve, reject) => this.pending.set(id, { resolve, reject }));
  }

  notify(method, params) {
    if (!this.proc) return;
    this.proc.stdin.write(this.frame({ jsonrpc: '2.0', method, params }));
  }

  async waitForProc() {
    while (!this.proc) {
      await new Promise((r) => setTimeout(r, 20));
    }
  }

  frame(msg) {
    const body = Buffer.from(JSON.stringify(msg), 'utf8');
    return Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, 'ascii'), body]);
  }

  onData(d) {
    this.buf = Buffer.concat([this.buf, d]);
    for (;;) {
      const headerEnd = this.buf.indexOf('\r\n\r\n');
      if (headerEnd < 0) return;
      const match = /Content-Length: (\d+)/i.exec(this.buf.slice(0, headerEnd).toString('ascii'));
      if (!match) return;
      const len = parseInt(match[1], 10);
      if (this.buf.length < headerEnd + 4 + len) return;
      const body = this.buf.slice(headerEnd + 4, headerEnd + 4 + len).toString('utf8');
      this.buf = this.buf.slice(headerEnd + 4 + len);
      this.handleMessage(JSON.parse(body));
    }
  }

  handleMessage(msg) {
    if (msg.method === 'textDocument/publishDiagnostics') {
      const uri = vscode.Uri.parse(msg.params.uri);
      const map = msg.params.diagnostics.map((d) => {
        const rng = new vscode.Range(d.range.start.line, d.range.start.character, d.range.end.line, d.range.end.character);
        const sev = d.severity === 1 ? vscode.DiagnosticSeverity.Error
          : d.severity === 2 ? vscode.DiagnosticSeverity.Warning
          : vscode.DiagnosticSeverity.Information;
        return new vscode.Diagnostic(rng, d.message, sev);
      });
      this.diags.set(uri, map);
      return;
    }
    if (msg.id !== undefined && this.pending.has(msg.id)) {
      const p = this.pending.get(msg.id);
      this.pending.delete(msg.id);
      if (msg.error) p.reject(new Error(msg.error.message));
      else p.resolve(msg.result);
    }
  }

  sendOpen(doc) {
    if (!this.relevant(doc)) return;
    this.notify('textDocument/didOpen', {
      textDocument: { uri: doc.uri.toString(), languageId: doc.languageId, version: doc.version, text: doc.getText() },
    });
  }

  sendChange(e) {
    if (!this.relevant(e.document)) return;
    if (!e.contentChanges.length) return;
    const change = e.contentChanges[e.contentChanges.length - 1];
    this.notify('textDocument/didChange', {
      textDocument: { uri: e.document.uri.toString(), version: e.document.version },
      contentChanges: [{ text: change.text }],
    });
  }

  sendSave(doc) {
    if (!this.relevant(doc)) return;
    this.notify('textDocument/didSave', { textDocument: { uri: doc.uri.toString() }, text: doc.getText() });
  }

  sendClose(doc) {
    if (!this.relevant(doc)) return;
    this.notify('textDocument/didClose', { textDocument: { uri: doc.uri.toString() } });
  }

  relevant(doc) {
    return doc.languageId === 'rust' || doc.languageId === 'toml' || doc.languageId === 'markdown';
  }

  registerProviders() {
    const sel = [{ language: 'rust' }, { language: 'toml' }, { language: 'markdown' }];
    this.context.subscriptions.push(
      vscode.languages.registerHoverProvider(sel, {
        provideHover: async (doc, pos) => {
          const r = await this.request('textDocument/hover', { textDocument: { uri: doc.uri.toString() }, position: this.pos(pos) });
          if (!r || !r.contents) return null;
          const value = r.contents.kind === 'markdown' ? r.contents.value : String(r.contents);
          return new vscode.Hover(new vscode.MarkdownString(value));
        },
      }),
      vscode.languages.registerDefinitionProvider(sel, {
        provideDefinition: async (doc, pos) => {
          const r = await this.request('textDocument/definition', { textDocument: { uri: doc.uri.toString() }, position: this.pos(pos) });
          return this.locs(r);
        },
      }),
      vscode.languages.registerImplementationProvider(sel, {
        provideImplementation: async (doc, pos) => {
          const r = await this.request('textDocument/implementation', { textDocument: { uri: doc.uri.toString() }, position: this.pos(pos) });
          return this.locs(r);
        },
      }),
      vscode.languages.registerReferenceProvider(sel, {
        provideReferences: async (doc, pos) => {
          const r = await this.request('textDocument/references', {
            textDocument: { uri: doc.uri.toString() }, position: this.pos(pos), context: { includeDeclaration: true },
          });
          return this.locs(r);
        },
      }),
      vscode.languages.registerCodeLensProvider(sel, {
        provideCodeLenses: async (doc) => {
          const r = await this.request('textDocument/codeLens', { textDocument: { uri: doc.uri.toString() } });
          return (r || []).map((l) => new vscode.CodeLens(
            new vscode.Range(l.range.start.line, l.range.start.character, l.range.end.line, l.range.end.character),
            { title: l.command.title, command: l.command.command, arguments: l.command.arguments || [] },
          ));
        },
      }),
    );
  }

  pos(p) {
    return { line: p.line, character: p.character };
  }

  locs(result) {
    const one = (loc) => new vscode.Location(
      vscode.Uri.parse(loc.uri),
      new vscode.Range(loc.range.start.line, loc.range.start.character, loc.range.end.line, loc.range.end.character),
    );
    if (!result) return [];
    if (Array.isArray(result)) return result.map(one);
    if (result.uri) return [one(result)];
    return [];
  }

  dispose() {
    if (this.proc) {
      this.notify('exit', {});
      setTimeout(() => this.proc && this.proc.kill(), 500);
    }
    this.diags.dispose();
  }
}

function deactivate() {}

module.exports = { activate, deactivate };
