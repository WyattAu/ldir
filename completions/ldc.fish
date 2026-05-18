complete -c ldc -s o -l output -d 'Output file path. Defaults to first input stem + extension based on format' -r -F
complete -c ldc -s f -l format -d 'Output format' -r -f -a "pdf\t''
gir\t''
sir\t''
html\t''
epub\t''
txt\t''
docx\t''
sir2\t''
ldir\t''"
complete -c ldc -l font -d 'Primary font family name (e.g., "DejaVu Sans", "Noto Serif"). Auto-detected from system fonts if not specified' -r
complete -c ldc -l font-mono -d 'Monospace font family name (e.g., "DejaVu Sans Mono"). Auto-detected from system fonts if not specified' -r
complete -c ldc -l font-path -d 'Path to primary font file (.ttf/.otf). Overrides --font when specified' -r -F
complete -c ldc -l title -d 'Document title for PDF metadata' -r
complete -c ldc -l author -d 'Document author for PDF metadata' -r
complete -c ldc -l subject -d 'Document subject for PDF metadata' -r
complete -c ldc -l margin -d 'Page margin in inches (applied uniformly to all sides)' -r
complete -c ldc -l page-size -d 'Page size preset ("a4", "letter", "legal")' -r
complete -c ldc -l page-width -d 'Custom page width in points (overrides --page-size)' -r
complete -c ldc -l page-height -d 'Custom page height in points (overrides --page-size)' -r
complete -c ldc -l header-left -d 'Header left template (supports %page, %pages, %title, %author, %date)' -r
complete -c ldc -l header-center -d 'Header center template' -r
complete -c ldc -l header-right -d 'Header right template' -r
complete -c ldc -l footer-left -d 'Footer left template' -r
complete -c ldc -l footer-center -d 'Footer center template' -r
complete -c ldc -l footer-right -d 'Footer right template (default: %page)' -r
complete -c ldc -l bibliography -d 'Path to BibTeX (.bib) file for citations' -r -F
complete -c ldc -l pdfa-level -d 'PDF/A conformance level ("4" for PDF/A-4, "2b" for PDF/A-2b)' -r
complete -c ldc -l list-fonts -d 'List available system fonts and exit'
complete -c ldc -l no-header-rule -d 'Disable header rule line'
complete -c ldc -l no-footer-rule -d 'Disable footer rule line'
complete -c ldc -l drop-caps -d 'Enable drop caps for the first paragraph after headings'
complete -c ldc -l lir -d 'Use the L-IR layout pipeline (S-IR → L-IR → G-IR) instead of direct compilation'
complete -c ldc -s h -l help -d 'Print help (see more with \'--help\')'
complete -c ldc -s V -l version -d 'Print version'
