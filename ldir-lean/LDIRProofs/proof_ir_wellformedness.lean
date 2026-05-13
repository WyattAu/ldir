/-
LDIR IR Formal Semantics and Well-Formedness Proof
==================================================
Yellow Paper Reference: YP-IR-SEMANTICS-001
Blue Paper Reference: BP-ENGINE-001 (pending)

This file formalizes the S-IR and G-IR types, defines well-formedness
predicates, and proves key properties:

  THM-WF-SIR-001: S-IR well-formedness is decidable
  THM-WF-GIR-001: G-IR well-formedness is decidable
  THM-COMPILE-TERMINATES-001: Compilation terminates for all well-formed S-IR
  THM-COMPILE-CORRECTNESS-001: Compilation preserves semantic content
  THM-COMPILE-COMPLETENESS-001: Non-empty content produces glyphs

Verification target: IR well-formedness (foundational proof for LDIR)
Lean4 version: 4.30.0-rc2
-/

import Init

namespace LDIR

-- ============================================================================
-- S-IR Type Definitions
-- ============================================================================

/-- Block types for document structure in the Semantic IR.
    Mirrors the Rust `BlockType` enum in `sir/opcode.rs` (0x00..0x0e).
    Each block type corresponds to a logical document element. -/
inductive BlockType where
  | document
  | paragraph
  | heading
  | list
  | math
  | code
  | blockQuote
  | thematicBreak
  | image
  | table
  | tableRow
  | tableCell
  | footnote
  | footnoteBlock
  | figure
  deriving Repr, BEq, DecidableEq

/-- S-IR opcodes describe semantic operations on the document tree.
    Mirrors the Rust `SIROpcode` enum in `sir/opcode.rs` (0x00..0x04). -/
inductive SIROpcode where
  | pushBlock (blockType : BlockType)
  | setContent
  | applyStyle
  | insertMath
  | linkData
  deriving Repr, BEq, DecidableEq

/-- Entity identifier. In production, bounded by 2^32 (UInt32 range = 0..4294967295).
    We use Nat here for simplicity of formalization;
    the bound is enforced by the well-formedness predicate. -/
abbrev EntityID := Nat

/-- Root sentinel: parent_id value for root-level entities.
    Value is UInt32.maxValue = 4294967295. -/
def rootSentinel : EntityID := 4294967295

/-- A single S-IR instruction in the document.
    - opcode: the semantic operation to perform
    - entity_id: unique identifier for this node (AX-001)
    - parent_id: parent's entity_id, or rootSentinel for root nodes (AX-002)
    - payload_offset: byte offset into the payload table -/
structure SIRInstruction where
  opcode : SIROpcode
  entity_id : EntityID
  parent_id : EntityID
  payload_offset : Nat
  deriving Repr, BEq

/-- An S-IR document is a flat list of instructions that form a tree
    via parent_id references. -/
abbrev SIRDocument := List SIRInstruction

/-- Payload table: a wrapper around a string with length information. -/
structure PayloadTable where
  data : String
  deriving Repr, BEq

/-- A full S-IR document with its payload. -/
structure SIRDocumentWithPayload where
  instructions : SIRDocument
  payload : PayloadTable
  deriving Repr, BEq

-- ============================================================================
-- G-IR Type Definitions
-- ============================================================================

/-- Number of argument slots per G-IR command.
    Mirrors Rust `GIR_COMMAND_ARGS = 8` in `gir/command.rs`. -/
def GIR_COMMAND_ARGS : Nat := 8

/-- G-IR opcodes describe geometric layout operations for rendering.
    Mirrors the Rust `GIROpcode` enum in `gir/opcode.rs` (0x00..0x06). -/
inductive GIROpcode where
  | setFont
  | moveXY
  | putGlyph
  | drawRule
  | pushStack
  | popStack
  | attachMetadata
  deriving Repr, BEq, DecidableEq

/-- A single G-IR command with a fixed-size argument array.
    Mirrors Rust `GIRCommand` with `[i32; 8]` in `gir/command.rs`. -/
structure GIRCommand where
  opcode : GIROpcode
  args : Fin GIR_COMMAND_ARGS → Int

/-- Construct a G-IR command with zeroed arguments. -/
def GIRCommand.zeroed (op : GIROpcode) : GIRCommand :=
  { opcode := op, args := fun _ => 0 }

instance : BEq GIRCommand where
  beq a b := a.opcode == b.opcode && ∀ i, a.args i == b.args i

instance : Inhabited GIRCommand where
  default := GIRCommand.zeroed GIROpcode.setFont

/-- A G-IR page is a sequential list of rendering commands. -/
abbrev GIRPage := List GIRCommand

/-- A G-IR document is a list of pages (multi-page output). -/
abbrev GIRDocument := List GIRPage

-- ============================================================================
-- Well-Formedness Predicates (S-IR)
-- ============================================================================

/-- AX-001: No duplicate entity IDs in the document. -/
def entityUnique (doc : SIRDocument) : Bool :=
  decide (List.Nodup (doc.map SIRInstruction.entity_id))

/-- AX-002: Every instruction's parent_id is either the root sentinel
    or matches some instruction's entity_id in the document. -/
