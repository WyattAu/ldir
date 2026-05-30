use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::diagnostics;
use crate::preview::PreviewManager;
use crate::symbols;

#[derive(Debug)]
struct DocumentState {
    text: String,
    #[allow(dead_code)]
    uri: Url,
}

#[derive(Debug)]
/// The LSP backend that tracks open documents and handles LSP requests.
pub struct Backend {
    client: Client,
    documents: RwLock<HashMap<String, DocumentState>>,
    preview: PreviewManager,
}

impl Backend {
    /// Create a new backend with the given LSP client.
    pub fn new(client: Client) -> Self {
        let output_path = PathBuf::from("/tmp/ldir-preview");
        Self {
            client: client.clone(),
            documents: RwLock::new(HashMap::new()),
            preview: PreviewManager::new(client, output_path),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions::default()),
                rename_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        tracing::info!("ldir-lsp initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        self.preview.shutdown();
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        let uri = doc.uri.clone();
        let uri_str = uri.to_string();
        let diag = diagnostics::compute_diagnostics(&doc.text, &uri);
        let state = DocumentState {
            text: doc.text,
            uri: doc.uri,
        };
        {
            let mut docs = self.documents.write().await;
            docs.insert(uri_str, state);
        }
        self.client.publish_diagnostics(uri, diag, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let uri_str = uri.to_string();
        let text = {
            let mut docs = self.documents.write().await;
            if let Some(state) = docs.get_mut(&uri_str) {
                for change in params.content_changes {
                    state.text = change.text;
                }
                Some(state.text.clone())
            } else {
                None
            }
        };

        let Some(text) = text else {
            return;
        };

        let diag = diagnostics::compute_diagnostics(&text, &uri);
        self.preview.trigger(&text, &uri_str).await;
        self.client.publish_diagnostics(uri, diag, None).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        let uri_str = uri.to_string();
        {
            let mut docs = self.documents.write().await;
            docs.remove(&uri_str);
        }
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let syms = symbols::extract_symbols(&state.text, uri);
        for sym in &syms {
            let r = sym.range;
            if pos.line >= r.start.line
                && pos.line <= r.end.line
                && pos.character >= r.start.character
                && pos.character <= r.end.character
            {
                let name = sym.name.clone();
                let hover_text = match &sym.detail {
                    Some(d) => format!("{name}\n\n{d}"),
                    None => name,
                };
                return Ok(Some(Hover {
                    contents: HoverContents::Scalar(MarkedString::String(hover_text)),
                    range: None,
                }));
            }
        }
        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let ext = crate::detect_extension(uri.path());
        let Some(line_text) = state.text.lines().nth(pos.line as usize) else {
            return Ok(None);
        };
        match ext {
            "tex" => {
                if let Some(label) = extract_ref_label(line_text) {
                    let pattern = format!("\\label{{{label}}}");
                    for (li, l) in state.text.lines().enumerate() {
                        if let Some(col) = l.find(&pattern) {
                            let start = col as u32;
                            let end = start + pattern.len() as u32;
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: li as u32,
                                        character: start,
                                    },
                                    end: Position {
                                        line: li as u32,
                                        character: end,
                                    },
                                },
                            })));
                        }
                    }
                }
            }
            "md" => {
                if let Some(target) = extract_md_link_target(line_text) {
                    for (li, l) in state.text.lines().enumerate() {
                        if l.contains(&format!("# {target}"))
                            || l.contains(&format!("## {target}"))
                            || l.contains(&format!("### {target}"))
                            || l.contains(&format!("#{target}"))
                        {
                            return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                                uri: uri.clone(),
                                range: Range {
                                    start: Position {
                                        line: li as u32,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: li as u32,
                                        character: l.len() as u32,
                                    },
                                },
                            })));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let ext = crate::detect_extension(uri.path());
        let Some(line_text) = state.text.lines().nth(pos.line as usize) else {
            return Ok(None);
        };
        match ext {
            "tex" => {
                if let Some(label) = extract_label_name(line_text) {
                    let ref_cmds = [
                        "\\ref{",
                        "\\eqref{",
                        "\\cref{",
                        "\\Cref{",
                        "\\autoref{",
                        "\\cite{",
                    ];
                    let mut locations = Vec::new();
                    for (doc_uri, doc_state) in docs.iter() {
                        if let Ok(doc_url) = Url::parse(doc_uri) {
                            for (li, line) in doc_state.text.lines().enumerate() {
                                for cmd in &ref_cmds {
                                    let pattern = format!("{cmd}{label}}}");
                                    if let Some(col) = line.find(&pattern) {
                                        let start = col as u32;
                                        let end = start + pattern.len() as u32;
                                        locations.push(Location {
                                            uri: doc_url.clone(),
                                            range: Range {
                                                start: Position {
                                                    line: li as u32,
                                                    character: start,
                                                },
                                                end: Position {
                                                    line: li as u32,
                                                    character: end,
                                                },
                                            },
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    return Ok(Some(locations));
                }
            }
            "md" => {
                if let Some(heading) = extract_md_heading_text(line_text) {
                    let mut locations = Vec::new();
                    let wiki_link = format!("[[{heading}]]");
                    for (doc_uri, doc_state) in docs.iter() {
                        if let Ok(doc_url) = Url::parse(doc_uri) {
                            for (li, line) in doc_state.text.lines().enumerate() {
                                if let Some(col) = line.find(&wiki_link) {
                                    let start = col as u32;
                                    let end = start + wiki_link.len() as u32;
                                    locations.push(Location {
                                        uri: doc_url.clone(),
                                        range: Range {
                                            start: Position {
                                                line: li as u32,
                                                character: start,
                                            },
                                            end: Position {
                                                line: li as u32,
                                                character: end,
                                            },
                                        },
                                    });
                                }
                            }
                        }
                    }
                    return Ok(Some(locations));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let ext = crate::detect_extension(uri.path());
        let Some(line_text) = state.text.lines().nth(pos.line as usize) else {
            return Ok(None);
        };
        let prefix = &line_text[..pos.character as usize];
        let items = match ext {
            "tex" => completion_latex(prefix, &state.text),
            "md" => completion_markdown(prefix),
            "typ" => completion_typst(prefix),
            _ => return Ok(None),
        };
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document.uri;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let syms = symbols::extract_symbols(&state.text, uri);
        Ok(Some(DocumentSymbolResponse::Nested(syms)))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document.uri;
        let pos = params.position;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let ext = crate::detect_extension(uri.path());
        if ext != "tex" {
            return Ok(None);
        }
        let Some(line_text) = state.text.lines().nth(pos.line as usize) else {
            return Ok(None);
        };
        let prefixes = [
            "\\label{",
            "\\ref{",
            "\\eqref{",
            "\\cref{",
            "\\Cref{",
            "\\autoref{",
            "\\cite{",
        ];
        let Some((key, start, end)) = find_key_at_position(line_text, pos.character, &prefixes)
        else {
            return Ok(None);
        };
        Ok(Some(PrepareRenameResponse::RangeWithPlaceholder {
            range: Range {
                start: Position {
                    line: pos.line,
                    character: start,
                },
                end: Position {
                    line: pos.line,
                    character: end,
                },
            },
            placeholder: key,
        }))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let docs = self.documents.read().await;
        let uri = &params.text_document_position.text_document.uri;
        let pos = params.text_document_position.position;
        let new_name = params.new_name;
        let Some(state) = docs.get(&uri.to_string()) else {
            return Ok(None);
        };
        let ext = crate::detect_extension(uri.path());
        if ext != "tex" {
            return Ok(None);
        }
        let Some(line_text) = state.text.lines().nth(pos.line as usize) else {
            return Ok(None);
        };
        let prefixes = [
            "\\label{",
            "\\ref{",
            "\\eqref{",
            "\\cref{",
            "\\Cref{",
            "\\autoref{",
            "\\cite{",
        ];
        let Some((label, _, _)) = find_key_at_position(line_text, pos.character, &prefixes) else {
            return Ok(None);
        };
        let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
        for (doc_uri_str, doc_state) in docs.iter() {
            let Ok(doc_url) = Url::parse(doc_uri_str) else {
                continue;
            };
            let mut edits = Vec::new();
            for (li, line) in doc_state.text.lines().enumerate() {
                for &cmd in &prefixes {
                    let pattern = format!("{cmd}{label}}}");
                    let mut offset = 0;
                    while let Some(idx) = line[offset..].find(&pattern) {
                        let abs_col = offset + idx;
                        let key_start = abs_col + cmd.len();
                        let key_end = abs_col + pattern.len() - 1;
                        edits.push(TextEdit {
                            range: Range {
                                start: Position {
                                    line: li as u32,
                                    character: key_start as u32,
                                },
                                end: Position {
                                    line: li as u32,
                                    character: key_end as u32,
                                },
                            },
                            new_text: new_name.clone(),
                        });
                        offset = abs_col + 1;
                    }
                }
            }
            if !edits.is_empty() {
                changes.insert(doc_url, edits);
            }
        }
        Ok(Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }))
    }
}

fn extract_ref_label(line: &str) -> Option<String> {
    for prefix in ["\\ref{", "\\eqref{", "\\cref{", "\\Cref{", "\\autoref{"] {
        if let Some(start) = line.find(prefix) {
            let rest = &line[start + prefix.len()..];
            if let Some(end) = rest.find('}') {
                let label = rest[..end].trim().to_string();
                if !label.is_empty() {
                    return Some(label);
                }
            }
        }
    }
    None
}

fn extract_md_link_target(line: &str) -> Option<String> {
    if let Some(start) = line.find("[[") {
        let rest = &line[start + 2..];
        if let Some(end) = rest.find("]]") {
            let target = rest[..end].trim().to_string();
            if !target.is_empty() {
                return Some(target);
            }
        }
    }
    None
}

fn extract_label_name(line: &str) -> Option<String> {
    let prefix = "\\label{";
    if let Some(start) = line.find(prefix) {
        let rest = &line[start + prefix.len()..];
        if let Some(end) = rest.find('}') {
            let label = rest[..end].trim().to_string();
            if !label.is_empty() {
                return Some(label);
            }
        }
    }
    None
}

fn find_key_at_position(line: &str, col: u32, prefixes: &[&str]) -> Option<(String, u32, u32)> {
    for &prefix in prefixes {
        let mut offset = 0;
        while let Some(idx) = line[offset..].find(prefix) {
            let abs_idx = offset + idx;
            let key_start = abs_idx + prefix.len();
            let rest = &line[key_start..];
            if let Some(brace_pos) = rest.find('}') {
                let key = rest[..brace_pos].trim().to_string();
                let key_end = key_start + brace_pos;
                if !key.is_empty() && col >= key_start as u32 && col < key_end as u32 {
                    return Some((key, key_start as u32, key_end as u32));
                }
                offset = key_start + brace_pos + 1;
            } else {
                break;
            }
        }
    }
    None
}

fn extract_md_heading_text(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let hashes: &str;
    if let Some(h) = trimmed.strip_prefix("#### ") {
        hashes = h;
    } else if let Some(h) = trimmed.strip_prefix("### ") {
        hashes = h;
    } else if let Some(h) = trimmed.strip_prefix("## ") {
        hashes = h;
    } else if let Some(h) = trimmed.strip_prefix("# ") {
        hashes = h;
    } else {
        return None;
    }
    let heading = hashes.trim().to_string();
    if !heading.is_empty() {
        Some(heading)
    } else {
        None
    }
}

const LATEX_COMMANDS: &[(&str, &str)] = &[
    ("section", "\\section{title}"),
    ("subsection", "\\subsection{title}"),
    ("subsubsection", "\\subsubsection{title}"),
    ("textbf", "\\textbf{text}"),
    ("textit", "\\textit{text}"),
    ("underline", "\\underline{text}"),
    ("emph", "\\emph{text}"),
    ("cite", "\\cite{key}"),
    ("ref", "\\ref{label}"),
    ("label", "\\label{name}"),
    ("eqref", "\\eqref{label}"),
    ("cref", "\\cref{label}"),
    ("autoref", "\\autoref{label}"),
    ("equation", "\\begin{equation}\n\n\\end{equation}"),
    (
        "figure",
        "\\begin{figure}\n\\centering\n\\includegraphics{}\n\\caption{}\n\\end{figure}",
    ),
    (
        "table",
        "\\begin{table}\n\\centering\n\\begin{tabular}{}\n\\end{tabular}\n\\caption{}\n\\end{table}",
    ),
    ("itemize", "\\begin{itemize}\n\\item \n\\end{itemize}"),
    ("enumerate", "\\begin{enumerate}\n\\item \n\\end{enumerate}"),
    ("align", "\\begin{align}\n\n\\end{align}"),
    ("item", "\\item "),
    ("caption", "\\caption{text}"),
    ("centering", "\\centering"),
    ("includegraphics", "\\includegraphics{file}"),
    ("footnote", "\\footnote{text}"),
    ("url", "\\url{link}"),
    ("href", "\\href{url}{text}"),
    ("usepackage", "\\usepackage{name}"),
    ("documentclass", "\\documentclass{cls}"),
    ("bibliography", "\\bibliography{file}"),
    ("bibliographystyle", "\\bibliographystyle{style}"),
    ("input", "\\input{file}"),
    ("include", "\\include{file}"),
    ("title", "\\title{text}"),
    ("author", "\\author{name}"),
    ("date", "\\date{text}"),
    ("maketitle", "\\maketitle"),
    ("tableofcontents", "\\tableofcontents"),
    ("abstract", "\\begin{abstract}\n\n\\end{abstract}"),
    ("proof", "\\begin{proof}\n\n\\end{proof}"),
    ("theorem", "\\begin{theorem}\n\n\\end{theorem}"),
    ("lemma", "\\begin{lemma}\n\n\\end{lemma}"),
    ("definition", "\\begin{definition}\n\n\\end{definition}"),
    ("paragraph", "\\paragraph{title}"),
    ("newline", "\\newline"),
    ("pagebreak", "\\pagebreak"),
    ("noindent", "\\noindent"),
    ("hline", "\\hline"),
    ("cline", "\\cline{}"),
    ("toprule", "\\toprule"),
    ("midrule", "\\midrule"),
    ("bottomrule", "\\bottomrule"),
    ("text", "\\text{}"),
    ("mathrm", "\\mathrm{}"),
    ("mathbf", "\\mathbf{}"),
    ("frac", "\\frac{num}{den}"),
    ("sqrt", "\\sqrt{}"),
    ("int", "\\int"),
    ("sum", "\\sum"),
    ("prod", "\\prod"),
    ("lim", "\\lim"),
    ("sin", "\\sin"),
    ("cos", "\\cos"),
    ("alpha", "\\alpha"),
    ("beta", "\\beta"),
    ("gamma", "\\gamma"),
    ("delta", "\\delta"),
    ("epsilon", "\\epsilon"),
    ("lambda", "\\lambda"),
    ("mu", "\\mu"),
    ("sigma", "\\sigma"),
    ("omega", "\\omega"),
    ("infinity", "\\infty"),
    ("partial", "\\partial"),
    ("nabla", "\\nabla"),
    ("left", "\\left"),
    ("right", "\\right"),
    ("big", "\\big"),
    ("Big", "\\Big"),
];

const LATEX_ENVIRONMENTS: &[(&str, &str)] = &[
    ("document", "\\begin{document}\n\n\\end{document}"),
    ("equation", "\\begin{equation}\n\n\\end{equation}"),
    ("equation*", "\\begin{equation*}\n\n\\end{equation*}"),
    (
        "figure",
        "\\begin{figure}\n\\centering\n\\includegraphics{}\n\\caption{}\n\\end{figure}",
    ),
    (
        "table",
        "\\begin{table}\n\\centering\n\\begin{tabular}{}\n\\end{tabular}\n\\caption{}\n\\end{table}",
    ),
    ("itemize", "\\begin{itemize}\n\\item \n\\end{itemize}"),
    ("enumerate", "\\begin{enumerate}\n\\item \n\\end{enumerate}"),
    ("align", "\\begin{align}\n\n\\end{align}"),
    ("align*", "\\begin{align*}\n\n\\end{align*}"),
    ("cases", "\\begin{cases}\n\n\\end{cases}"),
    ("split", "\\begin{split}\n\n\\end{split}"),
    ("gathered", "\\begin{gathered}\n\n\\end{gathered}"),
    ("array", "\\begin{array}{cc}\n\n\\end{array}"),
    ("tabular", "\\begin{tabular}{cc}\n\n\\end{tabular}"),
    (
        "minipage",
        "\\begin{minipage}{\\textwidth}\n\n\\end{minipage}",
    ),
    ("verbatim", "\\begin{verbatim}\n\n\\end{verbatim}"),
    ("lstlisting", "\\begin{lstlisting}\n\n\\end{lstlisting}"),
    ("quote", "\\begin{quote}\n\n\\end{quote}"),
    ("abstract", "\\begin{abstract}\n\n\\end{abstract}"),
    ("proof", "\\begin{proof}\n\n\\end{proof}"),
    ("theorem", "\\begin{theorem}\n\n\\end{theorem}"),
    ("lemma", "\\begin{lemma}\n\n\\end{lemma}"),
    ("definition", "\\begin{definition}\n\n\\end{definition}"),
    ("corollary", "\\begin{corollary}\n\n\\end{corollary}"),
    ("example", "\\begin{example}\n\n\\end{example}"),
    ("remark", "\\begin{remark}\n\n\\end{remark}"),
    ("multline", "\\begin{multline}\n\n\\end{multline}"),
    ("flalign", "\\begin{flalign}\n\n\\end{flalign}"),
];

fn completion_latex(prefix: &str, full_text: &str) -> Vec<CompletionItem> {
    if let Some(stripped) = prefix.strip_suffix("\\begin{") {
        let typed = stripped.to_lowercase();
        let mut items = Vec::new();
        for &(name, insert) in LATEX_ENVIRONMENTS {
            if name.starts_with(&typed) {
                items.push(CompletionItem {
                    label: name.to_string(),
                    detail: Some(format!("environment {name}")),
                    kind: Some(CompletionItemKind::SNIPPET),
                    insert_text: Some(insert.to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    filter_text: Some(name.to_string()),
                    ..Default::default()
                });
            }
        }
        return items;
    }

    if let Some(rest) = find_in_braces(prefix, "\\ref{")
        .or_else(|| find_in_braces(prefix, "\\cite{"))
        .or_else(|| find_in_braces(prefix, "\\eqref{"))
        .or_else(|| find_in_braces(prefix, "\\cref{"))
        .or_else(|| find_in_braces(prefix, "\\Cref{"))
        .or_else(|| find_in_braces(prefix, "\\autoref{"))
    {
        let typed = rest.to_lowercase();
        return extract_labels(full_text)
            .into_iter()
            .filter(|l| l.to_lowercase().contains(&typed))
            .map(|label| CompletionItem {
                label: label.clone(),
                detail: Some(format!("label: {label}")),
                kind: Some(CompletionItemKind::REFERENCE),
                insert_text: Some(label),
                ..Default::default()
            })
            .collect();
    }

    if let Some(stripped) = prefix.strip_suffix('\\') {
        let typed = stripped.to_lowercase();
        let mut items = Vec::new();
        for &(name, insert) in LATEX_COMMANDS {
            if name.starts_with(&typed) {
                items.push(CompletionItem {
                    label: name.to_string(),
                    detail: Some(insert.to_string()),
                    kind: Some(CompletionItemKind::KEYWORD),
                    insert_text: Some(insert.to_string()),
                    filter_text: Some(name.to_string()),
                    ..Default::default()
                });
            }
        }
        return items;
    }

    Vec::new()
}

fn completion_markdown(prefix: &str) -> Vec<CompletionItem> {
    let trimmed = prefix.trim_start();
    if prefix.len() != trimmed.len() && trimmed.is_empty() {
        return vec![
            CompletionItem {
                label: "#".to_string(),
                detail: Some("Heading 1".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some("# ".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "##".to_string(),
                detail: Some("Heading 2".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some("## ".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "###".to_string(),
                detail: Some("Heading 3".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some("### ".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "####".to_string(),
                detail: Some("Heading 4".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some("#### ".to_string()),
                ..Default::default()
            },
        ];
    }
    Vec::new()
}

fn completion_typst(prefix: &str) -> Vec<CompletionItem> {
    let trimmed = prefix.trim_start();
    if prefix.len() != trimmed.len() && trimmed.is_empty() {
        return vec![
            CompletionItem {
                label: "=".to_string(),
                detail: Some("Heading 1".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some("= ".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "==".to_string(),
                detail: Some("Heading 2".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some("== ".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "===".to_string(),
                detail: Some("Heading 3".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                insert_text: Some("=== ".to_string()),
                ..Default::default()
            },
        ];
    }
    Vec::new()
}

fn find_in_braces<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    if let Some(start) = line.rfind(prefix) {
        let after = &line[start + prefix.len()..];
        if !after.contains('}') {
            return Some(after);
        }
    }
    None
}

fn extract_labels(text: &str) -> Vec<String> {
    let prefix = "\\label{";
    let mut labels = Vec::new();
    for line in text.lines() {
        let mut search = line;
        while let Some(pos) = search.find(prefix) {
            let rest = &search[pos + prefix.len()..];
            if let Some(end) = rest.find('}') {
                labels.push(rest[..end].to_string());
                search = &rest[end + 1..];
            } else {
                break;
            }
        }
    }
    labels.sort();
    labels.dedup();
    labels
}
