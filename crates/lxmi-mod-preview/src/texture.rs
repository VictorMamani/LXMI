//! Albedo textures: DDS (BCn and plain 8-bit formats, top mip only) and PNG.

pub(crate) struct Texture {
    pub width: usize,
    pub height: usize,
    /// RGB, row-major from the top row.
    pub rgb: Vec<[u8; 3]>,
}

impl Texture {
    pub fn decode(path: &str, bytes: &[u8]) -> Option<Self> {
        if path.to_ascii_lowercase().ends_with(".png") {
            return decode_png(bytes);
        }
        decode_dds(bytes)
    }

    pub fn sample(&self, u: f32, v: f32) -> [f32; 3] {
        let x = u.clamp(0.0, 1.0) * (self.width - 1) as f32;
        let y = v.clamp(0.0, 1.0) * (self.height - 1) as f32;
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.width - 1), (y0 + 1).min(self.height - 1));
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let at = |x: usize, y: usize| self.rgb[y * self.width + x];
        let (c00, c10, c01, c11) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
        std::array::from_fn(|i| {
            let top = c00[i] as f32 * (1.0 - fx) + c10[i] as f32 * fx;
            let bottom = c01[i] as f32 * (1.0 - fx) + c11[i] as f32 * fx;
            top * (1.0 - fy) + bottom * fy
        })
    }
}

const MAX_DIMENSION: usize = 8192;

/// A texture2ddecoder block decoder: data, width, height, output pixels.
type BlockDecoder = fn(&[u8], usize, usize, &mut [u32]) -> Result<(), &'static str>;

enum Format {
    /// Decoder and bytes per 4×4 block.
    Block(BlockDecoder, usize),
    Rgba8,
    Bgra8,
}

fn decode_dds(bytes: &[u8]) -> Option<Texture> {
    if bytes.get(0..4)? != b"DDS " {
        return None;
    }
    let u32_at = |offset: usize| {
        bytes
            .get(offset..offset + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let height = u32_at(12)? as usize;
    let width = u32_at(16)? as usize;
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return None;
    }
    let fourcc = bytes.get(84..88)?;
    let (format, offset) = if fourcc == b"DX10" {
        (dxgi_format(u32_at(128)?)?, 148)
    } else {
        (legacy_format(fourcc, u32_at(88)?, u32_at(108)?)?, 128)
    };
    let data = bytes.get(offset..)?;
    let mut pixels = vec![0u32; width * height];
    match format {
        Format::Block(decode, block_bytes) => {
            let size = width.div_ceil(4) * height.div_ceil(4) * block_bytes;
            decode(data.get(..size)?, width, height, &mut pixels).ok()?;
        }
        Format::Rgba8 | Format::Bgra8 => {
            let rgba = matches!(format, Format::Rgba8);
            for (pixel, c) in pixels
                .iter_mut()
                .zip(data.get(..width * height * 4)?.chunks(4))
            {
                let (r, b) = if rgba { (c[0], c[2]) } else { (c[2], c[0]) };
                *pixel = u32::from_le_bytes([b, c[1], r, c[3]]);
            }
        }
    }
    // The decoder writes 0xAARRGGBB.
    let rgb = pixels
        .iter()
        .map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
        .collect();
    Some(Texture { width, height, rgb })
}

fn dxgi_format(code: u32) -> Option<Format> {
    use texture2ddecoder as t;
    Some(match code {
        70..=72 => Format::Block(t::decode_bc1, 8),
        73..=75 => Format::Block(t::decode_bc2, 16),
        76..=78 => Format::Block(t::decode_bc3, 16),
        79..=81 => Format::Block(t::decode_bc4, 8),
        82..=84 => Format::Block(t::decode_bc5, 16),
        94 | 95 => Format::Block(t::decode_bc6_unsigned, 16),
        96 => Format::Block(t::decode_bc6_signed, 16),
        97..=99 => Format::Block(t::decode_bc7, 16),
        27..=29 => Format::Rgba8,
        87 | 88 | 90..=93 => Format::Bgra8,
        _ => return None,
    })
}

/// Pre-DX10 headers: FourCC block formats and 32-bit RGB masks.
fn legacy_format(fourcc: &[u8], pf_flags: u32, red_mask: u32) -> Option<Format> {
    use texture2ddecoder as t;
    const FOURCC: u32 = 0x4;
    if pf_flags & FOURCC == 0 {
        return match red_mask {
            0x0000_00ff => Some(Format::Rgba8),
            0x00ff_0000 => Some(Format::Bgra8),
            _ => None,
        };
    }
    Some(match fourcc {
        b"DXT1" => Format::Block(t::decode_bc1, 8),
        b"DXT2" | b"DXT3" => Format::Block(t::decode_bc2, 16),
        b"DXT4" | b"DXT5" => Format::Block(t::decode_bc3, 16),
        b"ATI1" | b"BC4U" => Format::Block(t::decode_bc4, 8),
        b"ATI2" | b"BC5U" => Format::Block(t::decode_bc5, 16),
        _ => return None,
    })
}

fn decode_png(bytes: &[u8]) -> Option<Texture> {
    let mut decoder = png::Decoder::new(bytes);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let (width, height) = (reader.info().width as usize, reader.info().height as usize);
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return None;
    }
    let mut buffer = vec![0; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buffer).ok()?;
    let channels = frame.color_type.samples();
    let rgb = buffer[..frame.buffer_size()]
        .chunks(channels)
        .map(|c| match channels {
            1 | 2 => [c[0]; 3],
            _ => [c[0], c[1], c[2]],
        })
        .collect();
    Some(Texture { width, height, rgb })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dds_rgba(width: u32, height: u32, pixels: &[[u8; 4]]) -> Vec<u8> {
        let mut bytes = vec![0u8; 148];
        bytes[0..4].copy_from_slice(b"DDS ");
        bytes[12..16].copy_from_slice(&height.to_le_bytes());
        bytes[16..20].copy_from_slice(&width.to_le_bytes());
        bytes[80..84].copy_from_slice(&4u32.to_le_bytes());
        bytes[84..88].copy_from_slice(b"DX10");
        bytes[128..132].copy_from_slice(&28u32.to_le_bytes());
        bytes.extend(pixels.iter().flatten());
        bytes
    }

    #[test]
    fn decodes_uncompressed_dx10() {
        let texture = Texture::decode(
            "a.dds",
            &dds_rgba(2, 1, &[[255, 0, 0, 255], [0, 0, 255, 255]]),
        )
        .unwrap();
        assert_eq!(texture.rgb, [[255, 0, 0], [0, 0, 255]]);
        assert_eq!(texture.sample(1.0, 0.0), [0.0, 0.0, 255.0]);
    }

    #[test]
    fn rejects_truncated_block_data() {
        let mut bytes = dds_rgba(8, 8, &[]);
        bytes[128..132].copy_from_slice(&98u32.to_le_bytes());
        assert!(Texture::decode("a.dds", &bytes).is_none());
    }
}
