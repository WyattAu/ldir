use thiserror::Error;

#[derive(Debug, Error)]
pub enum IccProfileError {
    #[error("ICC profile data too small: {0} bytes")]
    TooSmall(usize),
    #[error("invalid ICC profile: {0}")]
    InvalidProfile(String),
    #[error("unknown ICC color space: {0}")]
    UnknownColorSpace(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IccColorSpace {
    Rgb,
    Cmyk,
    Gray,
    Lab,
}

pub struct IccProfile {
    pub data: Vec<u8>,
    pub name: String,
    pub color_space: IccColorSpace,
    pub components: u8,
}

impl IccProfile {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, IccProfileError> {
        if data.len() < 132 {
            return Err(IccProfileError::TooSmall(data.len()));
        }
        if &data[36..40] != b"acsp" {
            return Err(IccProfileError::InvalidProfile(
                "missing acsp signature".into(),
            ));
        }

        let color_space = match &data[16..20] {
            b"RGB " => IccColorSpace::Rgb,
            b"CMYK" => IccColorSpace::Cmyk,
            b"GRAY" => IccColorSpace::Gray,
            b"Lab " => IccColorSpace::Lab,
            other => return Err(IccProfileError::UnknownColorSpace(format!("{:?}", other))),
        };

        let components = match color_space {
            IccColorSpace::Rgb => 3,
            IccColorSpace::Cmyk => 4,
            IccColorSpace::Gray => 1,
            IccColorSpace::Lab => 3,
        };

        let name = extract_profile_name(&data).unwrap_or_else(|| "Unknown".into());

        Ok(Self {
            data,
            name,
            color_space,
            components,
        })
    }

    pub fn srgb() -> Self {
        Self {
            data: build_srgb_icc(),
            name: "sRGB".into(),
            color_space: IccColorSpace::Rgb,
            components: 3,
        }
    }

    pub fn cmyk() -> Self {
        Self {
            data: build_cmyk_icc(),
            name: "Generic CMYK".into(),
            color_space: IccColorSpace::Cmyk,
            components: 4,
        }
    }

    pub fn gray() -> Self {
        Self {
            data: build_gray_icc(),
            name: "Gray Gamma 2.2".into(),
            color_space: IccColorSpace::Gray,
            components: 1,
        }
    }
}

pub fn srgb_to_cmyk(r: u8, g: u8, b: u8) -> (u8, u8, u8, u8) {
    let rf = f32::from(r) / 255.0;
    let gf = f32::from(g) / 255.0;
    let bf = f32::from(b) / 255.0;

    let k = 1.0 - rf.max(gf).max(bf);
    if k >= 1.0 {
        return (0, 0, 0, 255);
    }
    let inv_k = 1.0 / (1.0 - k);
    let c = ((1.0 - rf - k) * inv_k * 255.0).round() as u8;
    let m = ((1.0 - gf - k) * inv_k * 255.0).round() as u8;
    let y = ((1.0 - bf - k) * inv_k * 255.0).round() as u8;
    let kv = (k * 255.0).round() as u8;
    (c, m, y, kv)
}

pub fn cmyk_to_srgb(c: u8, m: u8, y: u8, k: u8) -> (u8, u8, u8) {
    let cf = f32::from(c) / 255.0;
    let mf = f32::from(m) / 255.0;
    let yf = f32::from(y) / 255.0;
    let kf = f32::from(k) / 255.0;

    let r = (255.0 * (1.0 - cf) * (1.0 - kf)).round() as u8;
    let g = (255.0 * (1.0 - mf) * (1.0 - kf)).round() as u8;
    let b = (255.0 * (1.0 - yf) * (1.0 - kf)).round() as u8;
    (r, g, b)
}

fn extract_profile_name(data: &[u8]) -> Option<String> {
    if data.len() < 132 {
        return None;
    }
    let tag_count = u32::from_be_bytes([data[128], data[129], data[130], data[131]]) as usize;
    for i in 0..tag_count {
        let entry_offset = 132 + i * 12;
        if entry_offset + 12 > data.len() {
            break;
        }
        let sig = &data[entry_offset..entry_offset + 4];
        if sig == b"desc" {
            let data_offset = u32::from_be_bytes([
                data[entry_offset + 4],
                data[entry_offset + 5],
                data[entry_offset + 6],
                data[entry_offset + 7],
            ]) as usize;
            if data_offset + 12 > data.len() {
                return None;
            }
            let ascii_count = u32::from_be_bytes([
                data[data_offset + 8],
                data[data_offset + 9],
                data[data_offset + 10],
                data[data_offset + 11],
            ]) as usize;
            if ascii_count == 0 || data_offset + 12 + ascii_count > data.len() {
                return None;
            }
            let name_bytes = &data[data_offset + 12..data_offset + 12 + ascii_count - 1];
            return String::from_utf8(name_bytes.to_vec()).ok();
        }
    }
    None
}

const D50_X: u32 = 0x0000F6D6;
const D50_Y: u32 = 0x00010000;
const D50_Z: u32 = 0x0000D32D;

fn tag_sig(s: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*s)
}

fn build_desc_tag(description: &str) -> Vec<u8> {
    let mut ascii_with_null = description.as_bytes().to_vec();
    ascii_with_null.push(0);
    let mut padded = ascii_with_null.clone();
    while !padded.len().is_multiple_of(4) {
        padded.push(0);
    }

    let mut data = Vec::new();
    data.extend_from_slice(b"desc");
    data.extend_from_slice(&[0u8; 4]);
    data.extend_from_slice(&(ascii_with_null.len() as u32).to_be_bytes());
    data.extend_from_slice(&padded);
    data.extend_from_slice(&[0u8; 4]);
    data.extend_from_slice(&[0u8; 4]);
    data.extend_from_slice(&[0u8; 2]);
    data.extend_from_slice(&[0u8; 2]);
    data.extend_from_slice(&[0u8; 67]);
    data
}

fn build_cprt_tag(text: &str) -> Vec<u8> {
    let mut text_bytes = text.as_bytes().to_vec();
    text_bytes.push(0);
    while !text_bytes.len().is_multiple_of(4) {
        text_bytes.push(0);
    }

    let mut data = Vec::new();
    data.extend_from_slice(b"text");
    data.extend_from_slice(&[0u8; 4]);
    data.extend_from_slice(&text_bytes);
    data
}

fn build_xyz_type(x: u32, y: u32, z: u32) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"XYZ ");
    data.extend_from_slice(&[0u8; 4]);
    data.extend_from_slice(&x.to_be_bytes());
    data.extend_from_slice(&y.to_be_bytes());
    data.extend_from_slice(&z.to_be_bytes());
    data
}

