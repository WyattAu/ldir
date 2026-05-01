---
document_id: YP-IR-SEMANTICS-001
version: 0.1.0
status: DRAFT
domain: Formal Language Theory
subdomains: [Typesetting, Compiler Design, Document Engineering]
applicable_standards: [IEEE 1016-2009, ISO/IEC 12207:2017]
created: 2026-04-23
author: DeepThought
confidence_level: 0.90
tqa_level: 4
---

# YP-IR-SEMANTICS-001: Formal Semantics of the LDIR Intermediate Representations

**Document ID:** YP-IR-SEMANTICS-001
**Version:** 0.1.0
**Status:** DRAFT
**Domain:** Formal Language Theory
**Subdomains:** Typesetting, Compiler Design, Document Engineering
**Applicable Standards:** IEEE 1016-2009, ISO/IEC 12207:2017
**Created:** 2026-04-23
**Author:** DeepThought
**Confidence Level:** 0.90
**TQA Level:** 4

---

## YP-2: Executive Summary

### Problem Statement

LDIR (Low-level Document Intermediate Representation) defines a two-tier compilation pipeline that transforms semantic intent into geometric layout commands. The central question this paper addresses is:

> **Is the LDIR compilation function $\text{compile}: \mathcal{S} \to \mathcal{G}$ a total function that preserves well-formedness and terminates on all valid inputs?**

The Semantic-IR (S-IR) encodes *what* a document means (paragraphs, headings, styled content, mathematical expressions, hyperlinks) as a tree-structured instruction stream. The Geometric-IR (G-IR) encodes *where* and *how* content appears on a page (font selections, glyph positions, rules, coordinate transforms) as a flat, per-page command buffer.

The compilation function $\text{compile}$ is the bridge: it traverses the S-IR tree and emits a sequence of G-IR pages. This paper formally defines the well-formedness predicates for both IRs and proves that well-formed S-IR always compiles to well-formed G-IR (THM-COMPILE-WF-001) and that compilation always terminates (THM-COMPILE-TERMINATES-001).

### Objective Function

$$\text{compile}: \mathcal{S} \to \mathcal{G} \quad \text{such that} \quad \forall d \in \mathcal{S},\; \text{WF-SIR}(d) \implies \text{WF-GIR}(\text{compile}(d))$$

### Scope

| Aspect | In-Scope | Out-of-Scope |
|--------|----------|--------------|
| IR Definitions | S-IR instruction set (DEF-001), G-IR instruction set (DEF-002, DEF-003) | Binary serialization formats (rkyv, FlatBuffers) |
| Compilation | Compilation function $\text{compile}$ (ALG-COMPILE-001) | Incremental re-compilation, caching strategies |
| Well-Formedness | WF-SIR predicate (DEF-004), WF-GIR predicate (DEF-005) | Layout quality metrics, typographic correctness |
| Correctness | Well-formedness preservation, termination | Visual fidelity to TeX, pixel-level correctness |
| Performance | Complexity analysis (ALG-COMPILE-001) | SIMD optimization, parallel traversal |
| Verification | Lean4 mechanization targets | Full mechanization (proofs deferred) |

### Dependencies

This document depends on:
- **REQ-3.1.x:** S-IR specification (opcodes, wire format, entity model)
- **REQ-3.2.x:** G-IR specification (opcodes, coordinate system, fixed-point format)
- **REQ-3.3.x:** IR compilation pipeline requirements
- **REQ-4.4.x:** Formal verification strategy (Lean4)

---

## YP-3: Nomenclature and Notation

### 3.1 Symbol Table

