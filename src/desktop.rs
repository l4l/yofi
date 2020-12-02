use std::fs;
use std::path::PathBuf;

use xdg::BaseDirectories;

pub struct Entry {
    pub name: String,
    pub exec: String,
}

pub fn find_entries() -> Vec<Entry> {
    let xdg = BaseDirectories::new().unwrap();

    let mut dirs = xdg.get_data_dirs();
    dirs.push(xdg.get_data_home());
    traverse_dirs(dirs)
}

fn traverse_dirs(paths: impl IntoIterator<Item = PathBuf>) -> Vec<Entry> {
    let mut entries = vec![];

    for path in paths.into_iter() {
        let apps_dir = path.join("applications");

        for dir_entry in fs::read_dir(&apps_dir)
            .map_err(|e| eprintln!("cannot read {:?} folder: {}, skipping", apps_dir, e))
            .into_iter()
            .flatten()
            .filter_map(|e| {
                if let Err(err) = &e {
                    eprintln!("failed to read file: {}", err);
                }

                e.ok()
            })
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext.to_str().unwrap() == "desktop")
                    .unwrap_or(false)
            })
        {
            let entry = match fep::parse_entry(dir_entry.path()) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("cannot parse {:?}: {}, skipping", dir_entry, err);
                    continue;
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
                        eprintln!("entry {:?} has no \"Name\" attribute", dir_entry.path());
                    }
                    if e.is_none() {
                        eprintln!("entry {:?} has no \"Exec\" attribute", dir_entry.path());
                    }
                    continue;
                }
            }
        }
    }
    entries
}