fn build_wtpt_tag() -> Vec<u8> {
    build_xyz_type(D50_X, D50_Y, D50_Z)
}

fn build_trc_identity() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"curv");
    data.extend_from_slice(&[0u8; 4]);
    data.extend_from_slice(&0u32.to_be_bytes());
    data
}

fn assemble_icc(
    color_space: &[u8; 4],
    device_class: &[u8; 4],
    tags: &mut [(u32, Vec<u8>)],
) -> Vec<u8> {
    tags.sort_by_key(|(sig, _)| *sig);

    let tag_count = tags.len();
    let tag_table_size = 4 + tag_count * 12;
    let header_size = 128;
    let tag_data_start = header_size + tag_table_size;

    let mut offsets = Vec::with_capacity(tag_count);
    let mut offset = tag_data_start;
    for (_, data) in tags.iter() {
        offsets.push(offset);
        offset += data.len();
        while !offset.is_multiple_of(4) {
            offset += 1;
        }
    }

    let total_size = offset;
    let mut buf = vec![0u8; total_size];

    buf[0..4].copy_from_slice(&(total_size as u32).to_be_bytes());
    buf[8..12].copy_from_slice(&0x02100000u32.to_be_bytes());
    buf[12..16].copy_from_slice(device_class);
    buf[16..20].copy_from_slice(color_space);
    buf[20..24].copy_from_slice(b"XYZ ");
    buf[24..26].copy_from_slice(&2024u16.to_be_bytes());
    buf[26] = 1;
    buf[27] = 1;
    buf[36..40].copy_from_slice(b"acsp");
    buf[68..72].copy_from_slice(&D50_X.to_be_bytes());
    buf[72..76].copy_from_slice(&D50_Y.to_be_bytes());
    buf[76..80].copy_from_slice(&D50_Z.to_be_bytes());

    buf[128..132].copy_from_slice(&(tag_count as u32).to_be_bytes());
    for i in 0..tag_count {
        let entry_start = 132 + i * 12;
        buf[entry_start..entry_start + 4].copy_from_slice(&tags[i].0.to_be_bytes());
        buf[entry_start + 4..entry_start + 8].copy_from_slice(&(offsets[i] as u32).to_be_bytes());
        buf[entry_start + 8..entry_start + 12]
            .copy_from_slice(&(tags[i].1.len() as u32).to_be_bytes());
    }

    for i in 0..tag_count {
        let start = offsets[i];
        buf[start..start + tags[i].1.len()].copy_from_slice(&tags[i].1);
    }

    buf
}

