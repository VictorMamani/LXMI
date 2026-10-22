//! Development tool: classifies and renders every mod in a directory.
//!
//!   cargo run -p lxmi-mod-preview --example render_dir -- <mods-dir> <output-dir>
//!
//! Each subdirectory is one mod; the output gets `<mod>/front.png`,
//! `<mod>/back.png` and `<mod>/insight.json`.
use lxmi_mod_preview::{inspect, render_previews, ModFile, ModSource, PreviewOptions};
use std::path::{Path, PathBuf};
use std::time::Instant;

struct Folder(PathBuf);

impl ModSource for Folder {
    fn files(&self) -> Vec<ModFile> {
        fn walk(root: &Path, dir: &Path, out: &mut Vec<ModFile>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let Ok(meta) = entry.metadata() else { continue };
                if meta.is_dir() {
                    walk(root, &path, out);
                } else if let Ok(relative) = path.strip_prefix(root) {
                    out.push(ModFile {
                        path: relative.to_string_lossy().replace('\\', "/"),
                        size: meta.len(),
                    });
                }
            }
        }
        let mut out = vec![];
        walk(&self.0, &self.0, &mut out);
        out
    }

    fn read(&self, path: &str, limit: u64) -> Option<Vec<u8>> {
        let bytes = std::fs::read(self.0.join(path)).ok()?;
        (bytes.len() as u64 <= limit).then_some(bytes)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [source, output] = args.as_slice() else {
        eprintln!("usage: render_dir <mods-dir> <output-dir>");
        std::process::exit(2);
    };
    let mut mods: Vec<PathBuf> = std::fs::read_dir(source)
        .expect("readable mods directory")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    mods.sort();
    for dir in mods {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let folder = Folder(dir);
        let started = Instant::now();
        let insight = inspect(&name, &folder);
        let previews = if insight.previewable {
            render_previews(&folder, PreviewOptions::default())
        } else {
            vec![]
        };
        let target = Path::new(output).join(&name);
        std::fs::create_dir_all(&target).expect("writable output");
        for preview in &previews {
            std::fs::write(target.join(preview.view.file_name()), &preview.png).unwrap();
        }
        std::fs::write(
            target.join("insight.json"),
            format!(
                "{{\"kind\":\"{:?}\",\"character\":{:?},\"previewable\":{}}}\n",
                insight.kind, insight.character, insight.previewable
            ),
        )
        .unwrap();
        println!(
            "{name}: {:?}, character {:?}, {} previews in {:.2?}",
            insight.kind,
            insight.character,
            previews.len(),
            started.elapsed()
        );
    }
}
