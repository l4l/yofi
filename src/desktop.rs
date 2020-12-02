use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use xdg::BaseDirectories;

pub struct Entry {
    pub name: String,
    pub exec: String,
}

pub fn find_entries() -> Vec<Entry> {
    let xdg = BaseDirectories::new().unwrap();

    let mut dirs = xdg.get_data_dirs();
    dirs.push(xdg.get_data_home());
    let mut entries = vec![];
    traverse_dirs(&mut entries, dirs);
    entries.sort_unstable_by(|x, y| x.name.cmp(&y.name));
    entries.dedup_by(|x, y| x.name == y.name);
    entries
}

fn read_dir(path: &Path) -> impl Iterator<Item = DirEntry> {
    fs::read_dir(&path)
        .map_err(|e| eprintln!("cannot read {:?} folder: {}, skipping", path, e))
        .into_iter()
        .flatten()
        .filter_map(|e| {
            if let Err(err) = &e {
                eprintln!("failed to read file: {}", err);
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
        Err(err) => eprintln!("failed to get `{:?}` file type: {}", dir_entry_path, err),
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
            eprintln!("cannot parse {:?}: {}, skipping", dir_entry, err);
            return;
        }
    };
    let main_section = entry.section("Desktop Entry");
    match (main_section.attr("Name"), main_section.attr("Exec")) {
        (Some(n), Some(e)) => {
            entries.push(Entry {
                name: n.to_owned(),
                exec: e.to_owned(),
            });
        }
        (n, e) => {
            if n.is_none() {
                eprintln!("entry {:?} has no \"Name\" attribute", dir_entry_path);
            }
            if e.is_none() {
                eprintln!("entry {:?} has no \"Exec\" attribute", dir_entry_path);
            }
        }
    }
}
