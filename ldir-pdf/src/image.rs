#![deny(unsafe_code)]

use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    RGB,
    Gray,
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub color_space: ColorSpace,
    pub bits_per_component: u8,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    UnsupportedFormat,
    PngDecode(String),
    JpegDecode(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::UnsupportedFormat => write!(f, "unsupported image format"),
            Error::PngDecode(msg) => write!(f, "PNG decode error: {msg}"),
            Error::JpegDecode(msg) => write!(f, "JPEG decode error: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

#[must_use = "loading an image can fail; check the result"]
pub fn load_image(path: &Path) -> Result<ImageData, Error> {
    let data = std::fs::read(path)?;
    let format = detect_format(&data).ok_or(Error::UnsupportedFormat)?;
    decode_image(&data, format)
}

/// Magic-byte format detection, zero dependencies.
///
/// Checks mirror `media_kit::sniff` for the formats this backend supports:
/// PNG via the full 8-byte signature, JPEG via `FF D8 FF`.
pub fn detect_format(data: &[u8]) -> Option<ImageFormat> {
    if data.len() >= 8 && data[..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        Some(ImageFormat::Png)
    } else if data.len() >= 3 && data[..3] == [0xFF, 0xD8, 0xFF] {
        Some(ImageFormat::Jpeg)
    } else {
        None
    }
}

#[must_use = "decoding an image can fail; check the result"]
pub fn decode_image(data: &[u8], format: ImageFormat) -> Result<ImageData, Error> {
    match format {
        ImageFormat::Png => decode_png(data),
        ImageFormat::Jpeg => decode_jpeg(data),
    }
}

#[must_use = "decoding PNG can fail; check the result"]
pub fn decode_png(data: &[u8]) -> Result<ImageData, Error> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(data));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|e| Error::PngDecode(e.to_string()))?;

    let buf_size = reader
        .output_buffer_size()
        .ok_or_else(|| Error::PngDecode("output_buffer_size unknown after read_info".into()))?;
    let mut buf = vec![0; buf_size];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| Error::PngDecode(e.to_string()))?;
    buf.truncate(info.buffer_size());

    let width = info.width;
    let height = info.height;
    let color_type = info.color_type;

    let (color_space, pixel_data) = match color_type {
        png::ColorType::Grayscale => (ColorSpace::Gray, buf),
        png::ColorType::Rgb => (ColorSpace::RGB, buf),
        png::ColorType::GrayscaleAlpha => {
            let mut gray = Vec::with_capacity(width as usize * height as usize);
            for chunk in buf.chunks_exact(2) {
                gray.push(chunk[0]);
            }
            (ColorSpace::Gray, gray)
        }
        png::ColorType::Rgba => {
            let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
            for chunk in buf.chunks_exact(4) {
                rgb.extend_from_slice(&chunk[..3]);
            }
            (ColorSpace::RGB, rgb)
        }
        png::ColorType::Indexed => (ColorSpace::RGB, buf),
    };

    Ok(ImageData {
        width,
        height,
        color_space,
        bits_per_component: 8,
        data: pixel_data,
    })
}

