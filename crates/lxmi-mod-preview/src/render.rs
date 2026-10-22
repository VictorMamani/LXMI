//! Loads the default mesh of a mod and draws it with a small z-buffered
//! software rasterizer: textured albedo and one directional light.
use crate::analysis::{common_prefix, Draw, DrawRange, VertexBuffer};
use crate::texture::Texture;
use crate::ModSource;
use std::collections::HashMap;
use std::rc::Rc;

const MAX_BUFFER_BYTES: u64 = 256 * 1024 * 1024;
const MAX_TEXTURE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_TRIANGLES: usize = 4_000_000;
const CLAY: [f32; 3] = [184.0, 184.0, 194.0];

struct Vertices {
    positions: Vec<[f32; 3]>,
    normals: Option<Vec<[f32; 3]>>,
    uvs: Option<Vec<[f32; 2]>>,
}

pub(crate) struct Submesh {
    vertices: Rc<Vertices>,
    triangles: Vec<[u32; 3]>,
    texture: Option<Rc<Texture>>,
}

/// Caches decoded files: draws of one object share buffers and textures.
pub(crate) struct Loader<'a> {
    source: &'a dyn ModSource,
    indices: HashMap<String, Option<Rc<Vec<u32>>>>,
    vertices: HashMap<(String, String), Option<Rc<Vertices>>>,
    textures: HashMap<String, Option<Rc<Texture>>>,
}

impl<'a> Loader<'a> {
    pub fn new(source: &'a dyn ModSource) -> Self {
        Self {
            source,
            indices: HashMap::new(),
            vertices: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn submeshes(&mut self, draws: &[Draw]) -> Vec<Submesh> {
        let mut out = vec![];
        let mut triangles = 0;
        for draw in draws {
            let Some(ib) = self.indices(&draw.ib_path, draw.ib_16bit) else {
                continue;
            };
            let ranges: Vec<(usize, usize, Option<&String>)> = draw
                .ranges
                .iter()
                .filter_map(|(range, albedo)| {
                    let (start, count) = match *range {
                        DrawRange::Auto => (0, ib.len()),
                        DrawRange::Indexed { count, start } => (start, count),
                    };
                    let end = start.checked_add(count)?;
                    (end <= ib.len() && count >= 3).then_some((start, count, albedo.as_ref()))
                })
                .collect();
            let Some(need) = ranges
                .iter()
                .flat_map(|(start, count, _)| &ib[*start..start + count])
                .max()
                .map(|max| *max as usize + 1)
            else {
                continue;
            };
            let pick = |bound: &Option<VertexBuffer>, candidates: &[VertexBuffer]| {
                bound
                    .clone()
                    .filter(|b| b.count >= need)
                    .or_else(|| best_buffer(candidates, &draw.name, need))
            };
            let Some(position) = pick(&draw.bound_position, &draw.position_candidates) else {
                continue;
            };
            let texcoord = pick(&draw.bound_texcoord, &draw.texcoord_candidates);
            let Some(vertices) = self.vertices(&position, texcoord.as_ref()) else {
                continue;
            };
            for (start, count, albedo) in ranges {
                let tris = ib[start..start + count].as_chunks::<3>().0.to_vec();
                triangles += tris.len();
                if triangles > MAX_TRIANGLES {
                    return out;
                }
                out.push(Submesh {
                    vertices: vertices.clone(),
                    triangles: tris,
                    texture: albedo.and_then(|path| self.texture(path)),
                });
            }
        }
        out
    }

    fn indices(&mut self, path: &str, sixteen: bool) -> Option<Rc<Vec<u32>>> {
        let source = self.source;
        self.indices
            .entry(path.to_owned())
            .or_insert_with(|| {
                let bytes = source.read(path, MAX_BUFFER_BYTES)?;
                let ib: Vec<u32> = if sixteen {
                    bytes
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|b| u16::from_le_bytes(*b) as u32)
                        .collect()
                } else {
                    bytes
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|b| u32::from_le_bytes(*b))
                        .collect()
                };
                Some(Rc::new(ib))
            })
            .clone()
    }

