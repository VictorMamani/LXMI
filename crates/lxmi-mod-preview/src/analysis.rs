//! Reads a mod's INI files into draw calls, buffer bindings and metadata.
//! Only INI text is read here; buffers and textures are loaded by the renderer.
use crate::ini::{self, Section};
use crate::{ModFile, ModKind, ModSource};
use std::collections::HashMap;

const MAX_INIS: usize = 64;
const MAX_INI_BYTES: u64 = 4 * 1024 * 1024;

/// Payload paths, looked up case-insensitively: mods are written on Windows.
pub(crate) struct Files {
    by_lower: HashMap<String, ModFile>,
}

impl Files {
    pub fn new(files: Vec<ModFile>) -> Self {
        Self {
            by_lower: files
                .into_iter()
                .map(|f| (f.path.to_lowercase(), f))
                .collect(),
        }
    }

    /// Resolves an INI `filename` relative to the INI directory, never above the root.
    pub fn resolve(&self, dir: &str, filename: &str) -> Option<&ModFile> {
        let mut parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();
        for part in filename.trim().trim_matches('"').split(['\\', '/']) {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop()?;
                }
                _ if part.contains(':') => return None,
                _ => parts.push(part),
            }
        }
        self.by_lower.get(&parts.join("/").to_lowercase())
    }

    fn all(&self) -> impl Iterator<Item = &ModFile> {
        self.by_lower.values()
    }
}

pub(crate) struct IniFile {
    pub dir: String,
    pub sections: Vec<Section>,
}

/// 3DMigoto skips INIs whose name starts with DISABLED; desktop.ini is Explorer's.
pub(crate) fn load_inis(source: &dyn ModSource, files: &Files) -> Vec<IniFile> {
    let mut paths: Vec<&str> = files
        .all()
        .map(|f| f.path.as_str())
        .filter(|path| {
            let lower = path.to_lowercase();
            lower.ends_with(".ini")
                && !lower.ends_with("desktop.ini")
                && !lower.split('/').any(|c| c.starts_with("disabled"))
        })
        .collect();
    paths.sort();
    paths
        .into_iter()
        .take(MAX_INIS)
        .filter_map(|path| {
            let bytes = source.read(path, MAX_INI_BYTES)?;
            Some(IniFile {
                dir: path.rsplit_once('/').map_or("", |(d, _)| d).to_owned(),
                sections: ini::parse(&String::from_utf8_lossy(&bytes)),
            })
        })
        .collect()
}

#[derive(Clone)]
pub(crate) struct VertexBuffer {
    pub name: String,
    pub path: String,
    pub stride: usize,
    pub count: usize,
}

#[derive(Clone, Copy)]
pub(crate) enum DrawRange {
    Auto,
    Indexed { count: usize, start: usize },
}

pub(crate) struct Draw {
    pub name: String,
    /// Character or object named by the section, e.g. `Rina` for `RinaBodyA`.
    pub subject: Option<String>,
    pub ib_path: String,
    pub ib_size: u64,
    pub ib_16bit: bool,
    pub ranges: Vec<(DrawRange, Option<String>)>,
    pub bound_position: Option<VertexBuffer>,
    pub bound_texcoord: Option<VertexBuffer>,
    pub position_candidates: Vec<VertexBuffer>,
    pub texcoord_candidates: Vec<VertexBuffer>,
}

struct Resource {
    name: String,
    file: Option<ModFile>,
    stride: Option<usize>,
    format: String,
}

/// `ResourceX`, `ref ResourceX` or `copy ResourceX` in this INI's namespace.
fn resource_key(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value
        .strip_prefix("ref ")
        .or_else(|| value.strip_prefix("copy "))
        .unwrap_or(value)
        .trim();
    if value.contains('\\') || value.len() <= "Resource".len() {
        return None;
    }
    let (prefix, name) = value.split_at("Resource".len());
    prefix
        .eq_ignore_ascii_case("Resource")
        .then(|| name.to_lowercase())
}

fn resources(ini: &IniFile, files: &Files) -> HashMap<String, Resource> {
    let mut out = HashMap::new();
    for section in &ini.sections {
        let Some(key) = resource_key(&section.name) else {
            continue;
        };
        let mut resource = Resource {
            name: section.name["Resource".len()..].to_owned(),
            file: None,
            stride: None,
            format: String::new(),
        };
        for line in &section.lines {
            match ini::split_kv(line) {
                Some((k, v)) if k.eq_ignore_ascii_case("filename") => {
                    resource.file = files.resolve(&ini.dir, v).cloned();
                }
                Some((k, v)) if k.eq_ignore_ascii_case("stride") => {
                    resource.stride = v.parse().ok().filter(|s| *s > 0);
                }
                Some((k, v)) if k.eq_ignore_ascii_case("format") => {
                    resource.format = v.to_ascii_uppercase();
                }
                _ => {}
            }
        }
        out.insert(key, resource);
    }
    out
}