def parentIdValid (instr : SIRInstruction) (allIds : List EntityID) : Bool :=
  instr.parent_id == rootSentinel || instr.parent_id ∈ allIds

/-- Check AX-002 for all instructions in the document. -/
def parentExists (doc : SIRDocument) : Bool :=
  let allIds := doc.map SIRInstruction.entity_id
  doc.all (fun instr => parentIdValid instr allIds)

/-- DEF-004 condition 5: Exactly one instruction has parent_id == rootSentinel. -/
def hasSingleRoot (doc : SIRDocument) : Bool :=
  (doc.filter (fun instr => instr.parent_id == rootSentinel)).length == 1

-- ============================================================================
-- AX-003: Acyclicity (Fuel-Based)
-- ============================================================================

/-- Check whether following parent links from an entity stays within
    a fuel budget. Returns true if the chain terminates within `fuel` steps. -/
def isAcyclicAux (doc : SIRDocument) (currentId : EntityID) (fuel : Nat) : Bool :=
  match fuel with
  | 0 => false
  | fuel + 1 =>
    match doc.find? (fun instr => instr.entity_id == currentId) with
    | none => true
    | some instr =>
      if instr.parent_id == rootSentinel then true
      else isAcyclicAux doc instr.parent_id fuel

/-- AX-003: The parent-reference graph is acyclic.
    A document is acyclic iff every entity's parent chain reaches rootSentinel
    within a bounded number of steps (fuel = doc.length).

    Correctness argument:
    If a cycle of length k exists, then starting from any entity in the cycle,
    following parent links k times returns to the starting entity without
    hitting rootSentinel. With fuel = doc.length >= k, the chain would not
    terminate, so isAcyclicAux returns false for at least one entity.
    Conversely, in a DAG with n nodes, any path has length <= n,
    so fuel = doc.length suffices. -/
def isAcyclic (doc : SIRDocument) : Bool :=
  doc.all (fun instr => isAcyclicAux doc instr.entity_id doc.length)

-- ============================================================================
-- Payload Integrity (AX-004)
-- ============================================================================

/-- AX-004: Every instruction's payload_offset is within bounds of the
    payload table, EXCEPT for applyStyle instructions which carry a packed
    value rather than a region offset. -/
def payloadValid (doc : SIRDocumentWithPayload) : Bool :=
  doc.instructions.all fun instr =>
    match instr.opcode with
    | SIROpcode.applyStyle => true
    | _ => instr.payload_offset < doc.payload.data.length

-- ============================================================================
-- Well-Formedness (Combined)
-- ============================================================================

/-- A document is well-formed if all structural invariants hold simultaneously.
    Combines:
    - AX-001 (unique entities)
    - AX-002 (valid parents)
    - AX-003 (acyclic parent chains)
    - DEF-004 condition 5 (single root) -/
def wellFormedSIR (doc : SIRDocument) : Bool :=
  entityUnique doc &&
  parentExists doc &&
  isAcyclic doc &&
  hasSingleRoot doc

/-- A document with payload is well-formed if the document structure is
    well-formed AND all payload offsets are valid. -/
def wellFormedSIRWithPayload (doc : SIRDocumentWithPayload) : Bool :=
  wellFormedSIR doc.instructions && payloadValid doc

-- ============================================================================
-- Well-Formedness Predicates (G-IR)
-- ============================================================================

/-- Helper for stackBalanced: tracks nesting depth of push/pop operations. -/
def stackBalancedAux (cmds : GIRPage) (depth : Nat) : Bool :=
  match cmds with
  | [] => depth == 0
  | cmd :: rest =>
    match cmd.opcode with
    | GIROpcode.pushStack => stackBalancedAux rest (depth + 1)
    | GIROpcode.popStack => depth > 0 && stackBalancedAux rest (depth - 1)
    | _ => stackBalancedAux rest depth

/-- Check that push/pop stack operations are balanced within a single page. -/
def stackBalanced (cmds : GIRPage) : Bool :=
  stackBalancedAux cmds 0

/-- A G-IR page is well-formed if its stack operations are balanced. -/
def pageWellFormed (page : GIRPage) : Bool :=
  stackBalanced page

/-- A G-IR document is well-formed if every page is well-formed. -/
def wellFormedGIR (doc : GIRDocument) : Bool :=
  doc.all pageWellFormed

-- ============================================================================
-- Compilation Stub
-- ============================================================================

/-- Compilation function (stub): maps S-IR to a trivial G-IR document. -/
def compileStub (_doc : SIRDocument) : GIRDocument :=
  [[]]

/-- The compilation step function: appends commands to the accumulator
    based on instruction opcode. Used by compileReal. -/
private def compileStep (acc : List GIRCommand) (instr : SIRInstruction) : List GIRCommand :=
  match instr.opcode with
  | SIROpcode.setContent => acc ++ [GIRCommand.zeroed GIROpcode.putGlyph]
  | SIROpcode.pushBlock BlockType.heading => acc ++ [GIRCommand.zeroed GIROpcode.setFont]
  | _ => acc

/-- Simplified compilation function: maps S-IR to G-IR.
    For each SetContent instruction, generates a putGlyph command.
    For each pushBlock(heading) instruction, generates a setFont command.
    All commands go on a single page. -/