    fn vertices(
        &mut self,
        position: &VertexBuffer,
        texcoord: Option<&VertexBuffer>,
    ) -> Option<Rc<Vertices>> {
        let key = (
            position.path.clone(),
            texcoord.map(|t| t.path.clone()).unwrap_or_default(),
        );
        if position.stride < 12 {
            return None;
        }
        let source = self.source;
        self.vertices
            .entry(key)
            .or_insert_with(|| {
                let bytes = source.read(&position.path, MAX_BUFFER_BYTES)?;
                let rows = bytes.chunks_exact(position.stride);
                let positions: Vec<[f32; 3]> = rows.clone().map(|r| vec3(r, 0)).collect();
                // ZZMI/GIMI layout: position, normal, tangent (40 bytes).
                let normals = (position.stride >= 24).then(|| rows.map(|r| vec3(r, 12)).collect());
                // COLOR (4 bytes) then TEXCOORD0 as two halves.
                let uvs = texcoord.and_then(|t| {
                    let bytes = source.read(&t.path, MAX_BUFFER_BYTES)?;
                    Some(
                        bytes
                            .chunks_exact(t.stride)
                            .map(|r| [half(r, 4), half(r, 6)])
                            .collect(),
                    )
                });
                Some(Rc::new(Vertices {
                    positions,
                    normals,
                    uvs,
                }))
            })
            .clone()
    }

    fn texture(&mut self, path: &str) -> Option<Rc<Texture>> {
        let source = self.source;
        self.textures
            .entry(path.to_owned())
            .or_insert_with(|| {
                let bytes = source.read(path, MAX_TEXTURE_BYTES)?;
                Texture::decode(path, &bytes).map(Rc::new)
            })
            .clone()
    }
}

/// Fallback when the INI has no explicit vb0/vb1 binding: the buffer whose
/// resource name shares the longest prefix with the draw section.
fn best_buffer(candidates: &[VertexBuffer], name: &str, need: usize) -> Option<VertexBuffer> {
    let name = name.to_lowercase();
    candidates
        .iter()
        .filter(|c| c.count >= need)
        .max_by_key(|c| {
            let lower = c.name.to_lowercase();
            let shared = common_prefix(&name, &lower);
            (
                shared,
                std::cmp::Reverse(lower.len() - shared),
                c.name.clone(),
            )
        })
        .cloned()
}

fn vec3(row: &[u8], offset: usize) -> [f32; 3] {
    std::array::from_fn(|i| {
        let b = &row[offset + i * 4..offset + i * 4 + 4];
        let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        if v.is_finite() {
            v
        } else {
            0.0
        }
    })
}

fn half(row: &[u8], offset: usize) -> f32 {
    let Some(b) = row.get(offset..offset + 2) else {
        return 0.0;
    };
    let bits = u16::from_le_bytes([b[0], b[1]]);
    let sign = if bits & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exponent = (bits >> 10) & 0x1f;
    let mantissa = (bits & 0x3ff) as f32;
    match exponent {
        0 => sign * mantissa * 2f32.powi(-24),
        31 => 0.0,
        e => sign * (1.0 + mantissa / 1024.0) * 2f32.powi(e as i32 - 15),
    }
}

/// Camera around the vertical (Z) axis, in degrees.
#[derive(Clone, Copy)]
pub(crate) struct View {
    pub azimuth: f32,
    pub elevation: f32,
}

