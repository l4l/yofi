use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use once_cell::sync::{Lazy, OnceCell};
use xdg::BaseDirectories;

use crate::icon::Icon;

mod locale;

pub static XDG_DIRS: OnceCell<BaseDirectories> = OnceCell::new();

pub struct Entry {
    pub name: String,
    pub desktop_fname: String,
    pub path: PathBuf,
    pub exec: String,
    pub name_with_keywords: String,
    pub is_terminal: bool,
    pub icon: Option<Icon>,
}

pub fn xdg_dirs<'a>() -> &'a BaseDirectories {
    XDG_DIRS.get_or_init(|| BaseDirectories::new().expect("failed to get xdg dirs"))
}

pub fn find_entries() -> Vec<Entry> {
    let xdg_dirs = xdg_dirs();

    let mut dirs = xdg_dirs.get_data_dirs();
    dirs.push(xdg_dirs.get_data_home());
    let mut entries = vec![];
    traverse_dirs(&mut entries, dirs);
    entries.sort_unstable_by(|x, y| x.name.cmp(&y.name));
    entries.dedup_by(|x, y| x.name == y.name);
    entries
}

fn read_dir(path: &Path) -> impl Iterator<Item = DirEntry> {
    fs::read_dir(&path)
        .map_err(|e| log::debug!("cannot read {:?} folder: {}, skipping", path, e))
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
        if !apps_dir.exists() {
            continue;
        }

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
    let locale = locale::Locale::current();

    let localized_entry = |attr_name: &str| {
        locale
            .keys()
            .filter_map(|key| main_section.attr_with_param(attr_name, key))
            .next()
            .or_else(|| main_section.attr(attr_name))
    };

    match (localized_entry("Name"), main_section.attr("Exec")) {
        (Some(n), Some(e)) => {
            entries.push(Entry {
                name: n.to_owned(),
                desktop_fname: dir_entry_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .expect("desktop file name is not in utf-8")
                    .to_owned(),
                path: dir_entry_path,
                exec: e.to_owned(),
                name_with_keywords: n.to_owned() + localized_entry("Keywords").unwrap_or_default(),
                is_terminal: main_section
                    .attr("Terminal")
                    .map(|s| s == "true")
                    .unwrap_or(false),
                icon: localized_entry("Icon").and_then(|name| {
                    let icon_path = Path::new(name);

                    if icon_path.is_absolute() {
                        Icon::load_icon(icon_path)
                    } else {
                        icon_paths()
                            .and_then(|p| p.get(name))
                            .and_then(|icons| icons.iter().filter_map(Icon::load_icon).next())
                    }
                }),
            });
        }
        (n, e) => {
            if n.is_none() && e.is_none() {
                log::debug!(
                    r#"entry {:?} has no "Name" nor "Exec" attribute"#,
                    dir_entry_path
                );
            } else if n.is_none() {
                log::debug!(r#"entry {:?} has no "Name" attribute"#, dir_entry_path);
            } else if e.is_none() {
                log::debug!(r#"entry {:?} has no "Exec" attribute"#, dir_entry_path);
            }
        }
    }
}

const FALLBACK_THEME: &str = "hicolor";

pub static DEFAULT_THEME: Lazy<String> = Lazy::new(|| {
    let path = "/usr/share/icons/default/index.theme";

    fep::parse_entry(PathBuf::from(path))
        .map_err(|e| log::warn!("failed to parse index theme ({}): {}", path, e))
        .ok()
        .and_then(|entry| {
            entry
                .section("Icon Theme")
                .attr("Inherits")
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| FALLBACK_THEME.to_string())
});

pub struct IconConfig {
    pub icon_size: u16,
    pub theme: String,
}

use std::collections::HashMap;

type IconPaths = HashMap<String, Vec<PathBuf>>;
static ICON_PATHS: OnceCell<IconPaths> = OnceCell::new();

pub fn find_icon_paths(config: IconConfig) -> Result<(), ()> {
    ICON_PATHS.set(traverse_icon_dirs(config)).map_err(|_| ())
}

pub fn icon_paths<'a>() -> Option<&'a IconPaths> {
    ICON_PATHS.get()
}

fn traverse_icon_dirs(config: IconConfig) -> IconPaths {
    let mut icons = IconPaths::new();

    fn traverse_dir(mut icons: &mut IconPaths, theme: &str, icon_size: u16) {
        for dir in xdg_dirs().get_data_dirs() {
            let theme_dir = dir.join("icons").join(&theme);

            let base_path = theme_dir.join(format!("{0}x{0}", icon_size));
            if base_path.exists() {
                for entry in read_dir(&base_path) {
                    traverse_icon_dir(&mut icons, entry);
                }
            }

            let base_path = theme_dir.join("scalable");
            if base_path.exists() {
                for entry in read_dir(&base_path) {
                    traverse_icon_dir(&mut icons, entry);
                }
            }
        }
    }

    traverse_dir(&mut icons, &config.theme, config.icon_size);
    if config.theme != FALLBACK_THEME {
        traverse_dir(&mut icons, FALLBACK_THEME, config.icon_size);
    }

    let pixmap_dir = Path::new("/usr/share/pixmaps/");
    if pixmap_dir.exists() {
        for entry in read_dir(pixmap_dir) {
            traverse_icon_dir(&mut icons, entry);
        }
    }

    icons
}

fn traverse_icon_dir(mut icons: &mut IconPaths, entry: DirEntry) {
    let entry_path = entry.path();

    match entry.file_type() {
        Err(err) => log::warn!("failed to get `{:?}` file type: {}", entry_path, err),
        Ok(tp) if tp.is_dir() => {
            for entry in read_dir(&entry_path) {
                traverse_icon_dir(&mut icons, entry);
            }

            return;
        }
        _ => {}
    }

    match entry_path.extension().and_then(|ext| ext.to_str()) {
        Some("png") | Some("svg") => {
            let icon_name = entry_path
                .file_stem()
                .and_then(|f| f.to_str())
                .expect("failed to get icon name");
            icons
                .entry(icon_name.to_owned())
                .or_default()
                .push(entry_path);
        }
        _ => {}
    }
}