fn build_srgb_icc() -> Vec<u8> {
    let trc = build_trc_identity();
    let desc = build_desc_tag("sRGB");
    let cprt = build_cprt_tag("CC0");
    let wtpt = build_wtpt_tag();
    let rxyz = build_xyz_type(0x00006FA2, 0x00003904, 0x00000392);
    let gxyz = build_xyz_type(0x00006299, 0x0000B785, 0x000018DA);
    let bxyz = build_xyz_type(0x000024A6, 0x00000F84, 0x0000B6CF);

    let mut tags: Vec<(u32, Vec<u8>)> = vec![
        (tag_sig(b"desc"), desc),
        (tag_sig(b"cprt"), cprt),
        (tag_sig(b"wtpt"), wtpt),
        (tag_sig(b"rXYZ"), rxyz),
        (tag_sig(b"gXYZ"), gxyz),
        (tag_sig(b"bXYZ"), bxyz),
        (tag_sig(b"rTRC"), trc.clone()),
        (tag_sig(b"gTRC"), trc.clone()),
        (tag_sig(b"bTRC"), trc),
    ];

    assemble_icc(b"RGB ", b"mntr", &mut tags)
}

fn build_gray_icc() -> Vec<u8> {
    let trc = build_trc_identity();
    let desc = build_desc_tag("Gray Gamma 2.2");
    let cprt = build_cprt_tag("CC0");
    let wtpt = build_wtpt_tag();

    let mut tags: Vec<(u32, Vec<u8>)> = vec![
        (tag_sig(b"desc"), desc),
        (tag_sig(b"cprt"), cprt),
        (tag_sig(b"wtpt"), wtpt),
        (tag_sig(b"kTRC"), trc),
    ];

    assemble_icc(b"GRAY", b"mntr", &mut tags)
}

fn build_lut8_tag(
    input_channels: u8,
    output_channels: u8,
    grid_points: u8,
    matrix: &[i32],
    input_tables: &[&[u8]],
    clut: &[u8],
    output_tables: &[&[u8]],
) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(b"mft2");
    data.extend_from_slice(&[0u8; 4]);
    data.push(input_channels);
    data.push(output_channels);
    data.push(grid_points);
    data.push(0);
    for &v in matrix {
        data.extend_from_slice(&v.to_be_bytes());
    }
    for table in input_tables {
        data.extend_from_slice(table);
    }
    data.extend_from_slice(clut);
    for table in output_tables {
        data.extend_from_slice(table);
    }
    data
}

