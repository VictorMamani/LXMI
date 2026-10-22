//! Offline insight into 3DMigoto mods (ZZMI, GIMI and similar): what a mod
//! replaces, which character it targets, and software-rendered previews of
//! its default model. It never touches the file system directly: callers
//! expose the mod payload through [`ModSource`].
mod analysis;
mod ini;
mod render;
mod texture;

use serde::{Deserialize, Serialize};

/// One payload file, with `/` separators relative to the mod root.
#[derive(Debug, Clone)]
pub struct ModFile {
    pub path: String,
    pub size: u64,
}

pub trait ModSource {
    fn files(&self) -> Vec<ModFile>;
    /// Returns `None` when the file is missing, unreadable or over `limit` bytes.
    fn read(&self, path: &str, limit: u64) -> Option<Vec<u8>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModKind {
    /// Replaces meshes: a character, outfit, weapon or other 3D model.
    Model,
    /// Replaces textures only.
    Texture,
    /// Interface art: icons, HUD, portraits, menus.
    Ui,
    /// Shader overrides or ShaderFixes without textures.
    Shader,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ModInsight {
    pub kind: ModKind,
    /// Best guess from the INI section names, e.g. `Nicole`.
    pub character: Option<String>,
    /// A model whose buffers were found, so a preview can be attempted.
    pub previewable: bool,
}

/// Cheap: reads only INI files.
pub fn inspect(name: &str, source: &dyn ModSource) -> ModInsight {
    let files = analysis::Files::new(source.files());
    let inis = analysis::load_inis(source, &files);
    let insight = analysis::inspect(name, &inis, &files);
    ModInsight {
        kind: insight.kind,
        character: insight.character,
        previewable: insight.previewable,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewView {
    /// Three-quarter view from the front.
    Front,
    /// Three-quarter view from the back.
    Back,
    /// The front view at a fifth of the size, for lists.
    Thumbnail,
}

impl PreviewView {
    pub const ALL: [PreviewView; 3] = [
        PreviewView::Front,
        PreviewView::Back,
        PreviewView::Thumbnail,
    ];

    pub fn file_name(self) -> &'static str {
        match self {
            PreviewView::Front => "front.png",
            PreviewView::Back => "back.png",
            PreviewView::Thumbnail => "thumbnail.png",
        }
    }
}

const THUMBNAIL_FACTOR: usize = 5;

fn camera(azimuth: f32) -> render::View {
    render::View {
        azimuth,
        elevation: 8.0,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PreviewOptions {
    pub width: u32,
    pub height: u32,
    /// Samples per pixel along each axis; 2 smooths edges at 4× the cost.
    pub supersample: u32,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            width: 480,
            height: 640,
            supersample: 2,
        }
    }
}

pub struct Preview {
    pub view: PreviewView,
    /// Transparent-background PNG.
    pub png: Vec<u8>,
}

/// Renders the default look of the mod (toggles at their `[Constants]`
/// defaults). Returns an empty list when the mod has no drawable model.
/// Slow: reads every referenced buffer and decodes albedo textures.
pub fn render_previews(source: &dyn ModSource, options: PreviewOptions) -> Vec<Preview> {
    let files = analysis::Files::new(source.files());
    let inis = analysis::load_inis(source, &files);
    let mut draws: Vec<_> = inis
        .iter()
        .flat_map(|ini| analysis::draws(ini, &files))
        .collect();
    analysis::main_subject(&mut draws);
    analysis::inherit_albedo(&mut draws);
    let meshes = render::Loader::new(source).submeshes(&draws);
    if meshes.is_empty() {
        return vec![];
    }
    let (width, height) = (options.width as usize, options.height as usize);
    let supersample = options.supersample.clamp(1, 4) as usize;
    let mut out = vec![];
    for (view, azimuth) in [(PreviewView::Front, 205.0), (PreviewView::Back, 335.0)] {
        let Some(rgba) = render::render(&meshes, camera(azimuth), width, height, supersample)
        else {
            continue;
        };
        let Some(png) = encode_png(&rgba, width, height) else {
            continue;
        };
        out.push(Preview { view, png });
        if view == PreviewView::Front {
            let (w, h) = (width / THUMBNAIL_FACTOR, height / THUMBNAIL_FACTOR);
            let small = render::downscale(&rgba, width, THUMBNAIL_FACTOR, w, h);
            if let Some(png) = encode_png(&small, w, h) {
                out.push(Preview {
                    view: PreviewView::Thumbnail,
                    png,
                });
            }
        }
    }
    out
}

fn encode_png(rgba: &[u8], width: usize, height: usize) -> Option<Vec<u8>> {
    if width == 0 || height == 0 {
        return None;
    }
    let mut out = vec![];
    let mut encoder = png::Encoder::new(&mut out, width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Best);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(rgba).ok()?;
    writer.finish().ok()?;
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Memory(HashMap<String, Vec<u8>>);
    impl ModSource for Memory {
        fn files(&self) -> Vec<ModFile> {
            self.0
                .iter()
                .map(|(path, bytes)| ModFile {
                    path: path.clone(),
                    size: bytes.len() as u64,
                })
                .collect()
        }
        fn read(&self, path: &str, limit: u64) -> Option<Vec<u8>> {
            self.0
                .get(path)
                .filter(|b| b.len() as u64 <= limit)
                .cloned()
        }
    }

    const MODEL_INI: &str = "\
[Constants]
global persist $swap = 0

[TextureOverrideTestaBodyPosition]
hash = 00000000
vb0 = ResourceTestaBodyPosition

[TextureOverrideTestaBodyA]
hash = 00000001
ib = ResourceTestaBodyIB
if $swap == 1
  drawindexed = 3, 0, 0
else
  drawindexed = 6, 0, 0
endif

[ResourceTestaBodyPosition]
type = Buffer
stride = 40
filename = Meshes\\TestaBodyPosition.buf

[ResourceTestaBodyIB]
type = Buffer
format = DXGI_FORMAT_R32_UINT
filename = Meshes\\TestaBodyA.ib
";

    /// A unit square standing upright (X/Z plane), facing the camera.
    fn model() -> Memory {
        let mut vb = vec![];
        for [x, z] in [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
            for v in [x, 0.0, z, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0] {
                vb.extend(f32::to_le_bytes(v));
            }
        }
        let ib: Vec<u8> = [0u32, 1, 2, 0, 2, 3]
            .iter()
            .flat_map(|i| i.to_le_bytes())
            .collect();
        Memory(HashMap::from([
            ("mod.ini".into(), MODEL_INI.as_bytes().to_vec()),
            ("meshes/TestaBodyPosition.buf".into(), vb),
            ("meshes/TestaBodyA.ib".into(), ib),
        ]))
    }

    #[test]
    fn classifies_a_model_and_its_character() {
        let insight = inspect("Some outfit", &model());
        assert_eq!(insight.kind, ModKind::Model);
        assert_eq!(insight.character.as_deref(), Some("Testa"));
        assert!(insight.previewable);
    }

    #[test]
    fn renders_both_views_of_the_default_toggle() {
        let previews = render_previews(
            &model(),
            PreviewOptions {
                width: 64,
                height: 64,
                supersample: 1,
            },
        );
        let views: Vec<_> = previews.iter().map(|p| p.view).collect();
        assert_eq!(views.len(), 3);
        assert!(PreviewView::ALL.iter().all(|v| views.contains(v)));
        let mut decoder = png::Decoder::new(previews[0].png.as_slice());
        decoder.set_transformations(png::Transformations::EXPAND);
        let mut reader = decoder.read_info().unwrap();
        let mut rgba = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut rgba).unwrap();
        let covered = rgba.chunks(4).filter(|p| p[3] == 255).count();
        let corner = &rgba[0..4];
        // Both triangles drawn (default branch), background left transparent.
        assert!(covered > 64 * 64 / 4, "covered {covered}");
        assert_eq!(corner[3], 0);
    }

    #[test]
    fn classifies_texture_ui_and_shader_mods() {
        let texture = Memory(HashMap::from([
            (
                "a.ini".into(),
                b"[TextureOverrideHairDiffuse]\nhash = 1\nthis = ResourceHair\n[ResourceHair]\nfilename = hair.dds\n".to_vec(),
            ),
            ("hair.dds".into(), vec![0; 8]),
        ]));
        assert_eq!(inspect("Hair recolor", &texture).kind, ModKind::Texture);
        assert_eq!(inspect("Hair UI", &texture).kind, ModKind::Ui);

        let shader = Memory(HashMap::from([(
            "fx.ini".into(),
            b"[ShaderOverrideBloom]\nhash = 1\nhandling = skip\n".to_vec(),
        )]));
        let insight = inspect("No bloom", &shader);
        assert_eq!(insight.kind, ModKind::Shader);
        assert!(!insight.previewable);
        assert!(render_previews(&shader, PreviewOptions::default()).is_empty());
    }

    #[test]
    fn ignores_disabled_inis() {
        let mut mod_files = model();
        let ini = mod_files.0.remove("mod.ini").unwrap();
        mod_files.0.insert("DISABLED_mod.ini".into(), ini);
        assert_eq!(inspect("x", &mod_files).kind, ModKind::Other);
    }
}
