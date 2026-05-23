
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'ldc' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'ldc'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'ldc' {
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Output file path. Defaults to first input stem + extension based on format')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Output file path. Defaults to first input stem + extension based on format')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--font', '--font', [CompletionResultType]::ParameterName, 'Primary font family name (e.g., "DejaVu Sans", "Noto Serif"). Auto-detected from system fonts if not specified')
            [CompletionResult]::new('--font-mono', '--font-mono', [CompletionResultType]::ParameterName, 'Monospace font family name (e.g., "DejaVu Sans Mono"). Auto-detected from system fonts if not specified')
            [CompletionResult]::new('--font-path', '--font-path', [CompletionResultType]::ParameterName, 'Path to primary font file (.ttf/.otf). Overrides --font when specified')
            [CompletionResult]::new('--title', '--title', [CompletionResultType]::ParameterName, 'Document title for PDF metadata')
            [CompletionResult]::new('--author', '--author', [CompletionResultType]::ParameterName, 'Document author for PDF metadata')
            [CompletionResult]::new('--subject', '--subject', [CompletionResultType]::ParameterName, 'Document subject for PDF metadata')
            [CompletionResult]::new('--margin', '--margin', [CompletionResultType]::ParameterName, 'Page margin in inches (applied uniformly to all sides)')
            [CompletionResult]::new('--page-size', '--page-size', [CompletionResultType]::ParameterName, 'Page size preset ("a4", "letter", "legal")')
            [CompletionResult]::new('--page-width', '--page-width', [CompletionResultType]::ParameterName, 'Custom page width in points (overrides --page-size)')
            [CompletionResult]::new('--page-height', '--page-height', [CompletionResultType]::ParameterName, 'Custom page height in points (overrides --page-size)')
            [CompletionResult]::new('--header-left', '--header-left', [CompletionResultType]::ParameterName, 'Header left template (supports %page, %pages, %title, %author, %date)')
            [CompletionResult]::new('--header-center', '--header-center', [CompletionResultType]::ParameterName, 'Header center template')
            [CompletionResult]::new('--header-right', '--header-right', [CompletionResultType]::ParameterName, 'Header right template')
            [CompletionResult]::new('--footer-left', '--footer-left', [CompletionResultType]::ParameterName, 'Footer left template')
            [CompletionResult]::new('--footer-center', '--footer-center', [CompletionResultType]::ParameterName, 'Footer center template')
            [CompletionResult]::new('--footer-right', '--footer-right', [CompletionResultType]::ParameterName, 'Footer right template (default: %page)')
            [CompletionResult]::new('--bibliography', '--bibliography', [CompletionResultType]::ParameterName, 'Path to BibTeX (.bib) file for citations')
            [CompletionResult]::new('--pdfa-level', '--pdfa-level', [CompletionResultType]::ParameterName, 'PDF/A conformance level ("4" for PDF/A-4, "2b" for PDF/A-2b)')
            [CompletionResult]::new('--color', '--color', [CompletionResultType]::ParameterName, 'Color output. Options: auto, always, never. Default: auto')
            [CompletionResult]::new('--list-fonts', '--list-fonts', [CompletionResultType]::ParameterName, 'List available system fonts and exit')
            [CompletionResult]::new('--no-header-rule', '--no-header-rule', [CompletionResultType]::ParameterName, 'Disable header rule line')
            [CompletionResult]::new('--no-footer-rule', '--no-footer-rule', [CompletionResultType]::ParameterName, 'Disable footer rule line')
            [CompletionResult]::new('--drop-caps', '--drop-caps', [CompletionResultType]::ParameterName, 'Enable drop caps for the first paragraph after headings')
            [CompletionResult]::new('--lir', '--lir', [CompletionResultType]::ParameterName, 'Use the L-IR layout pipeline (S-IR → L-IR → G-IR) instead of direct compilation')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
