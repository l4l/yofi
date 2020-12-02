use sctk::{
    environment::SimpleGlobal,
    reexports::{
        calloop,
        protocols::wlr::unstable::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1,
    },
    WaylandSource,
};

pub use desktop::Entry as DesktopEntry;

mod desktop;
mod draw;
mod input;
mod state;
mod surface;

sctk::default_environment!(Env,
    fields = [
        layer_shell: SimpleGlobal<ZwlrLayerShellV1>,
    ],
    singles = [
        ZwlrLayerShellV1 => layer_shell
    ]
);

fn setup_logger() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("/tmp/yofi.log").unwrap())
        .apply()
        .unwrap();
}

fn main() {
    setup_logger();

    let (env, display, queue) =
        sctk::new_default_environment!(Env, fields = [layer_shell: SimpleGlobal::new()])
            .expect("Initial roundtrip failed!");
    let mut event_loop = calloop::EventLoop::<()>::new().unwrap();

    let mut surface = surface::Surface::new(&env);

    let (_input, key_stream) = input::InputHandler::new(&env, &event_loop);

    WaylandSource::new(queue)
        .quick_insert(event_loop.handle())
        .unwrap();

    let mut state = state::State::from_entries(desktop::find_entries());

    loop {
        let mut should_redraw = false;
        for event in key_stream.try_iter() {
            should_redraw = true;

            if state.process_event(event) {
                return;
            }
        }

        match surface.handle_events() {
            surface::EventStatus::Finished => break,
            surface::EventStatus::ShouldRedraw => should_redraw = true,
            surface::EventStatus::Idle => {}
        };

        if should_redraw {
            use std::iter::once;

            state.process_entries();

            let input_widget = draw::Widget::input_text(&state.input_buf());
            let list_view_widget = draw::Widget::list_view(
                state.processed_entries().map(|e| draw::ListItem {
                    name: e.name.as_str(),
                }),
                state.selected_item(),
            );

            surface.redraw(once(input_widget).chain(once(list_view_widget)));
        }

        display.flush().unwrap();
        event_loop.dispatch(None, &mut ()).unwrap();
    }
}