/// RGBA image, transparent where nothing is drawn. Rendered at `supersample`
/// times the size and averaged down to smooth edges.
pub(crate) fn render(
    meshes: &[Submesh],
    view: View,
    width: usize,
    height: usize,
    supersample: usize,
) -> Option<Vec<u8>> {
    // Frame only vertices that are actually drawn.
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for mesh in meshes {
        for &i in mesh.triangles.iter().flatten() {
            if let Some(p) = mesh.vertices.positions.get(i as usize) {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
        }
    }
    let extent = (0..3).map(|k| hi[k] - lo[k]).fold(0.0, f32::max);
    if extent.is_nan() || extent <= 0.0 {
        return None;
    }
    let center: [f32; 3] = std::array::from_fn(|k| (lo[k] + hi[k]) / 2.0);
    let scale = 1.0 / extent;

    let (sa, ca) = view.azimuth.to_radians().sin_cos();
    let (se, ce) = view.elevation.to_radians().sin_cos();
    // Model Z is up; returns screen x, screen y (up) and depth (larger is nearer).
    let project = |p: [f32; 3]| {
        let x = p[0] * ca - p[1] * sa;
        let y = p[0] * sa + p[1] * ca;
        let z = p[2];
        [x, z * ce - y * se, y * ce + z * se]
    };
    let normalized =
        |p: &[f32; 3]| -> [f32; 3] { project(std::array::from_fn(|k| (p[k] - center[k]) * scale)) };

    let (mut x_range, mut y_range) = ((f32::MAX, f32::MIN), (f32::MAX, f32::MIN));
    for mesh in meshes {
        for &i in mesh.triangles.iter().flatten() {
            if let Some(p) = mesh.vertices.positions.get(i as usize) {
                let v = normalized(p);
                x_range = (x_range.0.min(v[0]), x_range.1.max(v[0]));
                y_range = (y_range.0.min(v[1]), y_range.1.max(v[1]));
            }
        }
    }
    let (w, h) = (width * supersample, height * supersample);
    let fit = (w as f32 * 0.94 / (x_range.1 - x_range.0).max(1e-6))
        .min(h as f32 * 0.94 / (y_range.1 - y_range.0).max(1e-6));
    let (mid_x, mid_y) = ((x_range.0 + x_range.1) / 2.0, (y_range.0 + y_range.1) / 2.0);

    let light = normalize([0.35, 0.72, 0.60]);
    let mut depth = vec![f32::MIN; w * h];
    let mut color = vec![[0.0f32; 3]; w * h];

    for mesh in meshes {
        let vertices = &mesh.vertices;
        let count = vertices.positions.len();
        for tri in &mesh.triangles {
            if tri.iter().any(|&i| i as usize >= count) {
                continue;
            }
            let idx = tri.map(|i| i as usize);
            let v = idx.map(|i| normalized(&vertices.positions[i]));
            let sx = v.map(|p| (p[0] - mid_x) * fit + w as f32 / 2.0);
            let sy = v.map(|p| h as f32 / 2.0 - (p[1] - mid_y) * fit);
            let sz = v.map(|p| p[2] * fit);
            let area = (sx[1] - sx[0]) * (sy[2] - sy[0]) - (sx[2] - sx[0]) * (sy[1] - sy[0]);
            if area.abs() < 1e-9 {
                continue;
            }
            let normals = vertices
                .normals
                .as_ref()
                .map(|n| idx.map(|i| normalize(project(n[i]))));
            // Without vertex normals, light the face from either side.
            let face = normalize(cross(sub(v[1], v[0]), sub(v[2], v[0])));
            let uvs = vertices.uvs.as_ref().and_then(|uv| {
                idx.iter()
                    .map(|&i| uv.get(i).copied())
                    .collect::<Option<Vec<_>>>()
            });

            let min_x = sx.iter().copied().fold(f32::MAX, f32::min).floor().max(0.0) as usize;
            let max_x =
                (sx.iter().copied().fold(f32::MIN, f32::max).ceil() as isize).min(w as isize - 1);
            let min_y = sy.iter().copied().fold(f32::MAX, f32::min).floor().max(0.0) as usize;
            let max_y =
                (sy.iter().copied().fold(f32::MIN, f32::max).ceil() as isize).min(h as isize - 1);
            if max_x < 0 || max_y < 0 {
                continue;
            }
            for py in min_y..=max_y as usize {
                let y = py as f32 + 0.5;
                for px in min_x..=max_x as usize {
                    let x = px as f32 + 0.5;
                    let w0 = ((sx[1] - x) * (sy[2] - y) - (sx[2] - x) * (sy[1] - y)) / area;
                    let w1 = ((sx[2] - x) * (sy[0] - y) - (sx[0] - x) * (sy[2] - y)) / area;
                    let w2 = 1.0 - w0 - w1;
                    if w0 < -1e-6 || w1 < -1e-6 || w2 < -1e-6 {
                        continue;
                    }
                    let z = w0 * sz[0] + w1 * sz[1] + w2 * sz[2];
                    let at = py * w + px;
                    if z <= depth[at] {
                        continue;
                    }
                    depth[at] = z;
                    let albedo = match (&mesh.texture, &uvs) {
                        (Some(texture), Some(uv)) => texture.sample(
                            w0 * uv[0][0] + w1 * uv[1][0] + w2 * uv[2][0],
                            w0 * uv[0][1] + w1 * uv[1][1] + w2 * uv[2][1],
                        ),
                        _ => CLAY,
                    };
                    let shade = match &normals {
                        Some(n) => {
                            let n = normalize(std::array::from_fn(|k| {
                                w0 * n[0][k] + w1 * n[1][k] + w2 * n[2][k]
                            }));
                            let d = dot(n, light);
                            0.30 + 0.75 * d.max(0.0) + 0.18 * (-d).max(0.0)
                        }
                        None => 0.30 + 0.75 * dot(face, light).abs(),
                    };
                    color[at] = albedo.map(|c| (c * shade).min(255.0));
                }
            }
        }
    }

    let mut rgba = vec![0u8; w * h * 4];
    for (at, c) in color.iter().enumerate() {
        if depth[at] > f32::MIN {
            rgba[at * 4..at * 4 + 3].copy_from_slice(&c.map(|v| v.round() as u8));
            rgba[at * 4 + 3] = 255;
        }
    }
    Some(downscale(&rgba, w, supersample, width, height))
}

/// Averages `factor`×`factor` blocks of an RGBA image whose rows are
/// `stride` pixels wide. Colour is weighted by alpha so edges do not darken.
pub(crate) fn downscale(
    rgba: &[u8],
    stride: usize,
    factor: usize,
    width: usize,
    height: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; width * height * 4];
    let samples = (factor * factor) as f32;
    for y in 0..height {
        for x in 0..width {
            let (mut sum, mut alpha) = ([0.0f32; 3], 0.0f32);
            for sy in 0..factor {
                for sx in 0..factor {
                    let at = ((y * factor + sy) * stride + x * factor + sx) * 4;
                    let a = rgba[at + 3] as f32;
                    alpha += a;
                    for k in 0..3 {
                        sum[k] += rgba[at + k] as f32 * a;
                    }
                }
            }
            let o = (y * width + x) * 4;
            if alpha > 0.0 {
                for k in 0..3 {
                    out[o + k] = (sum[k] / alpha).round() as u8;
                }
                out[o + 3] = (alpha / samples).round() as u8;
            }
        }
    }
    out
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|k| a[k] - b[k])
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = dot(v, v).sqrt().max(1e-8);
    v.map(|c| c / length)
}

#[cfg(test)]
mod tests {
    use super::half;

    #[test]
    fn decodes_half_floats() {
        let at = |bits: u16| half(&bits.to_le_bytes(), 0);
        assert_eq!(at(0x3c00), 1.0);
        assert_eq!(at(0x3800), 0.5);
        assert_eq!(at(0xc000), -2.0);
        assert_eq!(at(0x0000), 0.0);
    }
}
