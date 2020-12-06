use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use xdg::BaseDirectories;

lazy_static! {
    pub static ref XDG_DIRS: BaseDirectories = BaseDirectories::new().unwrap();
}

pub struct Entry {
    pub name: String,
    pub desktop_fname: String,
    pub exec: String,
    pub is_terminal: bool,
}

pub fn find_entries() -> Vec<Entry> {
    let mut dirs = XDG_DIRS.get_data_dirs();
    dirs.push(XDG_DIRS.get_data_home());
    let mut entries = vec![];
    traverse_dirs(&mut entries, dirs);
    entries.sort_unstable_by(|x, y| x.name.cmp(&y.name));
    entries.dedup_by(|x, y| x.name == y.name);
    entries
}

fn read_dir(path: &Path) -> impl Iterator<Item = DirEntry> {
    fs::read_dir(&path)
        .map_err(|e| log::warn!("cannot read {:?} folder: {}, skipping", path, e))
        .into_iter()
        .flatten()
        .filter_map(|e| {
            if let Err(err) = &e {
                log::warn!("failed to read file: {}", err);
            }

            e.ok()
        })
}

fn traverse_dirs(mut entries: &mut Vec<Entry>, paths: impl IntoIterator<Item = PathBuf>) {
    for path in paths.into_iter() {
        let apps_dir = path.join("applications");

        for dir_entry in read_dir(&apps_dir) {
            traverse_dir_entry(&mut entries, dir_entry);
        }
    }
}

fn traverse_dir_entry(mut entries: &mut Vec<Entry>, dir_entry: DirEntry) {
    let dir_entry_path = dir_entry.path();

    match dir_entry.file_type() {
        Err(err) => log::warn!("failed to get `{:?}` file type: {}", dir_entry_path, err),
        Ok(tp) if tp.is_dir() => {
            for dir_entry in read_dir(&dir_entry_path) {
                traverse_dir_entry(&mut entries, dir_entry);
            }

            return;
        }
        _ => {}
    }

    let entry = match fep::parse_entry(&dir_entry_path) {
        Ok(e) => e,
        Err(err) => {
            log::warn!("cannot parse {:?}: {}, skipping", dir_entry, err);
            return;
        }
    };
    let main_section = entry.section("Desktop Entry");
    match (main_section.attr("Name"), main_section.attr("Exec")) {
        (Some(n), Some(e)) => {
            entries.push(Entry {
                name: n.to_owned(),
                desktop_fname: dir_entry_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .expect("desktop file name is not in utf-8")
                    .to_owned(),
                exec: e.to_owned(),
                is_terminal: main_section
                    .attr("Terminal")
                    .map(|s| s == "true")
                    .unwrap_or(false),
            });
        }
        (n, e) => {
            if n.is_none() {
                log::debug!("entry {:?} has no \"Name\" attribute", dir_entry_path);
            }
            if e.is_none() {
                log::debug!("entry {:?} has no \"Exec\" attribute", dir_entry_path);
            }
        }
    }
}
