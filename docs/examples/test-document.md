# LDIR Test Document

This is a **test document** for the LDIR compiler pipeline.

## Introduction

The LDIR project implements a low-level document intermediate representation language, verified by Lean4 formal proofs.

## Features

- S-IR validation with 6 well-formedness checks
- Knuth-Plass line breaking algorithm
- Cassowary constraint solver
- PDF output via pdf-writer
- 1,810 passing tests across 26 crates

## Code Example

```
fn compile(doc: &SIRDocument) -> Result<GIRDocument> {
    let tree = InstructionTree::build(doc)?;
    // ... DFS compilation
}
```

## Conclusion

This document was compiled from Markdown to PDF using the LDIR pipeline.
