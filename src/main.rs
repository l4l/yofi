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
mod state;
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

    let mut state = state::State::from_entries(desktop_entries);

    loop {
        let mut should_redraw = false;
        for event in key_stream.try_iter() {
            should_redraw = true;

            if state.process_event(event) {
                return;
            }
        }

        match surface.handle_events() {
            EventStatus::Finished => break,
            EventStatus::ShouldRedraw => should_redraw = true,
            EventStatus::Idle => {}
        };

        if should_redraw {
            use std::iter::once;

            state.process_entries();

            let input_widget = draw::Widget::input_text(&state.input_buf());
            let list_view_widget =
                draw::Widget::list_view(state.processed_entries(), state.selected_item());

            surface.redraw(once(input_widget).chain(once(list_view_widget)));
        }

        display.flush().unwrap();
        event_loop.dispatch(None, &mut ()).unwrap();
    }
}