fn build_cmyk_icc() -> Vec<u8> {
    let desc = build_desc_tag("Generic CMYK");
    let cprt = build_cprt_tag("CC0");
    let wtpt = build_wtpt_tag();

    let identity_matrix: [i32; 12] = [
        0x00010000, 0, 0, 0, 0, 0x00010000, 0, 0, 0, 0, 0x00010000, 0,
    ];

    let identity_table: [u8; 2] = [0, 255];

    let mut b2a0_clut = Vec::new();
    for k in 0..2u8 {
        for y in 0..2u8 {
            for m in 0..2u8 {
                for c in 0..2u8 {
                    let c_val = f32::from(c * 255) / 255.0;
                    let m_val = f32::from(m * 255) / 255.0;
                    let y_val = f32::from(y * 255) / 255.0;
                    let k_val = f32::from(k * 255) / 255.0;

                    let r = 255.0 * (1.0 - c_val) * (1.0 - k_val);
                    let g = 255.0 * (1.0 - m_val) * (1.0 - k_val);
                    let b = 255.0 * (1.0 - y_val) * (1.0 - k_val);

                    b2a0_clut.push(r.round() as u8);
                    b2a0_clut.push(g.round() as u8);
                    b2a0_clut.push(b.round() as u8);
                }
            }
        }
    }

    let b2a_input: Vec<&[u8]> = vec![&identity_table; 4];
    let b2a_output: Vec<&[u8]> = vec![&identity_table; 3];
    let b2a0 = build_lut8_tag(
        4,
        3,
        2,
        &identity_matrix,
        &b2a_input,
        &b2a0_clut,
        &b2a_output,
    );

    let mut a2b0_clut = Vec::new();
    for z in 0..2u8 {
        for y in 0..2u8 {
            for x in 0..2u8 {
                let x_val = f32::from(x * 255) / 255.0;
                let y_val = f32::from(y * 255) / 255.0;
                let z_val = f32::from(z * 255) / 255.0;

                let max_val = x_val.max(y_val).max(z_val);
                if max_val < 0.001 {
                    a2b0_clut.extend_from_slice(&[0, 0, 0, 255]);
                } else {
                    let k = (1.0 - max_val) * 255.0;
                    let inv_k = 255.0 / max_val;
                    let cv = ((max_val - x_val) * inv_k).round() as u8;
                    let mv = ((max_val - y_val) * inv_k).round() as u8;
                    let yv = ((max_val - z_val) * inv_k).round() as u8;
                    let kv = k.round() as u8;
                    a2b0_clut.extend_from_slice(&[cv, mv, yv, kv]);
                }
            }
        }
    }

    let a2b_input: Vec<&[u8]> = vec![&identity_table; 3];
    let a2b_output: Vec<&[u8]> = vec![&identity_table; 4];
    let a2b0 = build_lut8_tag(
        3,
        4,
        2,
        &identity_matrix,
        &a2b_input,
        &a2b0_clut,
        &a2b_output,
    );

    let mut tags: Vec<(u32, Vec<u8>)> = vec![
        (tag_sig(b"A2B0"), a2b0),
        (tag_sig(b"B2A0"), b2a0),
        (tag_sig(b"cprt"), cprt),
        (tag_sig(b"desc"), desc),
        (tag_sig(b"wtpt"), wtpt),
    ];

    assemble_icc(b"CMYK", b"prtr", &mut tags)
}

