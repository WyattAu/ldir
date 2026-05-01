//! Payload region types for variable-length S-IR data (REQ-3.1.4).
//!
//! Variable-length payloads referenced by `SIRInstruction.payload_offset`
//! contain inline data (text blobs, style parameters, math expressions)
//! stored contiguously after the instruction header region.
//!
//! Per REQ-3.1.4: "Variable-length payloads referenced by PayloadOffset
//! shall contain inline data stored contiguously after the instruction
//! header region."

/// A contiguous region of variable-length payload data.
///
/// Payloads are referenced by `SIRInstruction::payload_offset` and contain
/// the inline data associated with each instruction (text blobs, style
/// parameters, math expressions).
///
/// # Well-Formedness (AX-004)
///
/// Every `payload_offset` in the document must satisfy:
/// `payload_offset + payload_length <= payload_region.len()`
///
/// # Examples
///
/// ```
/// use ldir_ir::sir::PayloadRegion;
///
/// let region = PayloadRegion::from_bytes(b"Hello, world!".to_vec());
/// assert_eq!(region.len(), 13);
/// assert_eq!(region.as_str(), Ok("Hello, world!"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PayloadRegion {
    /// Contiguous payload data bytes.
    data: Vec<u8>,
}

impl PayloadRegion {
    /// Create a new empty payload region.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a payload region from raw bytes.
    #[inline]
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Create a payload region with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Total length of the payload region in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the payload region is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get a reference to the raw payload bytes.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get a mutable reference to the raw payload bytes.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Extract a subslice of the payload at the given offset and length.
    ///
    /// Returns `None` if the requested range is out of bounds.
    /// This corresponds to a single instruction's payload reference.
    ///
    /// # Arguments
    ///
    /// * `offset` - Start offset into the payload region.
    /// * `length` - Number of bytes to extract.
    #[inline]
    pub fn get(&self, offset: u32, length: u32) -> Option<&[u8]> {
        let start = offset as usize;
        let end = start.checked_add(length as usize)?;
        self.data.get(start..end)
    }

    /// Extract a subslice using only an offset, returning everything from
    /// `offset` to the next NUL byte or end of region.
    ///
    /// Returns `None` if `offset` is out of bounds.
    #[inline]
    pub fn get_until_nul(&self, offset: u32) -> Option<&[u8]> {
        let start = offset as usize;
        if start > self.data.len() {
            return None;
        }
        let remaining = &self.data[start..];
        Some(match remaining.iter().position(|&b| b == 0) {
            Some(nul_pos) => &remaining[..nul_pos],
            None => remaining,
        })
    }

    /// Interpret a payload slice as a UTF-8 string.
    ///
    /// Returns `Err(std::str::Utf8Error)` if the bytes are not valid UTF-8.
    #[inline]
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    /// Append bytes to the payload region, returning the offset where
    /// the appended data starts.
    pub fn append(&mut self, bytes: &[u8]) -> u32 {
        let offset = self.data.len() as u32;
        self.data.extend_from_slice(bytes);
        offset
    }

    /// Clear the payload region.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Reserve capacity for additional bytes.
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }
}

impl Default for PayloadRegion {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for PayloadRegion {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for PayloadRegion {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_region() {
        let region = PayloadRegion::new();
        assert!(region.is_empty());
        assert_eq!(region.len(), 0);
    }

    #[test]
    fn test_from_bytes() {
        let region = PayloadRegion::from_bytes(vec![1, 2, 3, 4]);
        assert_eq!(region.len(), 4);
        assert_eq!(region.as_bytes(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_get_slice() {
        let region = PayloadRegion::from_bytes(vec![10, 20, 30, 40, 50]);
        assert_eq!(region.get(1, 3), Some(&[20, 30, 40][..]));
        assert_eq!(region.get(0, 0), Some(&[][..]));
        assert_eq!(region.get(4, 2), None);
        assert_eq!(region.get(5, 1), None);
    }

    #[test]
    fn test_get_until_nul() {
        let data = b"hello\0world".to_vec();
        let region = PayloadRegion::from_bytes(data.clone());
        assert_eq!(region.get_until_nul(0), Some(b"hello".as_slice()));
        assert_eq!(region.get_until_nul(6), Some(b"world".as_slice()));
        assert_eq!(region.get_until_nul(20), None);
    }

    #[test]
    fn test_as_str() {
        let region = PayloadRegion::from_bytes(b"Hello, world!".to_vec());
        assert_eq!(region.as_str(), Ok("Hello, world!"));
    }

    #[test]
    fn test_as_str_invalid_utf8() {
        let region = PayloadRegion::from_bytes(vec![0xFF, 0xFE]);
        assert!(region.as_str().is_err());
    }

    #[test]
    fn test_append() {
        let mut region = PayloadRegion::new();
        let off1 = region.append(b"hello");
        let off2 = region.append(b" ");
        let off3 = region.append(b"world");
        assert_eq!(off1, 0);
        assert_eq!(off2, 5);
        assert_eq!(off3, 6);
        assert_eq!(region.as_str(), Ok("hello world"));
    }

    #[test]
    fn test_with_capacity() {
        let region = PayloadRegion::with_capacity(1024);
        assert!(region.is_empty());
        assert!(region.data.capacity() >= 1024);
    }

    #[test]
    fn test_clear() {
        let mut region = PayloadRegion::from_bytes(vec![1, 2, 3]);
        region.clear();
        assert!(region.is_empty());
    }
}