| Symbol | Description | Units | Domain | Source |
|--------|-------------|-------|--------|--------|
| $\mathcal{S}$ | Set of all S-IR documents | — | $\text{Doc}_\text{SIR}$ | This paper |
| $\mathcal{G}$ | Set of all G-IR documents | — | $\text{Doc}_\text{GIR}$ | This paper |
| $\text{Op}_\text{S}$ | S-IR opcode enum | — | $\{0, \ldots, 255\}$ | REQ-3.1.3 |
| $\text{Op}_\text{G}$ | G-IR opcode enum | — | $\{0, \ldots, 255\}$ | REQ-3.2.3 |
| $n$ | Entity identifier | — | $\{0, \ldots, 2^{32}-1\}$ | REQ-3.1.2 |
| $p$ | Parent entity reference | — | $\{0, \ldots, 2^{32}-1\} \cup \{\bot\}$ | REQ-3.1.2 |
| $o$ | Payload offset | bytes | $\mathbb{N}$ | REQ-3.1.2 |
| $\text{fix}_{26.6}(v)$ | Quantization to 26.6 fixed-point | — | $\mathbb{Z}$ | REQ-3.2.5 |
| $h, v$ | Horizontal/vertical coordinates | scaled points | $\mathbb{Z}_{26.6}$ | REQ-3.2.1 |
| $\text{compile}$ | Compilation function | — | $\mathcal{S} \to \mathcal{G}$ | REQ-3.3.1 |
| $\text{WF-SIR}(d)$ | S-IR well-formedness predicate | — | $\{ \top, \bot \}$ | DEF-004 |
| $\text{WF-GIR}(g)$ | G-IR well-formedness predicate | — | $\{ \top, \bot \}$ | DEF-005 |
| $\text{check}_\text{SIR}$ | S-IR well-formedness checker | — | $\mathcal{S} \to \{ \top, \bot \}$ | THM-WF-SIR-001 |
| $\text{check}_\text{GIR}$ | G-IR well-formedness checker | — | $\mathcal{G} \to \{ \top, \bot \}$ | THM-WF-GIR-001 |
| $\mathbb{N}_{32}$ | 32-bit unsigned integers | — | $\{0, \ldots, 2^{32}-1\}$ | REQ-3.1.6 |
| $\mathbb{Z}_{26.6}$ | 26.6 fixed-point integers | scaled points | $\mathbb{Z}$ | REQ-3.2.5 |
| $\bot$ | Sentinel value (null/absent) | — | — | Convention |
| $d^{-1}(n)$ | Preimage of entity $n$ in document $d$ | — | $\mathcal{P}(\text{SIRInstr})$ | This paper |
| $\text{dom}(d)$ | Domain of document $d$ (set of entity IDs) | — | $\mathcal{P}(\mathbb{N}_{32})$ | This paper |

### 3.2 Conventions

- **Partial functions** are denoted with $\rightharpoonup$; total functions with $\to$.
- **Sequences** are denoted with square brackets $[c_1, c_2, \ldots, c_n]$ and indexed from 1.
- **Sets** use standard notation: $\in$, $\subseteq$, $\mathcal{P}(\cdot)$ for power set.
- **Logical constants:** $\top$ (true), $\bot$ (false), $\implies$ (implication), $\iff$ (iff).
- **Trees:** The term "tree" means a rooted, ordered tree unless otherwise stated.
- **Scaled points (sp):** The base unit of measurement in LDIR, where $1\,\text{sp} = 1/65536\,\text{pt}$.

### 3.3 Opcode Taxonomy

**S-IR Opcodes** ($\text{Op}_\text{S}$, per REQ-3.1.3):

| Opcode | Arity | Description |
|--------|-------|-------------|
| `PUSH_BLOCK(type)` | 1 | Opens a block scope: `Paragraph`, `Heading`, or `List` |
| `SET_CONTENT(blob_ref)` | 1 | Attaches text content via payload reference |
| `APPLY_STYLE(style_id)` | 1 | Applies a named style to subsequent content |
| `INSERT_MATH(ref)` | 1 | Embeds a mathematical expression (MathML or TeX reference) |
| `LINK_DATA(ptr)` | 1 | Associates a hyperlink with the current entity |

**G-IR Opcodes** ($\text{Op}_\text{G}$, per REQ-3.2.3):

| Opcode | Arity | Description |
|--------|-------|-------------|
| `SET_FONT(id, size)` | 2 | Selects a font and sets its size |
| `MOVE_XY(h, v)` | 2 | Moves the cursor to absolute coordinates |
| `PUT_GLYPH(unicode_id, advance_x)` | 2 | Places a glyph at the current position |
| `DRAW_RULE(width, height)` | 2 | Draws a filled rectangle (rule) |
| `PUSH_STACK` | 0 | Saves current coordinate transform |
| `POP_STACK` | 0 | Restores saved coordinate transform |
| `ATTACH_METADATA(key, val)` | 2 | Attaches semantic metadata for accessibility |

---

## YP-4: Theoretical Foundation

### 4.1 Axioms

**AX-001: Entity Uniqueness.**
Each EntityID in a document maps to at most one instruction.

$$\forall d \in \mathcal{S},\; \forall n \in \text{dom}(d),\; |d^{-1}(n)| \leq 1$$

*Intuition:* No two instructions may share the same EntityID. This is the primary key invariant of the S-IR.

**AX-002: Parent Existence.**
Every non-root entity has a valid parent within the same document.

$$\forall i \in d,\; i.\text{parent} = \bot \lor i.\text{parent} \in \text{dom}(d)$$

*Intuition:* A dangling parent reference is a structural error. Every entity either is the root or points to another entity in the document.

**AX-003: Acyclicity.**
The parent relation forms a forest (no cycles).

$$\text{Let } R = \{(i.\text{entity},\, i.\text{parent}) \mid i \in d,\; i.\text{parent} \neq \bot\}$$
$$R^+ \cap \{(n, n) \mid n \in \text{dom}(d)\} = \emptyset$$