fn vertex_buffer(resource: &Resource, default_stride: usize) -> Option<VertexBuffer> {
    let file = resource.file.as_ref()?;
    let stride = resource.stride.unwrap_or(default_stride);
    if stride == 0 {
        return None;
    }
    Some(VertexBuffer {
        name: resource.name.clone(),
        path: file.path.clone(),
        stride,
        count: (file.size / stride as u64) as usize,
    })
}

const BINDING_SUFFIXES: [&str; 3] = ["position", "texcoord", "blend"];

/// Every indexed draw in TextureOverride sections, following the default
/// branch of each toggle.
pub(crate) fn draws(ini: &IniFile, files: &Files) -> Vec<Draw> {
    let resources = resources(ini, files);
    let defaults = ini::constants(&ini.sections);
    let lookup = |value: &str| resource_key(value).and_then(|key| resources.get(&key));

    let mut bindings: HashMap<String, (Option<VertexBuffer>, Option<VertexBuffer>)> =
        HashMap::new();
    for section in &ini.sections {
        let Some(short) = override_name(&section.name) else {
            continue;
        };
        let lower = short.to_lowercase();
        let Some(base) = BINDING_SUFFIXES
            .iter()
            .find_map(|suffix| lower.strip_suffix(suffix))
            .filter(|b| !b.is_empty())
        else {
            continue;
        };
        let entry = bindings.entry(base.to_owned()).or_default();
        for line in &section.lines {
            match ini::split_kv(line) {
                Some((k, v)) if k.eq_ignore_ascii_case("vb0") => {
                    entry.0 = lookup(v).and_then(|r| vertex_buffer(r, 40));
                }
                Some((k, v)) if k.eq_ignore_ascii_case("vb1") => {
                    entry.1 = lookup(v).and_then(|r| vertex_buffer(r, 20));
                }
                _ => {}
            }
        }
    }

    let mut positions = vec![];
    let mut texcoords = vec![];
    for resource in resources.values() {
        let (Some(stride), Some(buffer)) = (resource.stride, vertex_buffer(resource, 0)) else {
            continue;
        };
        let name = resource.name.to_lowercase();
        if stride == 40 || (stride == 12 && name.contains("position")) {
            positions.push(buffer);
        } else if matches!(stride, 20 | 24) && name.contains("texcoord") {
            texcoords.push(buffer);
        }
    }

    let mut out = vec![];
    for section in &ini.sections {
        let Some(name) = override_name(&section.name) else {
            continue;
        };
        if name.to_lowercase().contains("shadow") {
            continue;
        }
        let mut stack: Vec<(bool, bool)> = vec![];
        let (mut ib, mut albedo, mut ranges) = (None, None, vec![]);
        for line in &section.lines {
            let lower = line.to_lowercase();
            if let Some(expr) = lower.strip_prefix("if ") {
                let active = ini::condition(expr, &defaults);
                stack.push((active, active));
                continue;
            }
            if let (Some(expr), Some(frame)) = (lower.strip_prefix("elif "), stack.last_mut()) {
                frame.0 = !frame.1 && ini::condition(expr, &defaults);
                frame.1 |= frame.0;
                continue;
            }
            if let ("else", Some(frame)) = (lower.as_str(), stack.last_mut()) {
                frame.0 = !frame.1;
                frame.1 = true;
                continue;
            }
            if lower == "endif" {
                stack.pop();
                continue;
            }
            if !stack.iter().all(|frame| frame.0) {
                continue;
            }
            let Some((key, value)) = ini::split_kv(line) else {
                continue;
            };
            let key = key.to_lowercase();
            if key == "ib" {
                ib = lookup(value);
            } else if key == "ps-t3" || (key.contains("diffuse") && value.starts_with("ref ")) {
                albedo = lookup(value)
                    .and_then(|r| r.file.as_ref())
                    .map(|f| f.path.clone());
            } else if key == "drawindexed" {
                ranges.push((draw_range(value), albedo.clone()));
            }
        }
        let Some((ib, file)) = ib.and_then(|r| Some((r, r.file.as_ref()?))) else {
            continue;
        };
        if ranges.is_empty() {
            continue;
        }
        // Longest binding base that prefixes this section, e.g. `BodyA` → `Body`.
        let lower = name.to_lowercase();
        let binding = bindings
            .iter()
            .filter(|(base, _)| lower.starts_with(base.as_str()))
            .max_by_key(|(base, _)| base.len())
            .map(|(_, b)| b.clone())
            .unwrap_or_default();
        out.push(Draw {
            name: name.to_owned(),
            subject: subject_of(name),
            ib_path: file.path.clone(),
            ib_size: file.size,
            ib_16bit: ib.format.contains("R16"),
            ranges: ranges
                .into_iter()
                .filter_map(|(range, albedo)| Some((range?, albedo)))
                .collect(),
            bound_position: binding.0,
            bound_texcoord: binding.1,
            position_candidates: positions.clone(),
            texcoord_candidates: texcoords.clone(),
        });
    }
    out
}

