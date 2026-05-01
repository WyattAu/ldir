import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = 'ldir';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    const serverPath = vscode.workspace.getConfiguration('ldir').get<string>('serverPath', 'ldir-lsp');

    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'ldir-tex' },
            { scheme: 'file', language: 'ldir-typst' },
            { scheme: 'file', language: 'markdown' }
        ],
        synchronize: {
            configurationSection: 'ldir'
        }
    };

    client = new LanguageClient('ldir', 'ldir Language Server', serverOptions, clientOptions);
    client.start().catch(() => {
        statusBarItem.text = 'ldir (no server)';
    });

    const compileCmd = vscode.commands.registerCommand('ldir.compile', async () => {
        const doc = vscode.window.activeTextEditor?.document;
        if (!doc) return;
        statusBarItem.text = '$(sync~spin) ldir: compiling...';
        try {
            const compilerPath = vscode.workspace.getConfiguration('ldir').get<string>('compilerPath', 'ldc');
            const terminal = vscode.window.createTerminal('ldir compile');
            terminal.show();
            const outPath = doc.uri.fsPath.replace(/\.\w+$/, '.pdf');
            terminal.sendText(`${compilerPath} "${doc.uri.fsPath}" -o "${outPath}"`);
        } finally {
            statusBarItem.text = 'ldir';
        }
    });

    const previewCmd = vscode.commands.registerCommand('ldir.showPreview', async () => {
        const doc = vscode.window.activeTextEditor?.document;
        if (!doc) return;
        const pdfPath = doc.uri.fsPath.replace(/\.\w+$/, '.pdf');
        const uri = vscode.Uri.file(pdfPath);
        try {
            await vscode.commands.executeCommand('vscode.open', uri);
        } catch {
            vscode.window.showWarningMessage('PDF not found. Run "ldir: Compile Document" first.');
        }
    });

    const irCmd = vscode.commands.registerCommand('ldir.showIR', async () => {
        const doc = vscode.window.activeTextEditor?.document;
        if (!doc) return;
        const compilerPath = vscode.workspace.getConfiguration('ldir').get<string>('compilerPath', 'ldc');
        const terminal = vscode.window.createTerminal('ldir ir');
        terminal.show();
        terminal.sendText(`${compilerPath} "${doc.uri.fsPath}" --format ldir`);
    });

    context.subscriptions.push(compileCmd, previewCmd, irCmd);
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
