//! G-IR Binary Emitter and Parser (IF-EMIT-001).
//!
//! Serializes a `GIRDocument` to a binary wire format and parses it back.
//!
//! ## Binary Format
//!
//! ```text
//! Header:
//!   Magic:   "GIR0"           (4 bytes)
//!   Pages:   u32 LE           (4 bytes)
//!
//! Per page:
//!   Cmds:    u32 LE           (4 bytes)
//!   Width:   i32 LE (fp26_6)  (4 bytes)
//!   Height:  i32 LE (fp26_6)  (4 bytes)
//!
//! Per command:
//!   Opcode:  u8               (1 byte)
//!   Padding: 3 bytes          (3 bytes)
//!   Args:    8 × i32 LE       (32 bytes)
//!   Total:                     36 bytes
//! ```

pub mod binary;

pub use binary::{emit_gir, parse_gir};
