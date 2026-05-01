//! Binary serialization for S-IR v2.
//!
//! Format:
//! ```text
//! [magic: 4 bytes "LDIR"]
//! [version: 3 bytes major.minor.patch]
//! [ir_version: 2 bytes]
//! [created: 8 bytes unix timestamp]
//! [source_format_len: 2 bytes][source_format: UTF-8]
//! [source_path_len: 2 bytes][source_path: UTF-8]
//! --- metadata section ---
//! [metadata_len: 4 bytes][metadata JSON bytes]
//! --- resources section ---
//! [resources_len: 4 bytes][resources JSON bytes]
//! --- styles section ---
//! [styles_len: 4 bytes][styles JSON bytes]
//! --- annotations section ---
//! [annotations_len: 4 bytes][annotations JSON bytes]
//! --- body section ---
//! [body_len: 4 bytes][body JSON bytes]
//! ```

use super::module::SIRModuleV2;

/// Serialize a module to binary bytes.
pub fn serialize_module(module: &SIRModuleV2) -> Vec<u8> {
    let mut buf = Vec::new();
    
    // Header
    buf.extend_from_slice(&module.header.magic);
    buf.push(module.header.version.0);
    buf.push(module.header.version.1);
    buf.push(module.header.version.2);
    buf.extend_from_slice(&module.header.ir_version.to_le_bytes());
    buf.extend_from_slice(&module.header.created.to_le_bytes());
    
    // Source info
    let sf = module.header.source_format.as_deref().unwrap_or("");
    let sp = module.header.source_path.as_deref().unwrap_or("");
    buf.extend_from_slice(&(sf.len() as u16).to_le_bytes());
    buf.extend_from_slice(sf.as_bytes());
    buf.extend_from_slice(&(sp.len() as u16).to_le_bytes());
    buf.extend_from_slice(sp.as_bytes());
    
    // Sections (5 sections, each length-prefixed JSON)
    let metadata_json = serde_json::to_vec(&module.metadata).unwrap_or_default();
    let resources_json = serde_json::to_vec(&module.resources).unwrap_or_default();
    let styles_json = serde_json::to_vec(&module.styles).unwrap_or_default();
    let annotations_json = serde_json::to_vec(&module.annotations).unwrap_or_default();
    let body_json = serde_json::to_vec(&module.body).unwrap_or_default();
    
    for section in [&metadata_json, &resources_json, &styles_json, &annotations_json, &body_json] {
        buf.extend_from_slice(&(section.len() as u32).to_le_bytes());
        buf.extend_from_slice(section);
    }
    
    buf
}

/// Deserialize a module from binary bytes.
pub fn deserialize_module(bytes: &[u8]) -> Result<SIRModuleV2, String> {
    if bytes.len() < 17 { return Err("buffer too short".into()); }
    
    // Magic
    if &bytes[0..4] != b"LDIR" { return Err("invalid magic".into()); }
    
    // Version
    let major = bytes[4];
    let minor = bytes[5];
    let patch = bytes[6];
    if major != 2 { return Err(format!("unsupported version {}.{}.{}", major, minor, patch)); }
    
    // IR version
    let ir_version = u16::from_le_bytes([bytes[7], bytes[8]]);
    
    // Created
    let created = u64::from_le_bytes(bytes[9..17].try_into().map_err(|e: core::array::TryFromSliceError| e.to_string())?);
    
    let mut pos = 17;
    
    // Source format
    let sf_len = u16::from_le_bytes(bytes[pos..pos+2].try_into().map_err(|e: core::array::TryFromSliceError| e.to_string())?) as usize;
    pos += 2;
    let source_format = if sf_len > 0 {
        let s = std::str::from_utf8(&bytes[pos..pos+sf_len]).map_err(|e| e.to_string())?;
        pos += sf_len;
        Some(s.to_string())
    } else { None };
    
    // Source path
    let sp_len = u16::from_le_bytes(bytes[pos..pos+2].try_into().map_err(|e: core::array::TryFromSliceError| e.to_string())?) as usize;
    pos += 2;
    let source_path = if sp_len > 0 {
        let s = std::str::from_utf8(&bytes[pos..pos+sp_len]).map_err(|e| e.to_string())?;
        pos += sp_len;
        Some(s.to_string())
    } else { None };
    
    // Sections
    let mut sections = Vec::new();
    for _ in 0..5 {
        if pos + 4 > bytes.len() { return Err("unexpected end of sections".into()); }
        let len = u32::from_le_bytes(bytes[pos..pos+4].try_into().map_err(|e: core::array::TryFromSliceError| e.to_string())?) as usize;
        pos += 4;
        if pos + len > bytes.len() { return Err("section data truncated".into()); }
        sections.push(&bytes[pos..pos+len]);
        pos += len;
    }
    
    let metadata: crate::sir::v2::metadata::DocumentMetadata = 
        serde_json::from_slice(sections[0]).unwrap_or_default();
    let resources: crate::sir::v2::resources::ResourceDecls = 
        serde_json::from_slice(sections[1]).unwrap_or_default();
    let styles: crate::sir::v2::styles::StyleDecls = 
        serde_json::from_slice(sections[2]).unwrap_or_default();
    let annotations: crate::sir::v2::annotations::Annotations = 
        serde_json::from_slice(sections[3]).unwrap_or_default();
    let body: crate::sir::v2::nodes::NodeTree = 
        serde_json::from_slice(sections[4]).unwrap_or_default();
    
    Ok(SIRModuleV2 {
        header: super::module::ModuleHeader {
            magic: *super::module::SIR_V2_MAGIC,
            version: (major, minor, patch),
            ir_version,
            source_format,
            source_path,
            created,
        },
        metadata,
        resources,
        styles,
        annotations,
        body,
    })
}

/// Binary writer helper.
pub struct SIRBinaryWriter;

impl SIRBinaryWriter {
    pub fn write(module: &SIRModuleV2) -> Vec<u8> {
        serialize_module(module)
    }
    
    pub fn read(bytes: &[u8]) -> Result<SIRModuleV2, String> {
        deserialize_module(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_roundtrip() {
        let mut m = SIRModuleV2::new();
        m.metadata.title = Some("Test Document".to_string());
        m.metadata.author = Some("Test Author".to_string());

        let bytes = SIRBinaryWriter::write(&m);
        let restored = SIRBinaryWriter::read(&bytes).unwrap();
        assert_eq!(restored.metadata.title.as_deref(), Some("Test Document"));
        assert_eq!(restored.metadata.author.as_deref(), Some("Test Author"));
    }

    #[test]
    fn test_binary_magic() {
        let mut m = SIRModuleV2::new();
        let bytes = SIRBinaryWriter::write(&m);
        assert_eq!(&bytes[0..4], b"LDIR");
    }

    #[test]
    fn test_binary_version_check() {
        let bytes = SIRBinaryWriter::write(&SIRModuleV2::new());
        let mut bad_bytes = bytes.clone();
        bad_bytes[4] = 3; // change major version
        assert!(SIRBinaryWriter::read(&bad_bytes).is_err());
    }

    #[test]
    fn test_binary_empty_module() {
        let m = SIRModuleV2::new();
        let bytes = SIRBinaryWriter::write(&m);
        assert!(bytes.len() >= 17);
        let restored = SIRBinaryWriter::read(&bytes).unwrap();
        assert!(restored.metadata.title.is_none());
        assert!(restored.body.is_empty());
    }
}