fn draw_range(value: &str) -> Option<DrawRange> {
    if value.trim().eq_ignore_ascii_case("auto") {
        return Some(DrawRange::Auto);
    }
    let mut parts = value.split(',').map(|p| p.trim().parse::<usize>());
    let count = parts.next()?.ok()?;
    let start = parts.next()?.ok()?;
    (count > 0).then_some(DrawRange::Indexed { count, start })
}

/// `TextureOverrideBodyA` → `BodyA`; `TextureOverride Miyabi` → `Miyabi`.
fn override_name(section: &str) -> Option<&str> {
    let rest = section.get(..15)?;
    rest.eq_ignore_ascii_case("TextureOverride")
        .then(|| section[15..].trim())
        .filter(|name| !name.is_empty())
}

/// Missing albedos are taken from the closest sibling (same name prefix), as
/// mods often bind the diffuse map only on the first draw of an object.
pub(crate) fn inherit_albedo(draws: &mut [Draw]) {
    for i in 0..draws.len() {
        if draws[i].ranges.iter().all(|(_, a)| a.is_some()) {
            continue;
        }
        let name = draws[i].name.to_lowercase();
        let sibling = draws
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .max_by_key(|(_, d)| common_prefix(&name, &d.name.to_lowercase()))
            .map(|(j, _)| j);
        let Some(sibling) = sibling else {
            continue;
        };
        let mut counts: HashMap<&String, usize> = HashMap::new();
        for (_, albedo) in &draws[sibling].ranges {
            if let Some(albedo) = albedo {
                *counts.entry(albedo).or_default() += 1;
            }
        }
        let fallback = counts
            .into_iter()
            .max_by_key(|(path, n)| (*n, std::cmp::Reverse((*path).clone())))
            .map(|(path, _)| path.clone());
        for (_, albedo) in &mut draws[i].ranges {
            if albedo.is_none() {
                albedo.clone_from(&fallback);
            }
        }
    }
}

/// Packs sometimes bundle a second model (a Bangboo next to its agent). Keeps
/// the subject with the most index data, plus draws with generic names.
pub(crate) fn main_subject(draws: &mut Vec<Draw>) -> Option<String> {
    let mut sizes: HashMap<String, (u64, String)> = HashMap::new();
    let mut seen = std::collections::HashSet::new();
    for draw in draws.iter() {
        let Some(subject) = &draw.subject else {
            continue;
        };
        if seen.insert(&draw.ib_path) {
            let entry = sizes
                .entry(subject.to_lowercase())
                .or_insert((0, subject.clone()));
            entry.0 += draw.ib_size;
        }
    }
    let (key, (_, subject)) = sizes
        .into_iter()
        .max_by(|a, b| a.1 .0.cmp(&b.1 .0).then_with(|| b.0.cmp(&a.0)))?;
    draws.retain(|d| d.subject.as_ref().is_none_or(|s| s.to_lowercase() == key));
    Some(subject)
}

