use std::ffi::OsStr;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use freedesktop_icon_lookup::{Cache, LookupParam};
use once_cell::sync::OnceCell;
use xdg::BaseDirectories;

use crate::icon::Icon;

mod locale;

pub static XDG_DIRS: OnceCell<BaseDirectories> = OnceCell::new();

pub struct ExecEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<Icon>,
}

pub struct Entry {
    pub entry: ExecEntry,
    pub actions: Vec<ExecEntry>,
    pub desktop_fname: String,
    pub path: PathBuf,
    pub name_with_keywords: String,
    pub is_terminal: bool,
}

impl Entry {
    pub fn subname(&self, action: usize) -> Option<&str> {
        self.actions
            .get(action.checked_sub(1)?)
            .map(|a| a.name.as_ref())
    }

    pub fn icon(&self, action: usize) -> Option<&Icon> {
        if action == 0 {
            self.entry.icon.as_ref()
        } else {
            self.actions[action - 1]
                .icon
                .as_ref()
                .or_else(|| self.entry.icon.as_ref())
        }
    }
}

pub fn xdg_dirs<'a>() -> &'a BaseDirectories {
    XDG_DIRS.get_or_init(|| BaseDirectories::new().expect("failed to get xdg dirs"))
}

pub struct IconConfig {
    pub icon_size: u16,
    pub theme: Option<String>,
}

pub struct Traverser<F> {
    icon_config: Option<(IconConfig, Cache)>,
    filter: F,
}

impl<F> Traverser<F> {
    pub fn new(icon_config: Option<IconConfig>, filter: F) -> anyhow::Result<Self> {
        Ok(Self {
            icon_config: icon_config
                .map(|icon_config| -> anyhow::Result<_> {
                    let mut lookup = Cache::new()?;
                    if let Some(theme) = &icon_config.theme {
                        lookup.load(theme)
                    } else {
                        lookup.load_default()
                    }?;
                    Ok((icon_config, lookup))
                })
                .transpose()?,
            filter,
        })
    }

    fn find_icon(&self, name: &str) -> Option<Icon> {
        let (config, lookup) = self.icon_config.as_ref()?;

        let icon_path = Path::new(name);
        let path: Option<PathBuf>;

        let path = if icon_path.is_absolute() {
            Some(icon_path)
        } else {
            let lookup_icon = |name: &str, icon_size: u16| {
                lookup.lookup_param(
                    LookupParam::new(name)
                        .with_size(icon_size)
                        .with_theme(config.theme.as_deref()),
                )
            };

            path = lookup_icon(name, config.icon_size)
                .or_else(|| lookup_icon(name, config.icon_size + 8))
                .or_else(|| lookup_icon(name, config.icon_size + 16))
                .or_else(|| lookup_icon(name, 512))
                .or_else(|| lookup_icon(name, config.icon_size - 8))
                .or_else(|| {
                    let name = format!("{name}-symbolic");
                    lookup_icon(&name, config.icon_size)
                });

            path.as_deref()
        };

        path.map(Icon::new)
    }

    fn parse_entry(&self, dir_entry: &DirEntry, dir_entry_path: PathBuf) -> Option<Entry> {
        let entry = match fep::parse_entry(&dir_entry_path) {
            Ok(e) => e,
            Err(err) => {
                log::warn!("cannot parse {:?}: {}, skipping", dir_entry, err);
                return None;
            }
        };

        let main_section = entry.section("Desktop Entry");
        let locale = locale::Locale::current();

        if main_section.attr("NoDisplay") == Some("true") {
            log::trace!("Skipping NoDisplay entry {:?}", dir_entry);
            return None;
        }

        let localized_entry = |attr_name: &str| {
            locale
                .keys()
                .filter_map(|key| main_section.attr_with_param(attr_name, key))
                .next()
                .or_else(|| main_section.attr(attr_name))
        };

        match (localized_entry("Name"), main_section.attr("Exec")) {
            (Some(n), Some(e)) => {
                let filename = dir_entry_path.file_name().unwrap();
                let desktop_fname = if let Some(f) = filename.to_str() {
                    f.to_owned()
                } else {
                    log::error!("found non-UTF8 desktop file: {:?}, skipping", filename);
                    return None;
                };

                let actions = entry
                    .sections()
                    .filter_map(|s| {
                        if !s.name().starts_with("Desktop Action ") {
                            return None;
                        }
                        let name = s.attr("Name")?.to_owned();
                        let exec = s.attr("Exec")?.to_owned();
                        Some(ExecEntry {
                            name,
                            exec,
                            icon: localized_entry("Icon").and_then(|name| self.find_icon(name)),
                        })
                    })
                    .collect::<Vec<_>>();

                let entry = ExecEntry {
                    name: n.to_owned(),
                    exec: e.to_owned(),
                    icon: localized_entry("Icon").and_then(|name| self.find_icon(name)),
                };

                return Some(Entry {
                    entry,
                    actions,
                    desktop_fname,
                    path: dir_entry_path,
                    name_with_keywords: n.to_owned()
                        + localized_entry("Keywords").unwrap_or_default(),
                    is_terminal: main_section
                        .attr("Terminal")
                        .map(|s| s == "true")
                        .unwrap_or(false),
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
                None
            }
        }
    }
}

impl<F> Traverser<F>
where
    F: Fn(&OsStr) -> bool,
{
    pub fn find_entries(&self) -> Vec<Entry> {
        let xdg_dirs = xdg_dirs();

        let dirs = std::iter::once(xdg_dirs.get_data_home());
        let dirs = dirs.chain(xdg_dirs.get_data_dirs());
        let mut entries = self.traverse_dirs(dirs);
        entries.sort_by(|x, y| x.entry.name.cmp(&y.entry.name));
        entries.dedup_by(|x, y| x.entry.name == y.entry.name);
        entries
    }

    fn traverse_dirs(&self, paths: impl IntoIterator<Item = PathBuf>) -> Vec<Entry> {
        let mut entries = vec![];
        for path in paths.into_iter() {
            let apps_dir = path.join("applications");
            if !apps_dir.exists() {
                continue;
            }

            for dir_entry in read_dir(&apps_dir).filter(|e| (self.filter)(&e.file_name())) {
                self.traverse_dir_entry(&mut entries, dir_entry);
            }
        }
        entries
    }

    fn traverse_dir_entry(&self, entries: &mut Vec<Entry>, dir_entry: DirEntry) {
        let dir_entry_path = dir_entry.path();

        if dir_entry_path.extension().and_then(|s| s.to_str()) != Some("desktop") {
            return;
        }

        match dir_entry.file_type() {
            Err(err) => log::warn!("failed to get `{:?}` file type: {}", dir_entry_path, err),
            Ok(tp) if tp.is_dir() => {
                for dir_entry in read_dir(&dir_entry_path).filter(|e| (self.filter)(&e.file_name()))
                {
                    self.traverse_dir_entry(entries, dir_entry);
                }

                return;
            }
            _ => {}
        }

        if let Some(entry) = self.parse_entry(&dir_entry, dir_entry_path) {
            entries.push(entry);
        }
    }
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
