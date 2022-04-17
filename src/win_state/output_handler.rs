use sctk::{
    delegate_output,
    output::{OutputHandler, OutputState},
    reexports::client::{protocol::wl_output, ConnectionHandle, QueueHandle},
};

use super::WindowState;

delegate_output!(WindowState);

impl OutputHandler for WindowState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.protocols.output_state
    }

    fn new_output(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &mut ConnectionHandle,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}