pub(crate) fn common_prefix(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

const UI_WORDS: &[&str] = &[
    "ui",
    "hud",
    "icon",
    "icons",
    "portrait",
    "avatar",
    "menu",
    "minimap",
    "loading",
    "font",
    "cursor",
    "crosshair",
    "healthbar",
    "hpbar",
    "banner",
    "splash",
];
const PART_WORDS: &[&str] = &[
    "body",
    "face",
    "hair",
    "head",
    "dress",
    "extra",
    "main",
    "mesh",
    "component",
    "part",
    "model",
    "skin",
    "outfit",
    "texture",
    "override",
    "ib",
    "position",
    "blend",
    "texcoord",
    "vertexlimitraise",
    "eye",
    "eyes",
    "eyebrow",
    "mouth",
    "skirt",
    "coat",
    "weapon",
    "legs",
    "leg",
    "arm",
    "arms",
    "hand",
    "hands",
    "foot",
    "feet",
    "shoes",
    "hat",
    "tail",
    "ears",
    "wings",
    "jacket",
    "shirt",
    "pants",
    "top",
    "bottom",
    "upper",
    "lower",
    "hide",
    "original",
    "accessory",
    "glasses",
    "default",
    "fx",
];

pub(crate) struct Insight {
    pub kind: ModKind,
    pub character: Option<String>,
    pub previewable: bool,
}

pub(crate) fn inspect(name: &str, inis: &[IniFile], files: &Files) -> Insight {
    let mut draws: Vec<Draw> = inis.iter().flat_map(|ini| draws(ini, files)).collect();
    let model_subject = main_subject(&mut draws);
    let previewable = draws
        .iter()
        .any(|d| d.bound_position.is_some() || !d.position_candidates.is_empty());

    let overrides: Vec<&str> = inis
        .iter()
        .flat_map(|ini| &ini.sections)
        .filter_map(|s| override_name(&s.name))
        .collect();
    let mut tokens = words(name);
    for ini in inis {
        for section in &ini.sections {
            tokens.extend(words(&section.name));
        }
    }
    let ui = tokens.iter().any(|w| UI_WORDS.contains(&w.as_str()));
    let textures = files.all().any(|f| {
        let lower = f.path.to_lowercase();
        [".dds", ".png", ".jpg"].iter().any(|e| lower.ends_with(e))
    });
    let shaders = files
        .all()
        .any(|f| f.path.to_lowercase().starts_with("shaderfixes/"))
        || inis.iter().flat_map(|i| &i.sections).any(|s| {
            let lower = s.name.to_lowercase();
            lower.starts_with("shaderoverride") || lower.starts_with("customshader")
        });

    let kind = if !draws.is_empty() {
        ModKind::Model
    } else if ui {
        ModKind::Ui
    } else if textures && !overrides.is_empty() {
        ModKind::Texture
    } else if shaders {
        ModKind::Shader
    } else {
        ModKind::Other
    };

    // Without a model, the most frequent subject among texture overrides.
    let character = model_subject.or_else(|| {
        let mut votes: HashMap<String, (usize, String)> = HashMap::new();
        for subject in overrides.iter().filter_map(|name| subject_of(name)) {
            votes
                .entry(subject.to_lowercase())
                .or_insert((0, subject))
                .0 += 1;
        }
        votes
            .into_values()
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)))
            .map(|(_, subject)| subject)
    });

    Insight {
        kind,
        character,
        previewable,
    }
}

/// Lowercase words of a name, split on separators, case changes and digits.
fn words(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut word = String::new();
    let mut previous: Option<char> = None;
    for c in text.chars() {
        let boundary =
            !c.is_alphabetic() || previous.is_some_and(|p| p.is_lowercase() && c.is_uppercase());
        if boundary && !word.is_empty() {
            out.push(std::mem::take(&mut word));
        }
        if c.is_alphabetic() {
            word.extend(c.to_lowercase());
        }
        previous = Some(c);
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// First meaningful word of an override name: `NicoleBohe4Head` → `Nicole`,
/// `Claretbody1A` → `Claret`. Generic part names alone yield nothing.
fn subject_of(name: &str) -> Option<String> {
    let first = name
        .split(|c: char| !c.is_alphanumeric())
        .find(|w| !w.is_empty())?;
    if first.len() >= 6 && first.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let word = words(first).into_iter().next()?;
    let mut subject = word.as_str();
    if let Some(stem) = PART_WORDS
        .iter()
        .filter_map(|part| subject.strip_suffix(part))
        .find(|stem| stem.len() >= 3)
    {
        subject = stem;
    }
    if subject.len() < 3 || PART_WORDS.contains(&subject) || UI_WORDS.contains(&subject) {
        return None;
    }
    let mut chars = subject.chars();
    let head = chars.next()?;
    Some(head.to_uppercase().chain(chars).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subjects_come_from_override_names() {
        assert_eq!(subject_of("NicoleBohe4Head").as_deref(), Some("Nicole"));
        assert_eq!(subject_of("Claretbody1A").as_deref(), Some("Claret"));
        assert_eq!(subject_of("Zhao2BodyA").as_deref(), Some("Zhao"));
        assert_eq!(subject_of("Miyabi Small_icon").as_deref(), Some("Miyabi"));
        assert_eq!(subject_of("BodyA"), None);
        assert_eq!(subject_of("_a1b2c3d4"), None);
    }

    #[test]
    fn resolves_paths_relative_to_the_ini() {
        let files = Files::new(vec![ModFile {
            path: "Pack/Hash/Body.buf".into(),
            size: 40,
        }]);
        assert!(files.resolve("Pack", "hash\\body.BUF").is_some());
        assert!(files.resolve("Pack/Sub", "..\\Hash\\Body.buf").is_some());
        assert!(files.resolve("", "..\\Pack\\Hash\\Body.buf").is_none());
        assert!(files.resolve("Pack", "C:\\Hash\\Body.buf").is_none());
    }
}