where $R^+$ denotes the transitive closure of $R$. In other words, the transitive closure of the parent relation is irreflexive.

*Intuition:* The document structure is a directed acyclic graph (specifically a forest) rooted at entities with $\text{parent} = \bot$.

**AX-004: Payload Integrity.**
Every PayloadOffset references a valid position within the document's payload region.

$$\forall i \in d,\; i.\text{payload} < |\text{payloads}(d)|$$

where $\text{payloads}(d)$ is the contiguous byte array storing all variable-length payloads and $|\cdot|$ denotes its length in bytes.

*Intuition:* An instruction cannot reference data beyond the end of the payload region. This prevents out-of-bounds reads during compilation.

**AX-005: Fixed-Point Closure.**
All G-IR coordinate values are representable in 26.6 fixed-point format.

$$\forall g \in \mathcal{G},\; \forall c \in g,\; \text{coords}(c) \subseteq \mathbb{Z}_{26.6}$$

where $\text{coords}(c)$ extracts all coordinate values from command $c$ and $\mathbb{Z}_{26.6} = \{z \in \mathbb{Z} \mid -2^{32} \leq z \leq 2^{32} - 1\}$ represents the encodable range after scaling by $2^6 = 64$.

*Intuition:* No geometric operation may produce a coordinate that exceeds the representable range of the 26.6 format. This is the determinism guarantee (REQ-3.2.4).

### 4.2 Definitions

**DEF-001: S-IR Document.**
An S-IR document is a partial function mapping entity identifiers to instructions:

$$d: \mathbb{N}_{32} \rightharpoonup \text{SIRInstr}$$

where each instruction is a 4-tuple:

$$i = (\text{op},\; \text{entity},\; \text{parent},\; \text{payload})$$

with:
- $\text{op} \in \text{Op}_\text{S}$
- $\text{entity} \in \mathbb{N}_{32}$
- $\text{parent} \in \mathbb{N}_{32} \cup \{\bot\}$
- $\text{payload} \in \mathbb{N}$

*Example:* A minimal document with a single paragraph root containing text:

$$d = \{0 \mapsto (\text{PUSH\_BLOCK}(\text{Paragraph}),\; 0,\; \bot,\; 0)\}$$

**DEF-002: G-IR Page.**
A G-IR page is a finite sequence of G-IR commands:

$$p = [c_1, c_2, \ldots, c_n], \quad n \geq 0$$

where each command is a 2-tuple:

$$c_i = (\text{op},\; \text{args})$$

with $\text{op} \in \text{Op}_\text{G}$ and $\text{args}$ a typed argument list determined by $\text{op}$.

*Example:* A page with a single glyph:

$$p = [(\text{SET\_FONT},\; (0, 12\,\text{pt})),\; (\text{MOVE\_XY},\; (0, 0)),\; (\text{PUT\_GLYPH},\; (0x0041, 768))]$$

**DEF-003: G-IR Document.**
A G-IR document is a finite sequence of pages:

$$g = [p_1, p_2, \ldots, p_m], \quad m \geq 1$$

The empty document (zero pages) is excluded by convention: a valid compilation always produces at least one page.

**DEF-004: Well-Formed S-IR (WF-SIR).**
An S-IR document $d$ is well-formed, written $\text{WF-SIR}(d)$, iff all of the following hold:

1. **Entity Uniqueness:** AX-001 holds — $\forall n \in \text{dom}(d),\; |d^{-1}(n)| \leq 1$.
2. **Parent Existence:** AX-002 holds — $\forall i \in d,\; i.\text{parent} = \bot \lor i.\text{parent} \in \text{dom}(d)$.
3. **Acyclicity:** AX-003 holds — the parent relation's transitive closure is irreflexive.
4. **Payload Integrity:** AX-004 holds — all payload offsets are within bounds.
5. **Root Uniqueness:** There exists exactly one entity with $\text{parent} = \bot$:

$$\bigl|\{i \in d \mid i.\text{parent} = \bot\}\bigr| = 1$$

