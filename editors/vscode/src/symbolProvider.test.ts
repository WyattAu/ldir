import * as assert from 'assert';
import * as vscode from 'vscode';
import { parseDocumentSymbols } from '../src/symbolProvider';

const SAMPLE_MARKDOWN = [
    '# Introduction',
    'Some text here.',
    '',
    '## Background',
    '',
    'As shown in \\cite{smith2020}, the results are clear.',
    'The equation is $E = mc^2$.',
    '',
    '### Subsection',
    'See \\label{sec:background} for details.',
    '',
    '#### Deep heading',
    'With \\ref{fig:diagram}.',
    '',
    '\\begin{equation}',
    '  x = 1',
    '\\end{equation}',
    '',
    '\\label{eq:one}',
].join('\n');

function makeDocument(text: string): vscode.TextDocument {
    return {
        getText: () => text,
        uri: vscode.Uri.parse('file:///test.md'),
        lineCount: text.split('\n').length,
    } as unknown as vscode.TextDocument;
}

suite('Symbol Provider', () => {
    test('test_heading_symbols', () => {
        const doc = makeDocument(SAMPLE_MARKDOWN);
        const symbols = parseDocumentSymbols(doc);
        const headings = symbols.filter(s =>
            s.kind === vscode.SymbolKind.Module ||
            s.kind === vscode.SymbolKind.Namespace ||
            s.kind === vscode.SymbolKind.Package ||
            s.kind === vscode.SymbolKind.Class ||
            s.kind === vscode.SymbolKind.Method
        );

        assert.strictEqual(headings.length, 4, 'should find 4 headings');

        assert.strictEqual(headings[0].name, 'Introduction');
        assert.strictEqual(headings[0].kind, vscode.SymbolKind.Module);

        assert.strictEqual(headings[1].name, 'Background');
        assert.strictEqual(headings[1].kind, vscode.SymbolKind.Namespace);

        assert.strictEqual(headings[2].name, 'Subsection');
        assert.strictEqual(headings[2].kind, vscode.SymbolKind.Package);

        assert.strictEqual(headings[3].name, 'Deep heading');
        assert.strictEqual(headings[3].kind, vscode.SymbolKind.Class);
    });

    test('test_label_symbols', () => {
        const doc = makeDocument(SAMPLE_MARKDOWN);
        const symbols = parseDocumentSymbols(doc);
        const labels = symbols.filter(s => s.kind === vscode.SymbolKind.Constant);

        assert.strictEqual(labels.length, 2, 'should find 2 labels');

        assert.strictEqual(labels[0].name, 'sec:background');
        assert.strictEqual(labels[1].name, 'eq:one');
    });
});
