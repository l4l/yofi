use sctk::{
    environment::SimpleGlobal,
    reexports::{
        calloop,
        protocols::wlr::unstable::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1,
    },
    WaylandSource,
};

mod draw;
mod input;
mod surface;

use surface::EventStatus;

sctk::default_environment!(Env,
    fields = [
        layer_shell: SimpleGlobal<ZwlrLayerShellV1>,
    ],
    singles = [
        ZwlrLayerShellV1 => layer_shell
    ]
);

pub struct Entry {
    pub name: String,
    pub exec: String,
}

fn traverse_dirs(paths: impl IntoIterator<Item = std::path::PathBuf>) -> Vec<Entry> {
    let mut entries = vec![];

    for path in paths.into_iter() {
        let apps_dir = path.join("applications");

        for dir_entry in std::fs::read_dir(&apps_dir)
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

fn main() {
    let (env, display, queue) =
        sctk::new_default_environment!(Env, fields = [layer_shell: SimpleGlobal::new()])
            .expect("Initial roundtrip failed!");
    let mut event_loop = calloop::EventLoop::<()>::new().unwrap();

    let mut surface = surface::Surface::new(&env);

    let (_input, key_stream) = input::InputHandler::new(&env, &event_loop);

    WaylandSource::new(queue)
        .quick_insert(event_loop.handle())
        .unwrap();

    let xdg = xdg::BaseDirectories::new().unwrap();

    let mut dirs = xdg.get_data_dirs();
    dirs.push(xdg.get_data_home());
    let desktop_entries = traverse_dirs(dirs);

    let mut input = String::new();
    let mut selected = 0usize;
    let mut processed_entries: Vec<fuse_rust::SearchResult> = vec![];

    loop {
        let mut should_redraw = false;
        for event in key_stream.try_iter() {
            should_redraw = true;

            use input::KeyPress;
            use sctk::seat::keyboard::keysyms;
            match event {
                KeyPress {
                    keysym: keysyms::XKB_KEY_Escape,
                    ..
                } => return,
                KeyPress {
                    keysym: keysyms::XKB_KEY_BackSpace,
                    ..
                } => {
                    input.pop();
                }
                KeyPress {
                    keysym: keysyms::XKB_KEY_Up,
                    ..
                } => selected = selected.saturating_sub(1),
                KeyPress {
                    keysym: keysyms::XKB_KEY_Down,
                    ..
                } => selected = desktop_entries.len().min(selected + 1),
                KeyPress {
                    keysym: keysyms::XKB_KEY_Return,
                    ..
                } => {
                    if selected >= processed_entries.len() {
                        continue;
                    }
                    let entry = &desktop_entries[processed_entries[selected].index];
                    let args = shlex::split(&entry.exec)
                        .unwrap()
                        .into_iter()
                        .map(|s| std::ffi::CString::new(s).unwrap())
                        .collect::<Vec<_>>();
                    nix::unistd::execvp(&args[0], &args[1..]).unwrap();
                }
                KeyPress {
                    keysym: keysyms::XKB_KEY_bracketright,
                    ctrl: true,
                    ..
                } => input.clear(),
                KeyPress { sym: Some(sym), .. } if !sym.is_control() && !event.ctrl => {
                    input.push(sym)
                }
                _ => {
                    println!("unhandled sym: {:?} (ctrl: {})", event.sym, event.ctrl);
                }
            }
        }

        match surface.handle_events() {
            EventStatus::Finished => break,
            EventStatus::ShouldRedraw => should_redraw = true,
            EventStatus::Idle => {}
        };

        if should_redraw {
            use std::iter::once;

            let fuse = fuse_rust::Fuse::default();
            processed_entries = fuse
                .search_text_in_iterable(&input, desktop_entries.iter().map(|e| e.name.as_str()));

            let widgets = once(draw::Widget::input_text(&input))
                //
                .chain(once(draw::Widget::list_view(
                    processed_entries.iter().map(|r| &desktop_entries[r.index]),
                    // desktop_entries.iter(),
                    selected,
                )));
            surface.redraw(widgets.into_iter());
        }

        display.flush().unwrap();
        event_loop.dispatch(None, &mut ()).unwrap();
    }
}