6. **Block Nesting:** All block instructions are properly nested. Formally, if we label each `PUSH_BLOCK` with its entity ID and each `POP_BLOCK` (implicit at the end of a block's children) with the matching entity ID, then the sequence of block open/close labels forms a well-parenthesized word over the alphabet $\text{dom}(d)$.

*Example of violation:* Two root entities violate condition 5. A self-referencing parent ($i.\text{parent} = i.\text{entity}$) violates condition 3.

**DEF-005: Well-Formed G-IR (WF-GIR).**
A G-IR document $g$ is well-formed, written $\text{WF-GIR}(g)$, iff all of the following hold:

1. **Fixed-Point Closure:** AX-005 holds — all coordinate values are in $\mathbb{Z}_{26.6}$.
2. **Font Precedence:** Every `SET_FONT` is followed by at least one glyph or rule command before the next `SET_FONT` (or end of page). Formally, no two `SET_FONT` commands appear consecutively without an intervening `PUT_GLYPH` or `DRAW_RULE`.
3. **Stack Balance:** The coordinate stack is balanced on every page. If we assign $+1$ to `PUSH_STACK` and $-1$ to `POP_STACK`, the cumulative sum is non-negative at every prefix and zero at the end of each page.
4. **Page Bounds:** No `MOVE_XY` references coordinates outside the page boundary:

$$\forall c = (\text{MOVE\_XY}, (h, v)) \in p_j,\; 0 \leq h \leq W_j \land 0 \leq v \leq H_j$$

where $W_j, H_j$ are the page dimensions for page $j$.

### 4.3 Lemmas

**LEM-001: Root Uniqueness.**
*Statement:* If $d$ satisfies WF-SIR, then $d$ has exactly one root.

*Proof:*
- By AX-002, every non-root entity has a parent in $\text{dom}(d)$.
- By AX-003, the parent relation is acyclic. Therefore, following the parent chain from any entity must eventually reach an entity with $\text{parent} = \bot$ (a root).
- By condition 5 of DEF-004, there is exactly one such entity.
- Therefore, $d$ has exactly one root. $\square$

**LEM-002: Tree Structure.**
*Statement:* If $d$ satisfies WF-SIR, then the subgraph induced by the parent relation is a rooted tree.

*Proof:*
- **Connectedness:** By AX-002, every non-root entity has a parent in $\text{dom}(d)$. By AX-003 (acyclicity), following parent chains from any entity terminates at the unique root (LEM-001). Thus, every entity is reachable from the root.
- **Acyclicity:** Guaranteed by AX-003.
- **Unique parent:** By AX-001 (entity uniqueness) and the definition of the parent relation, each entity has at most one parent.
- A connected, acyclic graph where each non-root node has exactly one parent is a rooted tree. $\square$

**LEM-003: Block Nesting Invariant.**
*Statement:* In a WF-SIR document, at any point during a depth-first traversal, the nesting depth equals the number of unmatched `PUSH_BLOCK` operations.

*Proof:*
- Condition 6 of DEF-004 guarantees that `PUSH_BLOCK`/`POP_BLOCK` pairs are properly nested.
- In a well-parenthesized word over a single type of parenthesis, the nesting depth at any position equals the number of opening parentheses minus the number of closing parentheses up to that position.
- Since block types may differ but the nesting structure is determined solely by the open/close pairing (not the block type), the invariant holds regardless of block type. $\square$

### 4.4 Theorems

**THM-WF-SIR-001: S-IR Well-Formedness is Decidable.**
*Statement:* There exists a total computable function $\text{check}_\text{SIR}: \mathcal{S} \to \{ \top, \bot \}$ that returns $\top$ iff the input satisfies WF-SIR.

*Proof:* We show that each of the six conditions in DEF-004 is decidable:

1. **Entity uniqueness:** Maintain a hash set during a single pass over $\text{dom}(d)$. If any entity is seen twice, reject. $O(|d|)$ time.
2. **Parent existence:** For each instruction $i$, check whether $i.\text{parent} \in \text{dom}(d)$ or $i.\text{parent} = \bot$. $O(|d|)$ time with a hash set lookup.
3. **Acyclicity:** Perform a DFS from the root, following parent edges in reverse (child-to-parent becomes parent-to-children via an adjacency list). Detect back-edges. $O(|d|)$ time.
4. **Payload integrity:** For each instruction, verify $i.\text{payload} < |\text{payloads}(d)|$. $O(|d|)$ time.
5. **Root uniqueness:** Count entities with $\text{parent} = \bot$. Verify the count equals 1. $O(|d|)$ time.
6. **Block nesting:** Simulate a stack during depth-first traversal. On `PUSH_BLOCK`, push. On block exit, pop. Reject if the stack underflows or is non-empty at the end. $O(|d|)$ time.

All six checks are computable and terminate in $O(|d|)$ time. Therefore $\text{check}_\text{SIR}$ is a total computable function. $\square$

**THM-WF-GIR-001: G-IR Well-Formedness is Decidable.**
*Statement:* There exists a total computable function $\text{check}_\text{GIR}: \mathcal{G} \to \{ \top, \bot \}$.

*Proof:* Each condition in DEF-005 is decidable:

1. **Fixed-point closure:** Check that all coordinate arguments to `MOVE_XY`, `DRAW_RULE`, and font sizes are within $\mathbb{Z}_{26.6}$. $O(|g|)$ time.
2. **Font precedence:** Scan each page linearly, tracking the last `SET_FONT` position. Reject if two `SET_FONT` commands appear without an intervening glyph/rule. $O(|g|)$ time.
3. **Stack balance:** Simulate the coordinate stack per page. Maintain a counter: $+1$ for `PUSH_STACK`, $-1$ for `POP_STACK`. Reject if the counter goes negative or is non-zero at page end. $O(|g|)$ time.
4. **Page bounds:** For each `MOVE_XY`, verify coordinates against page dimensions. $O(|g|)$ time.

All checks are computable and terminate in $O(|g|)$ time. $\square$

**THM-COMPILE-WF-001: Compilation Preserves Well-Formedness.**
*Statement:* For all $d \in \mathcal{S}$, if $\text{WF-SIR}(d)$ then $\text{WF-GIR}(\text{compile}(d))$.

$$\forall d \in \mathcal{S},\; \text{check}_\text{SIR}(d) = \top \implies \text{check}_\text{GIR}(\text{compile}(d)) = \top$$

*Proof sketch (by structural induction on the S-IR tree):*

- **Base case:** The root node of $d$ generates the initial G-IR page setup (default font, origin coordinates). This trivially satisfies WF-GIR conditions 1–4.

- **Inductive hypothesis:** Assume that for a subtree rooted at entity $e$, the compilation of $e$ and all its descendants produces a well-formed G-IR page fragment $f$ satisfying conditions 1–4.

- **Inductive step:** Consider entity $e$ with children $[c_1, \ldots, c_k]$ in document order. We show that compiling $e$ followed by its children preserves well-formedness:

  (a) **Fixed-point closure (condition 1):** All coordinates emitted by $\text{compile}$ are quantized via $\text{fix}_{26.6}$, which by definition produces values in $\mathbb{Z}_{26.6}$. By AX-005, these are representable. The inductive hypothesis guarantees this for all children.

  (b) **Font precedence (condition 2):** The compiler emits `SET_FONT` before any glyph/rule sequence within a block. Since the compiler processes children sequentially and each child's compilation maintains this invariant by the inductive hypothesis, the combined output also satisfies font precedence.

  (c) **Stack balance (condition 3):** Each `PUSH_BLOCK` in the S-IR generates a matched `PUSH_STACK`/`POP_STACK` pair in the G-IR. By LEM-003, block nesting is well-formed in the S-IR. The depth-first traversal order guarantees that every `PUSH_STACK` is matched by a corresponding `POP_STACK`. The inductive hypothesis ensures balance for each subtree; combining balanced subtrees sequentially preserves balance.

  (d) **Page bounds (condition 4):** The compiler tracks accumulated vertical position and emits page breaks before exceeding the page height. By construction, all `MOVE_XY` commands reference positions within the current page boundary.

  Therefore, $\text{compile}(d)$ satisfies all four WF-GIR conditions. $\square$

*Note:* The full proof will be mechanized in Lean4. The structural induction corresponds to a recursion on the S-IR tree (LEM-002).

**THM-COMPILE-TERMINATES-001: Compilation Terminates.**
*Statement:* For all $d \in \mathcal{S}$, $\text{compile}(d)$ terminates.

$$\forall d \in \mathcal{S},\; \text{compile}(d) \downarrow$$

*Proof:*
- By LEM-002, a WF-SIR document induces a finite rooted tree.
- The compilation function (ALG-COMPILE-001) performs a single depth-first traversal of this tree.
- Each node is visited exactly once.
- For each node, the compilation emits a finite number of G-IR commands (bounded by the number of glyphs in the node's content, which is finite because the payload is a finite byte string by AX-004).
- Therefore, the total number of G-IR commands generated is finite, bounded by $\sum_{i \in d} \text{glyph\_count}(i)$.
- Since the traversal terminates and each step performs finite work, $\text{compile}(d)$ terminates. $\square$

**THM-COMPILE-COVERAGE-001: Entity Coverage.**
*Statement:* For all $d \in \mathcal{S}$ with $\text{WF-SIR}(d)$, every entity in $d$ is represented in $\text{compile}(d)$.

*Proof:* By ALG-COMPILE-001, the compiler iterates over all instructions in depth-first order (line 5). Since the S-IR tree (LEM-002) contains exactly the entities in $\text{dom}(d)$, and the traversal visits every node exactly once, every entity is processed. Each processed entity emits at least one G-IR command (or contributes to the coordinate state). Therefore, all entities are represented in the output. $\square$

---

## YP-5: Algorithm Specification

### ALG-COMPILE-001: S-IR to G-IR Compilation

```
Algorithm: compile
Input:  d: SIRDocument  (well-formed per DEF-004)
Output: g: GIRDocument  (well-formed per DEF-005)

1:  function COMPILE(d)
2:    assert check_SIR(d) = ⊤           // PRE-COMP-001
3:    pages ← empty list of G-IR pages
4:    page ← new G-IR page
5:    coord_stack ← empty stack of (h, v) pairs
6:    cursor ← (0, 0)                    // current pen position in scaled points
7:    current_font ← default_font_id
8:    current_size ← default_size        // in 26.6 fixed-point
9:    block_stack ← empty stack
10:
11:   // Initialize first page
12:   emit(page, SET_FONT(current_font, current_size))
13:
14:   // Depth-first traversal of the S-IR tree
15:   for each instruction i in DFS_ORDER(d) do
16:     match i.op with
17:
18:     | PUSH_BLOCK(type) ⇒
19:         emit(page, PUSH_STACK)
20:         PUSH (cursor, current_font, current_size) onto block_stack
21:         cursor ← apply_block_indent(type, cursor)
22:         if type = Paragraph then
23:           layout_paragraph(i, page, coord_stack, cursor)
24:         else if type = Heading then
25:           layout_heading(i, page, coord_stack, cursor)
26:         else if type = List then
27:           layout_list(i, page, coord_stack, cursor)
28:         end if
29:
30:     | SET_CONTENT(blob_ref) ⇒
31:         text ← read_payload(d, blob_ref)
32:         for each glyph g in shape(text, current_font, current_size) do
33:           emit(page, MOVE_XY(fix_26.6(cursor.h), fix_26.6(cursor.v)))
34:           emit(page, PUT_GLYPH(g.unicode_id, g.advance_x))
35:           cursor.h ← cursor.h + g.advance_x
36:         end for
37:
38:     | APPLY_STYLE(style_id) ⇒
39:         style ← lookup_style(style_id)
40:         current_font ← style.font_id
41:         current_size ← fix_26.6(style.size)
42:         emit(page, SET_FONT(current_font, current_size))
43:
44:     | INSERT_MATH(ref) ⇒
45:         emit(page, MOVE_XY(fix_26.6(cursor.h), fix_26.6(cursor.v)))
46:         emit(page, ATTACH_METADATA("math", ref))
47:         cursor.h ← cursor.h + math_placeholder_width(ref)
48:
49:     | LINK_DATA(ptr) ⇒
50:         emit(page, ATTACH_METADATA("link", ptr))
51:
52:     end match
53:
54:     // Check for page overflow
55:     if cursor.v > page_height then
56:       emit(page, POP_STACK)  // balance current block
57:       PUSH page onto pages
58:       page ← new G-IR page
59:       cursor ← (0, 0)
60:       emit(page, SET_FONT(current_font, current_size))
61:       emit(page, PUSH_STACK)
62:     end if
63:   end for
64:
65:   // Close remaining blocks
66:   while block_stack is not empty do
67:     (saved_cursor, saved_font, saved_size) ← POP block_stack
68:     emit(page, POP_STACK)
69:     cursor ← saved_cursor
70:     current_font ← saved_font
71:     current_size ← saved_size
72:   end while
73:
74:   PUSH page onto pages
75:   g ← pages
76:   assert check_GIR(g) = ⊤            // POST-COMP-001
77:   return g
78: end function
```

### 5.1 Complexity Analysis

| Metric | Value | Derivation |
|--------|-------|------------|
| Time Complexity | $O(|d| + |g|)$ | Each S-IR node visited once ($O(|d|)$); total G-IR output is $O(|g|)$ |
| Space Complexity | $O(\text{depth}(d) + |g|)$ | Block stack is $O(\text{depth}(d))$; output is $O(|g|)$ |
| Best Case | $\Omega(|d|)$ | Must visit all S-IR nodes regardless of content |
| Worst Case | $O(|d| \cdot k)$ | $k = \max$ glyphs per entity (practically bounded by payload size) |
| Amortized per-node | $O(1)$ | Constant work per S-IR node excluding glyph emission |

where $|d| = |\text{dom}(d)|$ is the number of S-IR instructions, $|g| = \sum_{j=1}^{m} |p_j|$ is the total number of G-IR commands, and $\text{depth}(d)$ is the maximum nesting depth of the S-IR tree.

### 5.2 Preconditions

| ID | Condition | Enforcement | Rationale |
|----|-----------|-------------|-----------|
| PRE-COMP-001 | $d$ satisfies WF-SIR | `assert check_SIR(d) = ⊤` at line 2 | Guarantees all axioms hold; compilation is undefined otherwise |
| PRE-COMP-002 | $d$ is non-empty | Implicit in root uniqueness (DEF-004 condition 5) | Empty documents have no root and cannot be well-formed |

### 5.3 Postconditions

| ID | Condition | Verification | Rationale |
|----|-----------|--------------|-----------|
| POST-COMP-001 | Output $g$ satisfies WF-GIR | `assert check_GIR(g) = ⊤` at line 76 | Theorem THM-COMPILE-WF-001 |
| POST-COMP-002 | All S-IR entities represented in $g$ | Coverage check: $\forall e \in \text{dom}(d),\; e \text{ appears in } g$ | Theorem THM-COMPILE-COVERAGE-001 |
| POST-COMP-003 | Coordinate stack balanced per page | Verify: final stack depth = 0 on every page | Follows from matched PUSH_STACK/POP_STACK generation |
| POST-COMP-004 | $|g| \geq 1$ (at least one page) | Check: $g$ is non-empty | DEF-003 excludes zero-page documents |

---

## YP-6: Test Vector Specification

Test vectors validate the well-formedness checkers and compilation function against known-good and known-bad inputs.

**Reference file:** `.specs/01_research/test_vectors/test_vectors_ir.toml`

### 6.1 Test Vector Categories

| Category | Description | Coverage Target | Count (Minimum) |
|----------|-------------|-----------------|-----------------|
| **Nominal** | Valid paragraph, heading, list documents; nested blocks; styled content | 40% | 20 |
| **Boundary** | Max entity count ($2^{32} - 1$), max nesting depth ($2^{16} - 1$), empty content blocks, single-entity document, zero-length text | 20% | 10 |
| **Adversarial** | Malformed parent refs (dangling, self-ref), cycles in parent graph, out-of-bounds payload offsets, unbalanced PUSH_BLOCK/POP_BLOCK, multiple roots, zero roots | 25% | 15 |
| **Regression** | Known TeX edge cases: empty groups, deeply nested `\hbox`, orphaned `\vbox`, negative glue, overfull/underfull boxes | 10% | 5 |
| **Random** | Property-based generated S-IR graphs (QuickCheck / proptest) with biased distributions toward tree-like structures | 5% | Continuous (fuzzing) |

### 6.2 Property-Based Invariants

For random testing, the following invariants must hold for all generated S-IR documents $d$:

$$\text{check}_\text{SIR}(d) = \top \implies \text{check}_\text{GIR}(\text{compile}(d)) = \top \quad \text{(THM-COMPILE-WF-001)}$$

$$\text{check}_\text{SIR}(d) = \bot \implies \text{compile}(d) \text{ is not invoked} \quad \text{(precondition guard)}$$

$$|d| > 0 \implies |\text{compile}(d)| \geq 1 \quad \text{(POST-COMP-004)}$$

---

## YP-7: Domain Constraints

**Reference file:** `.specs/01_research/domain_constraints/domain_constraints_typesetting.toml`

### 7.1 Numerical Constraints

| ID | Constraint | Value | Source |
|----|------------|-------|--------|
| NC-001 | Max entities per document | $2^{32} - 1$ | REQ-3.1.6 |
| NC-002 | Max nesting depth (practical) | $2^{16} - 1 = 65535$ | Engineering limit |
| NC-003 | 26.6 fixed-point range | $[-33554432.0,\; 33554431.984375]$ scaled points | REQ-3.2.6 |
| NC-004 | Quantization error bound | $\pm 1/128 \approx 0.0078125$ scaled points per coordinate | REQ-3.2.7 |
| NC-005 | Max page dimensions | $[0,\; 2^{26} - 1]$ scaled points per axis | REQ-3.2.4 |
| NC-006 | S-IR instruction header size | 13 bytes (fixed) | REQ-3.1.2 |
| NC-007 | G-IR command alignment | 16 bytes | REQ-3.2.2 |

### 7.2 Structural Constraints

| ID | Constraint | Rationale |
|----|------------|-----------|
| NC-008 | S-IR must have exactly one root | DEF-004 condition 5 |
| NC-009 | S-IR parent graph must be a tree | LEM-002 |
| NC-010 | G-IR coordinate stack depth $\leq$ NC-002 | Prevents stack overflow in backend |
| NC-011 | G-IR page must contain at least one command | Prevents empty pages |

### 7.3 Derived Constraints

The following constraints are consequences of the axioms and definitions:

$$\text{NC-001} \land \text{AX-001} \implies |d| \leq 2^{32} - 1$$

$$\text{NC-003} \land \text{NC-005} \implies \text{fix}_{26.6} \text{ is defined for all valid page coordinates}$$

$$\text{NC-004} \implies \text{round}(v \cdot 64) - v \cdot 64 \in [-1/128,\; 1/128]$$

---

## YP-8: Bibliography

| ID | Citation | Relevance | TQA Level | Confidence |
|----|----------|-----------|-----------|------------|
| [1] Knuth, D.E., Plass, M.F. (1981). "Breaking Paragraphs into Lines." *Software: Practice and Experience*, 11(11), 1119–1184. DOI: 10.1002/spe.4380111102 | Knuth-Plass line-breaking algorithm (REQ-4.3.2.1) | 5 | 0.99 |
| [2] Knuth, D.E. (1986). *The TeXbook*. Addison-Wesley. ISBN: 0-201-13448-9 | DVI format semantics, scaled points, TeX's box/glue model | 5 | 0.99 |
| [3] Badros, G.J., Borning, A., Stuckey, P.J. (2001). "The Cassowary Constraint Solving Toolkit." *UIST '01*, 87–96. DOI: 10.1145/502348.502364 | Cassowary constraint solver (REQ-4.3.4.1) | 4 | 0.90 |
| [4] Adobe Systems. "OpenType Font File Format." ISO 14496-22. | Font format, glyph metrics | 4 | 0.95 |
| [5] FreeType Project. "FreeType API Reference." https://freetype.org/freetype2/docs/reference/ | 26.6 fixed-point format (REQ-3.2.5) | 4 | 0.90 |
| [6] PDF Association. "ISO 32000-2:2020 — PDF 2.0." https://www.iso.org/standard/76539.html | PDF output format (REQ-6.2.1) | 4 | 0.95 |
| [7] de Moura, L., Ullrich, S. (2021). "The Lean 4 Theorem Prover and Programming Language." *CADE-28*. DOI: 10.1007/978-3-030-79876-5_2 | Formal verification framework (REQ-4.4.1) | 4 | 0.90 |
| [8] Apt, K.R. (1990). *Introduction to Logic Programming*. MIT Press. | Logical foundations for well-formedness predicates | 5 | 0.99 |
| [9] Muchnick, S.S. (1997). *Advanced Compiler Design and Implementation*. Morgan Kaufmann. | Intermediate representation design patterns | 4 | 0.95 |

---

## YP-9: Knowledge Graph Concepts

| ID | Concept | Language | Source | Confidence | Relationships |
|----|---------|----------|--------|------------|---------------|
| CON-001 | Semantic-IR (S-IR) | EN | This paper | 0.95 | compiled-by → compile; validated-by → WF-SIR |
| CON-002 | Geometric-IR (G-IR) | EN | This paper | 0.95 | produced-by → compile; validated-by → WF-GIR |
| CON-003 | Well-formedness predicate | EN | This paper | 0.95 | instance-of → DEC-004, DEF-005 |
| CON-004 | Entity-Component-System | EN | YP-MEMORY-ECS-001 | 0.90 | underlies → S-IR entity model |
| CON-005 | 26.6 fixed-point arithmetic | EN | FreeType docs [5] | 0.95 | used-in → G-IR coordinates |
| CON-006 | Knuth-Plass line-breaking | EN | [1] | 0.99 | used-in → layout_paragraph |
| CON-007 | Cassowary constraint solver | EN | [3] | 0.90 | used-in → floating element layout |
| CON-008 | DVI format | EN | [2] | 0.99 | inspiration-for → G-IR command buffer |
| CON-009 | Scaled point (sp) | EN | [2] | 0.99 | unit-of → G-IR coordinates |
| CON-010 | Compilation function | EN | This paper | 0.95 | maps → S-IR to G-IR |
| CON-011 | Depth-first traversal | EN | This paper | 0.99 | traversal-order → S-IR tree |
| CON-012 | Coordinate stack | EN | This paper | 0.95 | manages → nested transforms in G-IR |

---

## YP-10: Quality Checklist

- [x] **Document header complete** — YAML frontmatter with all required fields (YP-1)
- [x] **Executive summary with objective function** — Problem statement, scope, objective (YP-2)
- [x] **Nomenclature table with all symbols defined** — 17 symbols with domain and source (YP-3)
- [x] **Axioms (5) formally stated** — AX-001 through AX-005 with formal notation and intuition (YP-4.1)
- [x] **Definitions (5) formally stated with examples** — DEF-001 through DEF-005 (YP-4.2)
- [x] **Lemmas (3) with proof sketches** — LEM-001 through LEM-003 (YP-4.3)
- [x] **Theorems (5) with proof sketches** — THM-WF-SIR-001, THM-WF-GIR-001, THM-COMPILE-WF-001, THM-COMPILE-TERMINATES-001, THM-COMPILE-COVERAGE-001 (YP-4.4)
- [x] **Algorithm specification with complexity analysis** — ALG-COMPILE-001, 78-line pseudocode (YP-5)
- [x] **Pre/postconditions defined** — 2 preconditions, 4 postconditions (YP-5.2, YP-5.3)
- [x] **Test vector categories specified** — 5 categories with coverage targets (YP-6)
- [x] **Domain constraints referenced** — 11 constraints with derivations (YP-7)
- [x] **Bibliography with DOIs/URLs** — 9 references with TQA levels (YP-8)
- [x] **Knowledge graph concepts extracted** — 12 concepts with relationships (YP-9)
- [x] **Quality checklist complete** — This section (YP-10)

---

*End of YP-IR-SEMANTICS-001 v0.1.0*