pub(crate) fn icc_alternate_name(cs: IccColorSpace) -> &'static [u8] {
    match cs {
        IccColorSpace::Rgb => b"DeviceRGB",
        IccColorSpace::Cmyk => b"DeviceCMYK",
        IccColorSpace::Gray => b"DeviceGray",
        IccColorSpace::Lab => b"Lab",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_profile_valid() {
        let profile = IccProfile::srgb();
        assert_eq!(profile.color_space, IccColorSpace::Rgb);
        assert_eq!(profile.components, 3);
        assert_eq!(profile.name, "sRGB");
        assert!(profile.data.len() >= 132);
        assert_eq!(&profile.data[36..40], b"acsp");
        assert_eq!(&profile.data[16..20], b"RGB ");
    }

    #[test]
    fn test_gray_profile_valid() {
        let profile = IccProfile::gray();
        assert_eq!(profile.color_space, IccColorSpace::Gray);
        assert_eq!(profile.components, 1);
        assert_eq!(profile.name, "Gray Gamma 2.2");
        assert_eq!(&profile.data[16..20], b"GRAY");
    }

    #[test]
    fn test_cmyk_profile_valid() {
        let profile = IccProfile::cmyk();
        assert_eq!(profile.color_space, IccColorSpace::Cmyk);
        assert_eq!(profile.components, 4);
        assert_eq!(profile.name, "Generic CMYK");
        assert_eq!(&profile.data[16..20], b"CMYK");
    }

    #[test]
    fn test_profile_from_bytes_srgb() {
        let original = IccProfile::srgb();
        let parsed = IccProfile::from_bytes(original.data.clone()).unwrap();
        assert_eq!(parsed.color_space, IccColorSpace::Rgb);
        assert_eq!(parsed.components, 3);
        assert_eq!(parsed.name, "sRGB");
    }

    #[test]
    fn test_profile_from_bytes_gray() {
        let original = IccProfile::gray();
        let parsed = IccProfile::from_bytes(original.data.clone()).unwrap();
        assert_eq!(parsed.color_space, IccColorSpace::Gray);
        assert_eq!(parsed.components, 1);
    }

    #[test]
    fn test_profile_from_bytes_cmyk() {
        let original = IccProfile::cmyk();
        let parsed = IccProfile::from_bytes(original.data.clone()).unwrap();
        assert_eq!(parsed.color_space, IccColorSpace::Cmyk);
        assert_eq!(parsed.components, 4);
    }

    #[test]
    fn test_profile_from_bytes_too_small() {
        let result = IccProfile::from_bytes(vec![0; 100]);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_from_bytes_invalid_signature() {
        let mut data = vec![0u8; 200];
        data[36..40].copy_from_slice(b"xxxx");
        let result = IccProfile::from_bytes(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_from_bytes_unknown_color_space() {
        let mut data = IccProfile::srgb().data;
        data[16..20].copy_from_slice(b"XXXX");
        let result = IccProfile::from_bytes(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_srgb_to_cmyk_black() {
        let (c, m, y, k) = srgb_to_cmyk(0, 0, 0);
        assert_eq!((c, m, y, k), (0, 0, 0, 255));
    }

    #[test]
    fn test_srgb_to_cmyk_white() {
        let (c, m, y, k) = srgb_to_cmyk(255, 255, 255);
        assert_eq!((c, m, y, k), (0, 0, 0, 0));
    }

    #[test]
    fn test_srgb_to_cmyk_red() {
        let (c, m, y, k) = srgb_to_cmyk(255, 0, 0);
        assert_eq!(k, 0);
        assert_eq!(c, 0);
        assert!(m > 0);
        assert!(y > 0);
    }

    #[test]
    fn test_srgb_to_cmyk_pure_colors() {
        assert_eq!(srgb_to_cmyk(255, 0, 0).0, 0);
        assert_eq!(srgb_to_cmyk(0, 255, 0).1, 0);
        assert_eq!(srgb_to_cmyk(0, 0, 255).2, 0);
    }

    #[test]
    fn test_cmyk_to_srgb_black() {
        let (r, g, b) = cmyk_to_srgb(0, 0, 0, 255);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn test_cmyk_to_srgb_white() {
        let (r, g, b) = cmyk_to_srgb(0, 0, 0, 0);
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn test_cmyk_to_srgb_cyan() {
        let (r, g, b) = cmyk_to_srgb(255, 0, 0, 0);
        assert_eq!(r, 0);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
    }

    #[test]
    fn test_cmyk_to_srgb_roundtrip() {
        let (c, m, y, k) = srgb_to_cmyk(128, 64, 200);
        let (r, g, b) = cmyk_to_srgb(c, m, y, k);
        assert!((i32::from(r) - 128).abs() < 5);
        assert!((i32::from(g) - 64).abs() < 5);
        assert!((i32::from(b) - 200).abs() < 5);
    }

    #[test]
    fn test_extract_profile_name_srgb() {
        let profile = IccProfile::srgb();
        let name = extract_profile_name(&profile.data);
        assert_eq!(name.as_deref(), Some("sRGB"));
    }

    #[test]
    fn test_extract_profile_name_gray() {
        let profile = IccProfile::gray();
        let name = extract_profile_name(&profile.data);
        assert_eq!(name.as_deref(), Some("Gray Gamma 2.2"));
    }

    #[test]
    fn test_extract_profile_name_cmyk() {
        let profile = IccProfile::cmyk();
        let name = extract_profile_name(&profile.data);
        assert_eq!(name.as_deref(), Some("Generic CMYK"));
    }

    #[test]
    fn test_profile_size_matches_header() {
        let profiles = [IccProfile::srgb(), IccProfile::gray(), IccProfile::cmyk()];
        for profile in &profiles {
            let declared_size = u32::from_be_bytes([
                profile.data[0],
                profile.data[1],
                profile.data[2],
                profile.data[3],
            ]) as usize;
            assert_eq!(
                profile.data.len(),
                declared_size,
                "Size mismatch for {:?}",
                profile.color_space
            );
        }
    }

    #[test]
    fn test_icc_alternate_name() {
        assert_eq!(icc_alternate_name(IccColorSpace::Rgb), b"DeviceRGB");
        assert_eq!(icc_alternate_name(IccColorSpace::Cmyk), b"DeviceCMYK");
        assert_eq!(icc_alternate_name(IccColorSpace::Gray), b"DeviceGray");
        assert_eq!(icc_alternate_name(IccColorSpace::Lab), b"Lab");
    }
}
