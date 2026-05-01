//! Error types for LDIR compilation pipeline.
//!
//! Structured error hierarchy per BP-IR-COMPILER-001.
//! Each error carries EntityID and byte offset per REQ-3.3.4.

use std::fmt;

/// Entity identifier type (u32 per REQ-3.1.6).
pub type EntityId = u32;

/// Byte offset in the input stream.
pub type ByteOffset = u32;

/// Parse error codes (ERR-PARSE-001..003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// ERR-PARSE-001: Input too short (< 13 bytes minimum instruction size).
    InputTooShort {
        /// Actual byte length of the input.
        len: usize,
    },
    /// ERR-PARSE-002: Input not 4-byte aligned.
    AlignmentError {
        /// Byte offset of the alignment violation.
        offset: ByteOffset,
    },
    /// ERR-PARSE-003: Invalid opcode byte.
    InvalidOpcode {
        /// The invalid byte value encountered.
        byte: u8,
        /// Byte offset where the invalid opcode was found.
        offset: ByteOffset,
    },
    /// rkyv deserialization failure.
    DeserializationError {
        /// Human-readable error message from the deserializer.
        message: String,
    },
}

/// Validation error codes (ERR-VALID-001..007).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorKind {
    /// ERR-VALID-001: Duplicate entity ID.
    DuplicateEntityId {
        /// The entity ID that was duplicated.
        entity_id: EntityId,
    },
    /// ERR-VALID-002: Parent reference does not exist.
    ParentNotFound {
        /// The entity whose parent is missing.
        entity_id: EntityId,
        /// The parent ID that was not found.
        parent_id: EntityId,
    },
    /// ERR-VALID-003: Circular parent chain detected.
    CircularParentChain {
        /// The entity at which the cycle was detected.
        entity_id: EntityId,
    },
    /// ERR-VALID-004: Payload region out of bounds.
    PayloadOutOfBounds {
        /// The entity with the invalid payload offset.
        entity_id: EntityId,
        /// The byte offset that is out of bounds.
        offset: ByteOffset,
    },
    /// ERR-VALID-005: Multiple root nodes.
    MultipleRoots {
        /// The number of root nodes found.
        count: usize,
    },
    /// ERR-VALID-006: No root node found.
    NoRoot,
    /// ERR-VALID-007: Invalid block nesting.
    InvalidBlockNesting {
        /// The entity with invalid block nesting.
        entity_id: EntityId,
    },
}

/// Compile error codes (ERR-COMP-001..002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileErrorKind {
    /// ERR-COMP-001: Stack overflow in compilation.
    StackOverflow {
        /// The stack depth at which overflow was detected.
        depth: usize,
    },
    /// ERR-COMP-002: Unsupported instruction in context.
    UnsupportedInstruction {
        /// The entity with the unsupported instruction.
        entity_id: EntityId,
    },
}

/// Emit error code (ERR-EMIT-001).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmitErrorKind {
    /// ERR-EMIT-001: G-IR command too large for buffer.
    BufferOverflow {
        /// The number of bytes required.
        required: usize,
        /// The number of bytes available.
        available: usize,
    },
}

/// A structured error with location information.
#[derive(Debug, Clone)]
pub struct LdirError {
    /// The specific error kind.
    pub kind: ErrorKind,
    /// Optional entity ID associated with this error.
    pub entity_id: Option<EntityId>,
    /// Optional byte offset in the input stream.
    pub byte_offset: Option<ByteOffset>,
}

/// Top-level error category for LDIR operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// A parse-phase error (IF-PARSE-001).
    Parse(ParseErrorKind),
    /// A validation-phase error (IF-VALIDATE-001).
    Validation(ValidationErrorKind),
    /// A compile-phase error (IF-COMPILE-001).
    Compile(CompileErrorKind),
    /// An emit-phase error (IF-EMIT-001).
    Emit(EmitErrorKind),
}

impl fmt::Display for LdirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let loc = match (self.entity_id, self.byte_offset) {
            (Some(eid), Some(off)) => format!(" [entity={eid}, offset={off}]"),
            (Some(eid), None) => format!(" [entity={eid}]"),
            (None, Some(off)) => format!(" [offset={off}]"),
            (None, None) => String::new(),
        };
        match &self.kind {
            ErrorKind::Parse(e) => write!(f, "Parse error{loc}: {e:?}"),
            ErrorKind::Validation(e) => write!(f, "Validation error{loc}: {e:?}"),
            ErrorKind::Compile(e) => write!(f, "Compile error{loc}: {e:?}"),
            ErrorKind::Emit(e) => write!(f, "Emit error{loc}: {e:?}"),
        }
    }
}

impl std::error::Error for LdirError {}

impl From<ParseErrorKind> for LdirError {
    fn from(kind: ParseErrorKind) -> Self {
        Self {
            kind: ErrorKind::Parse(kind),
            entity_id: None,
            byte_offset: None,
        }
    }
}

impl From<ValidationErrorKind> for LdirError {
    fn from(kind: ValidationErrorKind) -> Self {
        Self {
            kind: ErrorKind::Validation(kind),
            entity_id: None,
            byte_offset: None,
        }
    }
}

impl From<CompileErrorKind> for LdirError {
    fn from(kind: CompileErrorKind) -> Self {
        Self {
            kind: ErrorKind::Compile(kind),
            entity_id: None,
            byte_offset: None,
        }
    }
}

impl From<EmitErrorKind> for LdirError {
    fn from(kind: EmitErrorKind) -> Self {
        Self {
            kind: ErrorKind::Emit(kind),
            entity_id: None,
            byte_offset: None,
        }
    }
}

impl LdirError {
    /// Attach entity ID to this error.
    pub fn with_entity(mut self, id: EntityId) -> Self {
        self.entity_id = Some(id);
        self
    }

    /// Attach byte offset to this error.
    pub fn with_offset(mut self, offset: ByteOffset) -> Self {
        self.byte_offset = Some(offset);
        self
    }
}

/// Result type alias for LDIR operations.
pub type Result<T> = std::result::Result<T, LdirError>;

/// Validation result: Ok(()) or a list of validation errors.
pub type ValidationResult = std::result::Result<(), Vec<LdirError>>;
