use sctk::{
    compositor::{CompositorHandler, CompositorState},
    delegate_layer,
    output::{OutputHandler, OutputState},
    reexports::client::{protocol::wl_surface::WlSurface, ConnectionHandle, QueueHandle},
    registry::RegistryState,
    seat::{keyboard::Modifiers, SeatState},
    shell::layer::LayerState,
    shm::{pool::raw::RawPool, ShmHandler, ShmState},
};

use super::{RenderKind, WindowState};

delegate_layer!(WindowState);

impl LayerHandler for WindowState {
    fn layer_state(&mut self) -> &mut LayerState {
        match &mut self.protocols.render_kind {
            RenderKind::Layer { layer_state } => layer_state,
            RenderKind::Xdg { .. } => panic!("unexpected RenderKind"),
        }
    }

    fn closed(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
    ) {
        self.running = false;
    }

    fn configure(
        &mut self,
        conn: &mut ConnectionHandle,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        serial: u32,
    ) {
        if self.initial_configure {
            self.initial_configure = false;
            self.draw(conn, qh)
        }
    }
}
