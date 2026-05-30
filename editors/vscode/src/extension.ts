import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';
import { registerSymbolProviders } from './symbolProvider';

let client: LanguageClient;
let statusBarItem: vscode.StatusBarItem;

interface PreviewStatusParams {
    state: 'compiling' | 'ready' | 'error';
    message?: string;
}

export function activate(context: vscode.ExtensionContext) {
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = 'LDIR: Starting...';
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
            { scheme: 'file', language: 'ldir-markdown' }
        ],
        synchronize: {
            configurationSection: 'ldir'
        }
    };

    client = new LanguageClient('ldir', 'ldir Language Server', serverOptions, clientOptions);

    client.onNotification('ldir/previewStatus', (params: PreviewStatusParams) => {
        switch (params.state) {
            case 'compiling':
                statusBarItem.text = 'LDIR: Compiling...';
                statusBarItem.color = undefined;
                break;
            case 'ready':
                statusBarItem.text = 'LDIR: Ready';
                statusBarItem.color = undefined;
                break;
            case 'error':
                statusBarItem.text = 'LDIR: Error';
                statusBarItem.color = new vscode.ThemeColor('errorForeground');
                if (params.message) {
                    vscode.window.showErrorMessage(`ldir: ${params.message}`);
                }
                break;
        }
    });

    client.start().then(() => {
        statusBarItem.text = 'LDIR: Ready';
    }).catch(() => {
        statusBarItem.text = 'LDIR: No Server';
        statusBarItem.color = new vscode.ThemeColor('errorForeground');
    });

    const compileCmd = vscode.commands.registerCommand('ldir.compile', async () => {
        const doc = vscode.window.activeTextEditor?.document;
        if (!doc) return;
        statusBarItem.text = 'LDIR: Compiling...';
        try {
            const compilerPath = vscode.workspace.getConfiguration('ldir').get<string>('compilerPath', 'ldc');
            const terminal = vscode.window.createTerminal('ldir compile');
            terminal.show();
            const outPath = doc.uri.fsPath.replace(/\.\w+$/, '.pdf');
            terminal.sendText(`${compilerPath} "${doc.uri.fsPath}" -o "${outPath}"`);
        } finally {
            statusBarItem.text = 'LDIR: Ready';
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

    registerSymbolProviders(context);
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
