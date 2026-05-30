import * as vscode from 'vscode';

const HEADING_RE = /^(#{1,6})\s+(.+)$/;
const LABEL_RE = /\\label\{([^}]+)\}/g;

function headingKind(level: number): vscode.SymbolKind {
    const kinds: vscode.SymbolKind[] = [
        vscode.SymbolKind.File,
        vscode.SymbolKind.Module,
        vscode.SymbolKind.Namespace,
        vscode.SymbolKind.Package,
        vscode.SymbolKind.Class,
        vscode.SymbolKind.Method
    ];
    return kinds[Math.min(level - 1, kinds.length - 1)];
}

function parseDocumentSymbols(document: vscode.TextDocument): vscode.DocumentSymbol[] {
    const symbols: vscode.DocumentSymbol[] = [];
    const text = document.getText();

    for (let i = 0; i < text.length; i++) {
        const rest = text.slice(i);
        const headingMatch = rest.match(HEADING_RE);
        if (headingMatch) {
            const line = text.slice(0, i).split('\n').length - 1;
            const range = new vscode.Range(line, 0, line, headingMatch[0].length);
            const name = headingMatch[2].trim();
            const kind = headingKind(headingMatch[1].length);
            symbols.push(new vscode.DocumentSymbol(name, '', kind, range, range));
            i += headingMatch[0].length;
        } else {
            LABEL_RE.lastIndex = 0;
            const labelMatch = LABEL_RE.exec(rest);
            if (labelMatch) {
                const offset = i + labelMatch.index;
                const before = text.slice(0, offset);
                const line = before.split('\n').length - 1;
                const col = before.split('\n').pop()!.length;
                const start = new vscode.Position(line, col);
                const end = new vscode.Position(line, col + labelMatch[0].length);
                const range = new vscode.Range(start, end);
                symbols.push(new vscode.DocumentSymbol(labelMatch[1], '', vscode.SymbolKind.Constant, range, range));
                i = offset + labelMatch[0].length;
            } else {
                break;
            }
        }
    }

    return symbols;
}

class LdirMarkdownDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
    provideDocumentSymbols(document: vscode.TextDocument): vscode.DocumentSymbol[] {
        return parseDocumentSymbols(document);
    }
}

class LdirWorkspaceSymbolProvider implements vscode.WorkspaceSymbolProvider {
    async provideWorkspaceSymbols(
        query: string
    ): Promise<vscode.SymbolInformation[]> {
        const results: vscode.SymbolInformation[] = [];
        const uris = await vscode.workspace.findFiles('**/*.{md,tex,typst}', '**/node_modules/**');

        for (const uri of uris) {
            try {
                const doc = await vscode.workspace.openTextDocument(uri);
                const symbols = parseDocumentSymbols(doc);
                const q = query.toLowerCase();
                for (const sym of symbols) {
                    if (sym.name.toLowerCase().includes(q)) {
                        const loc = new vscode.Location(uri, sym.range);
                        results.push(new vscode.SymbolInformation(sym.name, sym.kind, sym.detail ?? '', loc));
                    }
                }
            } catch {
                continue;
            }
        }

        return results;
    }
}

export function registerSymbolProviders(context: vscode.ExtensionContext): void {
    const docProvider = new LdirMarkdownDocumentSymbolProvider();
    context.subscriptions.push(
        vscode.languages.registerDocumentSymbolProvider(
            { language: 'ldir-markdown' },
            docProvider
        )
    );

    const wsProvider = new LdirWorkspaceSymbolProvider();
    context.subscriptions.push(
        vscode.languages.registerWorkspaceSymbolProvider(wsProvider)
    );
}

export { parseDocumentSymbols };