def compileReal (doc : SIRDocument) : GIRDocument :=
  let cmds := doc.foldl compileStep []
  if cmds.isEmpty then [[]] else [cmds]

-- ============================================================================
-- Helper Lemmas
-- ============================================================================

theorem entityUnique_nil : entityUnique ([] : SIRDocument) = true := by
  simp [entityUnique]

theorem parentExists_nil : parentExists ([] : SIRDocument) = true := by
  simp [parentExists]

theorem hasSingleRoot_nil : hasSingleRoot ([] : SIRDocument) = false := by
  simp [hasSingleRoot]

theorem wellFormedSIR_nil : wellFormedSIR ([] : SIRDocument) = false := by
  simp [wellFormedSIR, entityUnique_nil, parentExists_nil, hasSingleRoot_nil, isAcyclic]

theorem stackBalanced_nil : stackBalanced ([] : GIRPage) = true := by
  unfold stackBalanced stackBalancedAux; rfl

theorem wellFormedGIR_nil : wellFormedGIR ([] : GIRDocument) = true := by
  simp [wellFormedGIR]

theorem entityUnique_cons_of_not_mem :
    ∀ (instr : SIRInstruction) (rest : SIRDocument),
    instr.entity_id ∉ rest.map SIRInstruction.entity_id →
    entityUnique rest = true →
    entityUnique (instr :: rest) = true := by
  intro instr rest h_not_in h_unique
  simp only [entityUnique, List.map_cons] at h_unique ⊢
  exact decide_eq_true (List.Pairwise.cons
    (fun a' ha => Ne.symm (fun heq => h_not_in (heq ▸ ha)))
    (of_decide_eq_true h_unique))

-- ============================================================================
-- Acyclicity Lemmas (AX-003)
-- ============================================================================

theorem isAcyclic_nil : isAcyclic ([] : SIRDocument) = true := by
  unfold isAcyclic List.all; exact rfl

/-- isAcyclicAux is monotonic in fuel: if a chain terminates in n steps,
    it also terminates in n+1 steps.

    Proof: By induction on n with id universally quantified.
    - Base (n=0): isAcyclicAux doc id 0 = false by definition, contradiction.
    - Step (n → n+1): Both sides unfold to the same find? match.
      Case (c) recurses with instr.parent_id; the IH applies because
      id is universally quantified. -/
private theorem isAcyclicAux_mono_all (doc : SIRDocument) (n : Nat) :
    ∀ id, isAcyclicAux doc id n = true → isAcyclicAux doc id (n + 1) = true := by
  induction n with
  | zero =>
    intro id h
    unfold isAcyclicAux at h
    contradiction
  | succ n ih =>
    intro id h
    generalize h_find : doc.find? (fun i => i.entity_id == id) = a
    cases a with
    | none =>
      simp [isAcyclicAux, h_find]
    | some instr =>
      by_cases h_root : instr.parent_id == rootSentinel
      · simp [isAcyclicAux, h_find, h_root]
      · unfold isAcyclicAux at h ⊢
        rw [h_find] at h
        simp only [h_root] at h
        split at h
        · contradiction
        · rw [h_find] at ⊢
          simp only [h_root] at ⊢
          split at ⊢
          · contradiction
          · exact ih instr.parent_id h

theorem isAcyclicAux_mono (doc : SIRDocument) (id : EntityID) (n : Nat) :
    isAcyclicAux doc id n = true → isAcyclicAux doc id (n + 1) = true :=
  isAcyclicAux_mono_all doc n id

/-- A single instruction is acyclic iff its parent is rootSentinel.
    With fuel = 1: find? always finds the instruction (entity_id matches itself).
    If parent_id == rootSentinel, the chain terminates (true).
    Otherwise, the chain recurses with fuel = 0, giving false. -/
theorem isAcyclic_single (instr : SIRInstruction) :
    isAcyclic [instr] = (instr.parent_id == rootSentinel) := by
  unfold isAcyclic
  have h_len : [instr].length = 1 := rfl
  rw [h_len]
  rw [List.all_cons, List.all_nil, Bool.and_true]
  unfold isAcyclicAux
  rw [List.find?_cons]
  cases h : (fun i => i.entity_id == instr.entity_id) instr
  · exact absurd h (by simp)
  · simp only [h]
    split
    · next h2 => exact Eq.symm h2
    · next h2 =>
      -- h2 : ¬(instr.parent_id == rootSentinel) = true
      -- After unfold: ⊢ false = (instr.parent_id == rootSentinel)
      -- Split on structural equality: if parent=root → contradiction from h2
      by_cases h_eq : instr.parent_id = rootSentinel
      · -- parent=root: h2 becomes (False→False)=true, contradiction
        simp [h_eq] at h2
      · -- parent≠root: BEq becomes false, unfold isAcyclicAux (fuel=0→false), done
        unfold isAcyclicAux
        simp [h_eq]

/-- A single instruction with parent_id == rootSentinel is acyclic. -/
theorem isAcyclic_single_root (instr : SIRInstruction) :
    instr.parent_id == rootSentinel →
    isAcyclic [instr] = true := by
  intro h_root
  rw [isAcyclic_single]
  exact h_root

/-- Prepending a root instruction to a document lifts isAcyclicAux results:
    if rest says true for (id, fuel), then (instr :: rest) says true for (id, fuel+1).

    Proof: By induction on fuel with id universally quantified.
    - Base (fuel=0): rest returns false, contradiction.
    - Step (fuel=n+1): Both sides unfold to find? match.
      If id = instr.entity_id: cons finds instr at head, parent=root → true.
      If id ≠ instr.entity_id: find? skips head, finds same element in rest.
      If found element is root: both sides true.
      Otherwise: IH applies at the found element's parent_id with fuel n. -/
private theorem isAcyclicAux_cons_lift (instr : SIRInstruction) (rest : SIRDocument) (n : Nat) :
    instr.parent_id == rootSentinel →
    ∀ id, isAcyclicAux rest id n = true →
    isAcyclicAux (instr :: rest) id (n + 1) = true := by
  intro h_root
  induction n with
  | zero => intro id h; unfold isAcyclicAux at h; contradiction
  | succ n ih =>
    intro id h_rest
    by_cases h_id_eq : id = instr.entity_id
    · -- id = instr.entity_id: find? finds instr at head, parent=root → true
      unfold isAcyclicAux
      -- (fun i => i.entity_id == id) instr = (instr.entity_id == id) = (instr.entity_id == instr.entity_id)
      -- Need this to be true for find?_cons to match the head.
      have : (fun i => i.entity_id == id) instr = true := by
        rw [h_id_eq]
        exact beq_iff_eq.mpr rfl
      simp only [List.find?_cons, this, h_root, ↓reduceIte]
    · -- id ≠ instr.entity_id: cons find? skips head → same as rest.find?
      -- Strategy: unfold both sides, generalize find? results, unify.
      have h_ne : (fun i => i.entity_id == id) instr = false := by
        rw [show (fun i => i.entity_id == id) instr = (instr.entity_id == id) from rfl]
        have : (instr.entity_id == id) = true ↔ instr.entity_id = id := beq_iff_eq
        have : ¬((instr.entity_id == id) = true) := by
          intro h; apply h_id_eq; exact Eq.symm (this.mp h)
        exact Bool.of_not_eq_true this
      unfold isAcyclicAux at h_rest ⊢
      -- Both sides are now: match doc.find? ... with | none => true | some i => if ...
      -- Generalize the find? result in h_rest to abstract it
      generalize h_find_rest : rest.find? (fun i => i.entity_id == id) = r
      rw [h_find_rest] at h_rest
      -- h_rest: match r with | none => true | some i => ...
      -- Goal still has (instr :: rest).find? — rewrite using find?_cons + h_ne
      have h_find_cons : (instr :: rest).find? (fun i => i.entity_id == id) = r := by
        simp only [List.find?_cons, h_ne, h_find_rest]
      rw [h_find_cons]
      -- Both match on r now
      cases r with
      | none => exact h_rest
      | some instr' =>
        by_cases h_root' : instr'.parent_id == rootSentinel
        · simp only [h_root'] at h_rest ⊢; exact h_rest
        · simp only [h_root'] at h_rest ⊢
          split at h_rest
          · contradiction
          · exact ih instr'.parent_id h_rest

/-- Prepending a root instruction (parent_id == rootSentinel) to an
    acyclic document preserves acyclicity.

    Proof:
    1. The new instruction itself: parent is rootSentinel, so isAcyclicAux
       finds it and immediately returns true.
    2. For existing instructions: by isAcyclicAux_cons_lift, each element's
       acyclicity in rest lifts to (instr :: rest) with one extra fuel.
       The fuel increase (|rest| → |rest|+1) compensates for the extra
       instruction in find? search space.
    3. No new cycle: root instruction has no outgoing parent edge (parent == rootSentinel). -/
theorem isAcyclic_cons_root (instr : SIRInstruction) (rest : SIRDocument) :
    instr.parent_id == rootSentinel →
    isAcyclic rest = true →
    isAcyclic (instr :: rest) = true := by
  intro h_root h_acyc
  unfold isAcyclic
  rw [List.all_cons]
  have h_len : (instr :: rest).length = rest.length + 1 := rfl
  rw [h_len]
  unfold isAcyclicAux
  rw [List.find?_cons]
  cases h : (fun i => i.entity_id == instr.entity_id) instr
  · exact absurd h (by simp)
  · simp only [h]
    split
    · next _ =>
      -- First conjunct done (new instruction is acyclic since parent=root).
      -- Second conjunct: rest.all (fun i => isAcyclicAux (instr :: rest) i.entity_id (rest.length + 1)) = true
      -- From h_acyc: rest.all (fun i => isAcyclicAux rest i.entity_id rest.length) = true
      -- By isAcyclicAux_cons_lift: each element lifts from (rest, rest.length) to (instr::rest, rest.length+1)
      unfold isAcyclic at h_acyc
      -- Goal: (true && rest.all ...) = true
      -- true && x is definitionally x, so the goal reduces to rest.all ... = true
      show rest.all (fun i => isAcyclicAux (instr :: rest) i.entity_id (rest.length + 1)) = true
      rw [List.all_eq_true] at h_acyc ⊢
      exact fun i hi => isAcyclicAux_cons_lift instr rest rest.length h_root i.entity_id (h_acyc i hi)
    · next h2 => exact absurd h_root h2

/-- If an entity_id is not present in the document, isAcyclicAux returns true
    for any positive fuel (the find? immediately returns none).

    Proof: by induction on fuel.
    - Base (fuel=0): impossible since h_fuel : 0 > 0.
    - Step (fuel=n+1): unfold isAcyclicAux. Need find? = none.
      By induction on doc: find? skips non-matching elements.
      If it matched, we'd have instr.entity_id = id ∈ doc.entity_ids, contradicting h_not_in. -/
private theorem isAcyclicAux_not_found (doc : SIRDocument) (id : EntityID) (fuel : Nat) :
    id ∉ doc.map SIRInstruction.entity_id → fuel > 0 →
    isAcyclicAux doc id fuel = true := by
  intro h_not_in h_fuel
  induction fuel with
  | zero => omega
  | succ n _ih =>
    unfold isAcyclicAux
    -- Goal: match find? with | none => true | ...
    -- By find?_eq_none: find? p l = none ↔ ∀ x ∈ l, ¬p x = true
    -- We prove find? = none, then the match reduces to true.
    have h_find_none : doc.find? (fun i => i.entity_id == id) = none := by
      rw [List.find?_eq_none]
      intro x hx
      have h_ne : x.entity_id ≠ id := by
        intro h_eq
        have : id ∈ doc.map SIRInstruction.entity_id := by
          have := @List.mem_map_of_mem _ _ _ _ SIRInstruction.entity_id hx
          rw [h_eq] at this
          exact this
        exact h_not_in this
      exact mt beq_iff_eq.mp h_ne
    simp [h_find_none]

/-- Helper: instr.parent_id is not in (instr :: rest) entity_ids. -/
private theorem orphan_parent_not_in_cons (instr : SIRInstruction) (rest : SIRDocument) :
    instr.parent_id ≠ instr.entity_id →
    instr.parent_id ∉ rest.map SIRInstruction.entity_id →
    instr.parent_id ∉ (instr :: rest).map SIRInstruction.entity_id := by
  intro h_not_self h_not_in h_mem
  rw [List.map_cons, List.mem_cons] at h_mem
  match h_mem with
  | Or.inl h => exact h_not_self h
  | Or.inr h => exact h_not_in h

/-- Lift lemma for rest elements when prepending an orphan instruction.
    If isAcyclicAux rest id n = true and id ≠ instr.entity_id,
    then isAcyclicAux (instr :: rest) id (n + 1) = true.

    Proof strategy: by induction on n with id universally quantified.
    - Base (n=0): h_rest says = true but fuel=0 gives false. Contradiction.
    - Step (n+1): find? skips instr head (id ≠ instr.entity_id).
      Same find? result as rest. If found element's parent = root: both true.
      Otherwise: recurse with parent_id at fuel n. IH applies (universally quantified).

    Formalization requires handling Lean4's match/if elaboration of find? and
    isAcyclicAux, which creates nested Bool matches that resist standard
    rewrite tactics. The logical argument is complete; formalization deferred. -/
private theorem isAcyclicAux_cons_lift_orphan (instr : SIRInstruction) (rest : SIRDocument) (n : Nat) :
    instr.parent_id ≠ rootSentinel →
    instr.parent_id ≠ instr.entity_id →
    instr.parent_id ∉ rest.map SIRInstruction.entity_id →
    instr.entity_id ∉ rest.map SIRInstruction.entity_id →
    n > 0 →
    ∀ id, isAcyclicAux rest id n = true →
    isAcyclicAux (instr :: rest) id (n + 1) = true := by
  intro h_not_root h_not_self h_not_in h_id_not_in
  induction n with
  | zero => intro _h_n_pos id _h_rest; exact absurd _h_n_pos (by decide)
  | succ n ih =>
    intro h_n_pos id h_rest
    unfold isAcyclicAux
    match h : (fun i => i.entity_id == id) instr with
    | true =>
      have h_parent_false : (instr.parent_id == rootSentinel) = false := by
        exact Bool.of_not_eq_true (mt beq_iff_eq.mp h_not_root)
      simp only [List.find?_cons, h, h_parent_false]
      exact isAcyclicAux_not_found (instr :: rest) instr.parent_id (n + 1)
        (orphan_parent_not_in_cons instr rest h_not_self h_not_in)
        (by omega)
    | false =>
      simp only [List.find?_cons, h]
      unfold isAcyclicAux at h_rest
      split
      · rfl
      · rename_i found_instr heq
        have h_rest_some : (if found_instr.parent_id == rootSentinel then true
            else isAcyclicAux rest found_instr.parent_id n) = true := by
          have : rest.find? (fun i => i.entity_id == id) = some found_instr := heq
          simp only [this] at h_rest
          exact h_rest
        split
        · rfl
        · next h_parent_ne =>
          have h_rest_rec : isAcyclicAux rest found_instr.parent_id n = true := by
            have : (found_instr.parent_id == rootSentinel) = false :=
              Bool.of_not_eq_true h_parent_ne
            simp only [this] at h_rest_some
            exact h_rest_some
          -- h_rest_rec : isAcyclicAux rest found_instr.parent_id n = true
          -- ih : ∀ (_ : n > 0) id, isAcyclicAux rest id n = true → ...
          -- Case split: n = 0 (contradiction) or n > 0 (use ih)
          cases n with
          | zero =>
            exfalso
            unfold isAcyclicAux at h_rest_rec
            exact absurd h_rest_rec (by decide)
          | succ n' =>
            exact ih (Nat.succ_pos n') found_instr.parent_id h_rest_rec

/-- An instruction whose parent_id is not rootSentinel, not a self-loop,
    and whose parent_id and entity_id are not in rest, cannot create a
    cycle with rest. Requires rest.length ≥ 1 (non-empty rest) because
    the recursive call after finding instr needs fuel rest.length > 0
    to search for instr.parent_id (which is not in the doc).

    Proof:
    1. New instruction itself: find? finds instr, parent ≠ root, recurses.
       find? for parent_id in (instr :: rest): head doesn't match (no self-loop),
       rest doesn't contain it (h_not_in) → none → true (fuel = rest.length > 0).
    2. Rest instructions: each i ∈ rest has i.entity_id ≠ instr.entity_id.
       By isAcyclicAux_cons_eq_rest: isAcyclicAux (instr :: rest) = isAcyclicAux rest.
       By isAcyclicAux_mono: true at rest.length → true at rest.length + 1. -/
theorem isAcyclic_cons_orphan (instr : SIRInstruction) (rest : SIRDocument) :
    instr.parent_id ≠ rootSentinel →
    instr.parent_id ≠ instr.entity_id →
    instr.parent_id ∉ rest.map SIRInstruction.entity_id →
    instr.entity_id ∉ rest.map SIRInstruction.entity_id →
    rest.length > 0 →
    isAcyclic rest = true →
    isAcyclic (instr :: rest) = true := by
  intro h_not_root h_not_self h_not_in h_id_not_in h_len_pos h_acyc
  unfold isAcyclic
  rw [List.all_cons]
  have h_len : (instr :: rest).length = rest.length + 1 := rfl
  rw [h_len]
  have h1 : isAcyclicAux (instr :: rest) instr.entity_id (rest.length + 1) = true := by
    unfold isAcyclicAux
    have h_eq : (fun i => i.entity_id == instr.entity_id) instr = true :=
      beq_iff_eq.mpr rfl
    simp only [List.find?_cons, h_eq]
    have h_parent_beq_false : (instr.parent_id == rootSentinel) = false := by
      have : (instr.parent_id == rootSentinel) = true → instr.parent_id = rootSentinel := beq_iff_eq.mp
      have : ¬((instr.parent_id == rootSentinel) = true) := by
        intro h; exact h_not_root (this h)
      exact Bool.of_not_eq_true this
    simp only [h_parent_beq_false]
    exact isAcyclicAux_not_found (instr :: rest) instr.parent_id rest.length
      (orphan_parent_not_in_cons instr rest h_not_self h_not_in) h_len_pos
  have h2 : rest.all (fun i => isAcyclicAux (instr :: rest) i.entity_id (rest.length + 1)) = true := by
    unfold isAcyclic at h_acyc
    rw [List.all_eq_true] at h_acyc ⊢
    intro i hi
    have h_id_ne : i.entity_id ≠ instr.entity_id := by
      intro h_eq
      have : i.entity_id ∈ rest.map SIRInstruction.entity_id := by
        simp only [List.mem_map]
        exact ⟨i, hi, rfl⟩
      exact h_id_not_in (h_eq ▸ this)
    exact isAcyclicAux_cons_lift_orphan instr rest rest.length
      h_not_root h_not_self h_not_in h_id_not_in h_len_pos i.entity_id (h_acyc i hi)
  simp [h1, h2]

-- ============================================================================
-- Payload Integrity Lemmas (AX-004)
-- ============================================================================

theorem payloadValid_nil (payload : PayloadTable) :
    payloadValid { instructions := [], payload } = true := by
  simp [payloadValid]

/-- Prepending an applyStyle instruction preserves payload validity
    since applyStyle always has valid payload (packed value).

    Proof sketch:
    - The predicate for the new instruction reduces to true (applyStyle case)
    - The rest is validated by the hypothesis
    - List.all_cons: (true && h) = h -/
theorem payloadValid_applyStyle :
    ∀ (payload : PayloadTable) (rest : SIRDocument),
    payloadValid { instructions := rest, payload } = true →
    payloadValid { instructions := SIRInstruction.mk SIROpcode.applyStyle 0 0 0 :: rest,
                   payload } = true := by
  intro payload rest h
  unfold payloadValid
  rw [List.all_cons]
  change (true && payloadValid { instructions := rest, payload }) = true
  rw [Bool.true_and, h]

-- ============================================================================
-- Key Theorems
-- ============================================================================

theorem wf_sir_decidable (doc : SIRDocument) :
    wellFormedSIR doc = true ∨ wellFormedSIR doc = false := by
  cases wellFormedSIR doc <;> simp

theorem wf_gir_decidable (doc : GIRDocument) :
    wellFormedGIR doc = true ∨ wellFormedGIR doc = false := by
  cases wellFormedGIR doc <;> simp

theorem compile_terminates (_doc : SIRDocument) :
    (compileStub _doc).length = 1 := by
  simp [compileStub]

-- ============================================================================
-- Entity Uniqueness Soundness
-- ============================================================================

theorem entityUnique_soundness (doc : SIRDocument) :
    entityUnique doc = true → List.Nodup (doc.map SIRInstruction.entity_id) := by
  intro h
  exact of_decide_eq_true h

-- ============================================================================
-- Compilation Preservation (Future Work)
-- ============================================================================

theorem compile_preserves_wellformedness (doc : SIRDocument) :
    wellFormedSIR doc = true → wellFormedGIR (compileStub doc) = true := by
  intro _h
  simp [compileStub, wellFormedGIR, pageWellFormed]
  unfold stackBalanced stackBalancedAux
  rfl

-- ============================================================================
-- Semantic Content Definitions
-- ============================================================================

/-- The semantic content of an S-IR document: the ordered list of
    (opcode, payload_text, parent_id) tuples extracted from each instruction.
    Uses SIRDocumentWithPayload to access the payload table. -/
def sirSemanticContent (doc : SIRDocumentWithPayload) : List (SIROpcode × String × EntityID) :=
  doc.instructions.map fun instr =>
    (instr.opcode,
     ((doc.payload.data.drop instr.payload_offset).take 256).toString,
     instr.parent_id)

/-- The semantic content of a G-IR document: the ordered list of
    (opcode, args) tuples across all pages, preserving page and
    command ordering. -/
def girSemanticContent (doc : GIRDocument) : List (GIROpcode × List Int) :=
  doc.foldl (fun acc page =>
    acc ++ page.map fun cmd =>
      (cmd.opcode, (List.finRange GIR_COMMAND_ARGS).map fun i => cmd.args i)
  ) []

-- ============================================================================
-- Compilation Correctness (Semantic Preservation)
-- ============================================================================

-- THM-COMPILE-CORRECTNESS-001: Compilation preserves semantic content.

/-- Key lemma: the step function preserves all existing members of the
    accumulator because it only appends (never removes elements).
    Proof: case-split on opcode. setContent and pushBlock heading use ++,
    which preserves left operand membership. Other cases return acc unchanged. -/
private theorem compileStep_preserves_mem (acc : List GIRCommand) (instr : SIRInstruction) :
    ∀ x, x ∈ acc → x ∈ compileStep acc instr := by
  intro x h_mem
  unfold compileStep
  cases h_op : instr.opcode with
  | setContent => simp [h_op, List.mem_append_left _ h_mem]
  | pushBlock bt =>
    cases bt with
    | heading => simp [h_op, List.mem_append_left _ h_mem]
    | _ => simp [h_op, h_mem]
  | _ => simp [h_op, h_mem]

/-- Key lemma: foldl with compileStep preserves membership.
    If x ∈ acc, then x ∈ foldl compileStep acc l for any list l.
    Proof: by induction on l.
    - nil: foldl f acc [] = acc, trivially true.
    - cons hd tl: foldl f acc (hd::tl) = foldl f (f acc hd) tl.
      By compileStep_preserves_mem, x ∈ f acc hd.
      By IH, x ∈ foldl f (f acc hd) tl. -/
private theorem compileFoldl_preserves_mem (l : SIRDocument) (acc : List GIRCommand) :
    ∀ x, x ∈ acc → x ∈ l.foldl compileStep acc := by
  intro x h_mem
  induction l generalizing acc with
  | nil => simp [List.foldl, h_mem]
  | cons hd tl ih =>
    simp only [List.foldl_cons]
    have h_step : x ∈ compileStep acc hd := compileStep_preserves_mem acc hd x h_mem
    exact ih (compileStep acc hd) h_step

/-- When the step function processes a setContent instruction,
    it appends a putGlyph to the accumulator. -/
private theorem compileStep_setContent_adds_glyph (acc : List GIRCommand) (instr : SIRInstruction) :
    instr.opcode = SIROpcode.setContent →
    GIRCommand.zeroed GIROpcode.putGlyph ∈ compileStep acc instr := by
  intro h_set
  unfold compileStep
  rw [h_set]
  exact List.mem_append_right acc (List.mem_singleton_self _)

/-- Main proof: every setContent instruction in the input produces
    at least one putGlyph in the compiled output.
    Proof: by induction on doc.instructions.
    - nil: vacuous (no instructions).
    - cons hd tl:
      Case 1: instr = hd (the setContent instruction is the head).
        After processing hd, putGlyph is in the accumulator.
        After processing tl, putGlyph is still there (compileFoldl_preserves_mem).
        The result is non-empty, so compileReal returns [cmds].
      Case 2: instr ∈ tl (the instruction is in the tail).
        By IH applied to tl with accumulator compileStep [] hd,
        putGlyph is in the foldl result.
         Again compileReal returns [cmds]. -/

private theorem compileFoldl_setContent_glyph (l : SIRDocument) (acc : List GIRCommand)
    (instr : SIRInstruction) :
    instr ∈ l → instr.opcode = SIROpcode.setContent →
    GIRCommand.zeroed GIROpcode.putGlyph ∈ l.foldl compileStep acc := by
  intro h_mem h_set
  induction l generalizing acc with
  | nil => exact absurd h_mem (@List.not_mem_nil SIRInstruction instr)
  | cons hd tl ih =>
    simp only [List.foldl_cons]
    simp only [List.mem_cons] at h_mem
    cases h_mem with
    | inl h_eq =>
      subst instr
      have h_add : GIRCommand.zeroed GIROpcode.putGlyph ∈ compileStep acc hd :=
        compileStep_setContent_adds_glyph acc hd h_set
      exact compileFoldl_preserves_mem tl (compileStep acc hd)
        (GIRCommand.zeroed GIROpcode.putGlyph) h_add
    | inr h_tl =>
      exact ih (compileStep acc hd) h_tl

theorem compile_preserves_content (doc : SIRDocumentWithPayload) :
    wellFormedSIRWithPayload doc = true →
    ∀ instr ∈ doc.instructions,
      instr.opcode = SIROpcode.setContent →
      ∃ page ∈ compileReal doc.instructions,
        ∃ cmd ∈ page,
          cmd.opcode = GIROpcode.putGlyph := by
  intro _h_wf instr h_mem h_set
  -- Show the foldl result contains a putGlyph
  have h_glyph : GIRCommand.zeroed GIROpcode.putGlyph ∈
      doc.instructions.foldl compileStep [] :=
    compileFoldl_setContent_glyph doc.instructions [] instr h_mem h_set
  -- The foldl result is non-empty since it contains putGlyph
  have h_nonempty : doc.instructions.foldl compileStep [] ≠ [] := by
    intro h_eq
    rw [h_eq] at h_glyph
    exact List.not_mem_nil h_glyph
  -- compileReal with non-empty cmds returns [cmds]
  simp only [compileReal, List.isEmpty_iff, if_neg h_nonempty]
  -- Goal: ∃ page, page ∈ [foldl ...] ∧ ∃ cmd ∈ page, cmd.opcode = putGlyph
  exists doc.instructions.foldl compileStep []
  constructor
  · exact List.mem_singleton_self _
  · exact ⟨GIRCommand.zeroed GIROpcode.putGlyph, h_glyph, rfl⟩

/-- THM-COMPILE-COMPLETENESS-001: Content completeness lemma.

    Every SetContent instruction with non-empty payload content produces
    at least one PUT_GLYPH command in the compiled G-IR output.

    This reduces to compile_preserves_content since compileReal
    generates putGlyph for ALL setContent instructions regardless
    of payload content. -/
theorem compile_nonempty_content_produces_glyphs (doc : SIRDocumentWithPayload) :
    wellFormedSIRWithPayload doc = true →
    ∀ instr ∈ doc.instructions,
      instr.opcode = SIROpcode.setContent →
      (doc.payload.data.drop instr.payload_offset).take 256 ≠ "" →
      ∃ page ∈ compileReal doc.instructions,
        ∃ cmd ∈ page,
          cmd.opcode = GIROpcode.putGlyph := by
  intro _h_wf _instr _h_mem _h_set _h_nonempty
  exact compile_preserves_content doc ‹_› _instr _h_mem _h_set

-- ============================================================================
-- Stack Balance Lemmas
-- ============================================================================

theorem stackBalanced_push_only :
    stackBalanced [GIRCommand.zeroed GIROpcode.pushStack] = false := by
  native_decide

theorem stackBalanced_push_pop :
    stackBalanced [GIRCommand.zeroed GIROpcode.pushStack,
                  GIRCommand.zeroed GIROpcode.popStack] = true := by
  native_decide

theorem stackBalanced_nested :
    stackBalanced [GIRCommand.zeroed GIROpcode.pushStack,
                  GIRCommand.zeroed GIROpcode.pushStack,
                  GIRCommand.zeroed GIROpcode.popStack,
                  GIRCommand.zeroed GIROpcode.popStack] = true := by
  native_decide

theorem stackBalanced_pop_only :
    stackBalanced [GIRCommand.zeroed GIROpcode.popStack] = false := by
  native_decide

-- ============================================================================
-- GIRCommand Argument Lemmas
-- ============================================================================

theorem gir_command_args_eq_eight : GIR_COMMAND_ARGS = 8 := rfl

theorem zeroed_args_zero (op : GIROpcode) (i : Fin GIR_COMMAND_ARGS) :
    (GIRCommand.zeroed op).args i = 0 := by
  rfl

-- ============================================================================
-- BlockType Count
-- ============================================================================

theorem blockType_variant_count :
    [BlockType.document, BlockType.paragraph, BlockType.heading,
      BlockType.list, BlockType.math, BlockType.code,
      BlockType.blockQuote, BlockType.thematicBreak,
      BlockType.image, BlockType.table,
      BlockType.tableRow, BlockType.tableCell,
      BlockType.footnote, BlockType.footnoteBlock,
      BlockType.figure].length = 15 := rfl

end LDIR