#[must_use = "decoding JPEG can fail; check the result"]
pub fn decode_jpeg(data: &[u8]) -> Result<ImageData, Error> {
    let mut decoder = jpeg_decoder::Decoder::new(data);
    let pixels = decoder
        .decode()
        .map_err(|e| Error::JpegDecode(e.to_string()))?;
    let info = decoder
        .info()
        .ok_or_else(|| Error::JpegDecode("no image info after decode".to_string()))?;

    let width = info.width as u32;
    let height = info.height as u32;

    let (color_space, pixel_data) = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => (ColorSpace::RGB, pixels),
        jpeg_decoder::PixelFormat::L8 => (ColorSpace::Gray, pixels),
        jpeg_decoder::PixelFormat::CMYK32 => {
            let mut rgb = Vec::with_capacity(width as usize * height as usize * 3);
            for chunk in pixels.chunks_exact(4) {
                let c = f32::from(chunk[0]) / 255.0;
                let m = f32::from(chunk[1]) / 255.0;
                let y = f32::from(chunk[2]) / 255.0;
                let k = f32::from(chunk[3]) / 255.0;
                let r = (255.0 * (1.0 - c) * (1.0 - k)) as u8;
                let g = (255.0 * (1.0 - m) * (1.0 - k)) as u8;
                let b = (255.0 * (1.0 - y) * (1.0 - k)) as u8;
                rgb.extend_from_slice(&[r, g, b]);
            }
            (ColorSpace::RGB, rgb)
        }
        _ => return Err(Error::UnsupportedFormat),
    };

    Ok(ImageData {
        width,
        height,
        color_space,
        bits_per_component: 8,
        data: pixel_data,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

pub fn detect_png_dimensions(data: &[u8]) -> Option<ImageDimensions> {
    if data.len() < 24 {
        return None;
    }
    if data.get(0..4) != Some(&[0x89, 0x50, 0x4E, 0x47][..]) {
        return None;
    }
    if &data[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Some(ImageDimensions { width, height })
}

pub fn detect_jpeg_dimensions(data: &[u8]) -> Option<ImageDimensions> {
    if data.len() < 4 {
        return None;
    }
    if data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut pos = 2usize;
    while pos + 1 < data.len() {
        if data[pos] != 0xFF {
            return None;
        }
        let marker = data[pos + 1];
        pos += 2;
        if marker == 0xC0 || marker == 0xC2 {
            if pos + 7 > data.len() {
                return None;
            }
            let height = u16::from_be_bytes([data[pos + 3], data[pos + 4]]) as u32;
            let width = u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32;
            return Some(ImageDimensions { width, height });
        }
        if marker == 0xD8 || marker == 0xD9 {
            continue;
        }
        if pos + 2 > data.len() {
            return None;
        }
        let seg_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        if seg_len < 2 {
            return None;
        }
        pos += seg_len;
    }
    None
}

pub fn scale_to_fit(width: u32, height: u32, max_width_pt: f64, max_height_pt: f64) -> (f64, f64) {
    let w = width as f64;
    let h = height as f64;
    if w <= 0.0 || h <= 0.0 {
        return (0.0, 0.0);
    }
    let scale_w = if w > max_width_pt {
        max_width_pt / w
    } else {
        1.0
    };
    let scaled_w = w * scale_w;
    let scaled_h = h * scale_w;
    if scaled_h > max_height_pt && max_height_pt > 0.0 {
        let scale_h = max_height_pt / scaled_h;
        (scaled_w * scale_h, max_height_pt)
    } else {
        (scaled_w, scaled_h)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_png(width: u32, height: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, width, height);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("header");
            let pixel_data: Vec<u8> = (0..width * height)
                .flat_map(|i| {
                    let r = ((i * 7) % 256) as u8;
                    let g = ((i * 13) % 256) as u8;
                    let b = ((i * 23) % 256) as u8;
                    [r, g, b]
                })
                .collect();
            writer.write_image_data(&pixel_data).expect("write data");
        }
        buf
    }

    fn make_test_grayscale_png(width: u32, height: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, width, height);
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("header");
            let pixel_data: Vec<u8> = (0..width * height).map(|i| ((i * 7) % 256) as u8).collect();
            writer.write_image_data(&pixel_data).expect("write data");
        }
        buf
    }

    fn make_test_rgba_png(width: u32, height: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, width, height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("header");
            let pixel_data: Vec<u8> = (0..width * height)
                .flat_map(|i| {
                    let r = ((i * 7) % 256) as u8;
                    let g = ((i * 13) % 256) as u8;
                    let b = ((i * 23) % 256) as u8;
                    let a = 255u8;
                    [r, g, b, a]
                })
                .collect();
            writer.write_image_data(&pixel_data).expect("write data");
        }
        buf
    }

    #[test]
    fn test_decode_rgb_png() {
        let data = make_test_png(4, 3);
        let img = decode_png(&data).expect("decode RGB PNG");

        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        assert_eq!(img.color_space, ColorSpace::RGB);
        assert_eq!(img.bits_per_component, 8);
        assert_eq!(img.data.len(), 4 * 3 * 3);
    }

    #[test]
    fn test_decode_grayscale_png() {
        let data = make_test_grayscale_png(4, 3);
        let img = decode_png(&data).expect("decode grayscale PNG");

        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        assert_eq!(img.color_space, ColorSpace::Gray);
        assert_eq!(img.bits_per_component, 8);
        assert_eq!(img.data.len(), 4 * 3);
    }

    #[test]
    fn test_decode_rgba_png_strips_alpha() {
        let data = make_test_rgba_png(4, 3);
        let img = decode_png(&data).expect("decode RGBA PNG");

        assert_eq!(img.width, 4);
        assert_eq!(img.height, 3);
        assert_eq!(img.color_space, ColorSpace::RGB);
        assert_eq!(img.bits_per_component, 8);
        assert_eq!(img.data.len(), 4 * 3 * 3);
    }

    #[test]
    fn test_decode_png_1x1() {
        let data = make_test_png(1, 1);
        let img = decode_png(&data).expect("decode 1x1 PNG");

        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
        assert_eq!(img.data.len(), 3);
    }

    #[test]
    fn test_detect_format_png() {
        let data = make_test_png(2, 2);
        assert_eq!(detect_format(&data), Some(ImageFormat::Png));
    }

    #[test]
    fn test_detect_format_jpeg() {
        let jpeg_header: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_format(jpeg_header), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn test_detect_format_unknown() {
        let garbage: &[u8] = &[0x00, 0x01, 0x02, 0x03];
        assert_eq!(detect_format(garbage), None);
    }

    #[test]
    fn test_detect_format_empty() {
        let empty: &[u8] = &[];
        assert_eq!(detect_format(empty), None);
    }

    #[test]
    fn test_detect_format_rejects_truncated_png_signature() {
        let truncated: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x00, 0x00];
        assert_eq!(detect_format(truncated), None);
    }

    #[test]
    fn test_decode_image_with_format() {
        let data = make_test_png(2, 2);
        let img = decode_image(&data, ImageFormat::Png).expect("decode");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
    }

    #[test]
    fn test_decode_invalid_data() {
        let garbage = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let result = decode_png(&garbage);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_missing_file() {
        let result = load_image(Path::new("/nonexistent/path/to/image.png"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_unsupported_format() {
        let dir = std::env::temp_dir();
        let path = dir.join("ldir_test_unsupported.img");
        std::fs::write(&path, &[0x00, 0x01, 0x02]).expect("write temp");
        let result = load_image(&path);
        assert!(result.is_err());
        let _ = std::fs::remove_file(&path);
    }

    fn make_minimal_png_header(width: u32, height: u32) -> Vec<u8> {
        let mut data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52,
        ];
        data.extend_from_slice(&width.to_be_bytes());
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&[0x08, 0x02, 0x00, 0x00, 0x00]);
        let crc = crc32_png(&data[12..29]);
        data.extend_from_slice(&crc.to_be_bytes());
        data
    }

    fn crc32_png(data: &[u8]) -> u32 {
        let mut crc: u32 = 0xFFFFFFFF;
        for &byte in data {
            crc ^= byte as u32;
            for _ in 0..8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
            }
        }
        !crc
    }

    #[test]
    fn test_detect_png_dimensions() {
        let data = make_minimal_png_header(800, 600);
        let dims = detect_png_dimensions(&data);
        assert_eq!(
            dims,
            Some(ImageDimensions {
                width: 800,
                height: 600
            })
        );
    }

    #[test]
    fn test_detect_png_dimensions_small() {
        let data = make_minimal_png_header(1, 1);
        let dims = detect_png_dimensions(&data);
        assert_eq!(
            dims,
            Some(ImageDimensions {
                width: 1,
                height: 1
            })
        );
    }

    fn make_minimal_jpeg_with_sof0(width: u16, height: u16) -> Vec<u8> {
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x02];
        data.extend_from_slice(&[0xFF, 0xC0, 0x00, 0x0B, 0x08]);
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&width.to_be_bytes());
        data.extend_from_slice(&[0x03, 0x01, 0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01]);
        data
    }

    #[test]
    fn test_detect_jpeg_dimensions() {
        let data = make_minimal_jpeg_with_sof0(1920, 1080);
        let dims = detect_jpeg_dimensions(&data);
        assert_eq!(
            dims,
            Some(ImageDimensions {
                width: 1920,
                height: 1080
            })
        );
    }

    #[test]
    fn test_detect_jpeg_dimensions_sof2() {
        let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x02];
        data.extend_from_slice(&[0xFF, 0xC2, 0x00, 0x0B, 0x08]);
        data.extend_from_slice(&640u16.to_be_bytes());
        data.extend_from_slice(&480u16.to_be_bytes());
        data.extend_from_slice(&[0x03, 0x01, 0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01]);
        let dims = detect_jpeg_dimensions(&data);
        assert_eq!(
            dims,
            Some(ImageDimensions {
                width: 480,
                height: 640
            })
        );
    }

    #[test]
    fn test_detect_invalid_returns_none() {
        assert_eq!(detect_png_dimensions(&[]), None);
        assert_eq!(detect_png_dimensions(&[0x00, 0x01]), None);
        assert_eq!(detect_jpeg_dimensions(&[]), None);
        assert_eq!(detect_jpeg_dimensions(&[0x00, 0x01]), None);
        assert_eq!(
            detect_png_dimensions(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            None
        );
    }

    #[test]
    fn test_detect_png_dimensions_validates_magic() {
        let mut data = make_minimal_png_header(100, 200);
        data[0] = 0x00;
        assert_eq!(detect_png_dimensions(&data), None);
    }

    #[test]
    fn test_detect_png_dimensions_validates_ihdr() {
        let mut data = make_minimal_png_header(100, 200);
        data[12] = 0x00;
        assert_eq!(detect_png_dimensions(&data), None);
    }

    #[test]
    fn test_scale_to_fit_wider_than_max() {
        let (w, h) = scale_to_fit(200, 100, 100.0, 1000.0);
        assert!((w - 100.0).abs() < 0.01);
        assert!((h - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_scale_to_fit_no_scale_needed() {
        let (w, h) = scale_to_fit(50, 50, 100.0, 1000.0);
        assert!((w - 50.0).abs() < 0.01);
        assert!((h - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_scale_to_fit_height_constrained() {
        let (w, h) = scale_to_fit(100, 200, 1000.0, 50.0);
        assert!((w - 25.0).abs() < 0.01);
        assert!((h - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_scale_to_fit_both_constrained() {
        let (w, h) = scale_to_fit(400, 400, 100.0, 100.0);
        assert!((w - 100.0).abs() < 0.01);
        assert!((h - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_scale_to_fit_aspect_ratio_preserved() {
        let (w, h) = scale_to_fit(1600, 900, 500.0, 500.0);
        let ratio_before = 1600.0 / 900.0;
        let ratio_after = w / h;
        assert!((ratio_before - ratio_after).abs() < 0.001);
    }

    #[test]
    fn test_scale_to_fit_zero_dimensions() {
        let (w, h) = scale_to_fit(0, 100, 500.0, 500.0);
        assert!((w - 0.0).abs() < 0.01);
        assert!((h - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_load_png_from_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("ldir_test_image.png");
        let png_data = make_test_png(8, 6);
        std::fs::write(&path, &png_data).expect("write temp PNG");

        let img = load_image(&path).expect("load PNG from file");
        assert_eq!(img.width, 8);
        assert_eq!(img.height, 6);
        assert_eq!(img.color_space, ColorSpace::RGB);
        assert_eq!(img.bits_per_component, 8);
        assert_eq!(img.data.len(), 8 * 6 * 3);

        let _ = std::fs::remove_file(&path);
    }
}
