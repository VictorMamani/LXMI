//! Optional mod insight (kind, character) and cached model previews.
//! Previews live beside the packages, keyed by the immutable mod id, so a
//! rendered preview never goes stale. Nothing here touches a game or runtime.
use super::{decode, error, temp_name, validate_game, validate_id, ModLibrary, ModManifest};
use crate::{
    filesystem::{missing, Directory},
    ErrorCode, Result,
};
use lxmi_mod_preview::{ModFile, ModInsight, ModSource, PreviewOptions, PreviewView};
use serde::Serialize;

/// Bump when the renderer output changes; older previews are then ignored.
const PREVIEWS: &str = "previews-v1";
const UNAVAILABLE: &str = "unavailable";
const MAX_PREVIEW_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewState {
    /// Not generated yet.
    Missing,
    Ready,
    /// Generated, but no drawable model was found.
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModDetails {
    pub mod_id: String,
    #[serde(flatten)]
    pub insight: ModInsight,
    pub preview: PreviewState,
}

/// Reads the verified payload through descriptor-anchored paths.
struct Payload {
    dir: Directory,
    files: Vec<ModFile>,
}

impl ModSource for Payload {
    fn files(&self) -> Vec<ModFile> {
        self.files.clone()
    }
    fn read(&self, path: &str, limit: u64) -> Option<Vec<u8>> {
        self.dir.read_file(path, limit).ok()
    }
}

impl ModLibrary {
    fn payload(&self, dir: &Directory, manifest: &ModManifest) -> Result<Payload> {
        Ok(Payload {
            dir: dir.descend(&format!("packages/{}/payload", manifest.id))?,
            files: manifest
                .files
                .iter()
                .map(|f| ModFile {
                    path: f.relative_path.clone(),
                    size: f.size,
                })
                .collect(),
        })
    }

    fn preview_state(dir: &Directory, id: &str) -> Result<PreviewState> {
        let preview = match dir.descend(&format!("{PREVIEWS}/{id}")) {
            Ok(preview) => preview,
            Err(e) if missing(&e) => return Ok(PreviewState::Missing),
            Err(e) => return Err(e),
        };
        if preview.metadata(PreviewView::Front.file_name())?.is_some() {
            Ok(PreviewState::Ready)
        } else if preview.metadata(UNAVAILABLE)?.is_some() {
            Ok(PreviewState::Unavailable)
        } else {
            Ok(PreviewState::Missing)
        }
    }

    /// Kind and character of every mod. Reads only manifests and INI files;
    /// mods that fail to read are left to `list`, which reports them.
    pub fn mod_details(&self, game: &str) -> Result<Vec<ModDetails>> {
        let dir = match self.existing(game) {
            Ok(dir) => dir,
            Err(e) if missing(&e) => return Ok(vec![]),
            Err(e) => return Err(e),
        };
        let mut out = vec![];
        for id in dir.child("packages")?.entries(super::MAX_MODS)? {
            let details = (|| {
                validate_id(&id)?;
                let manifest: ModManifest = decode(
                    &dir.read_file(&format!("packages/{id}/lxmi-mod.json"), super::MAX_METADATA)?,
                )?;
                if manifest.id != id || manifest.game_id != game {
                    return Err(error(ErrorCode::InvalidMetadata, "Invalid mod manifest."));
                }
                let payload = self.payload(&dir, &manifest)?;
                Ok(ModDetails {
                    insight: lxmi_mod_preview::inspect(&manifest.name, &payload),
                    preview: Self::preview_state(&dir, &id)?,
                    mod_id: id.clone(),
                })
            })();
            if let Ok(details) = details {
                out.push(details);
            }
        }
        Ok(out)
    }

    /// Renders and caches the previews of one verified mod. Slow (seconds);
    /// does not hold the library lock, so profiles stay editable meanwhile.
    pub fn generate_mod_preview(&self, game: &str, id: &str) -> Result<ModDetails> {
        validate_game(game)?;
        validate_id(id)?;
        let dir = self.initialize(game)?;
        let manifest = self.verify_in(&dir, game, id)?;
        let payload = self.payload(&dir, &manifest)?;
        let insight = lxmi_mod_preview::inspect(&manifest.name, &payload);
        let details = |preview| ModDetails {
            mod_id: id.into(),
            insight: insight.clone(),
            preview,
        };
        let previews = dir.private_child(PREVIEWS)?;
        if previews.metadata(id)?.is_some() {
            return Ok(details(Self::preview_state(&dir, id)?));
        }
        let rendered = if insight.previewable {
            lxmi_mod_preview::render_previews(&payload, PreviewOptions::default())
        } else {
            vec![]
        };

        let staging = dir.child("staging")?;
        let stage_name = temp_name();
        let stage = staging.new_private_child(&stage_name)?;
        let result = (|| {
            for preview in &rendered {
                stage.write_new(preview.view.file_name(), &preview.png)?;
            }
            if rendered.is_empty() {
                stage.write_new(UNAVAILABLE, b"")?;
            }
            stage.sync()?;
            match staging.promote(&stage_name, &previews, id) {
                Ok(()) => {}
                // Another window rendered the same mod first; keep its result.
                Err(_) if previews.metadata(id)?.is_some() => {}
                Err(e) => return Err(e),
            }
            Ok(details(Self::preview_state(&dir, id)?))
        })();
        if staging.metadata(&stage_name)?.is_some() {
            staging.remove_stage(&stage_name)?;
        }
        result
    }

    /// PNG bytes of a generated preview.
    pub fn mod_preview(&self, game: &str, id: &str, view: PreviewView) -> Result<Vec<u8>> {
        validate_id(id)?;
        let dir = self.existing(game)?;
        dir.read_file(
            &format!("{PREVIEWS}/{id}/{}", view.file_name()),
            MAX_PREVIEW_BYTES,
        )
        .map_err(|e| {
            if missing(&e) {
                error(ErrorCode::NotFound, "This mod has no preview yet.")
            } else {
                e
            }
        })
    }

    /// Removes every cached preview of the game, including older renderer
    /// versions. Mods and profiles are untouched.
    pub fn clear_mod_previews(&self, game: &str) -> Result<()> {
        let dir = match self.existing(game) {
            Ok(dir) => dir,
            Err(e) if missing(&e) => return Ok(()),
            Err(e) => return Err(e),
        };
        for name in dir.entries(64)? {
            if name.starts_with("previews-v") && dir.metadata(&name)?.is_some_and(|m| m.is_dir()) {
                dir.remove_stage(&name)?;
            }
        }
        dir.sync()
    }
}
