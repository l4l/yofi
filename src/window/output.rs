use sctk::{
    output::{OutputHandler, OutputState},
    reexports::client::*,
};

use super::Window;

impl OutputHandler for Window {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: protocol::wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: protocol::wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: protocol::wl_output::WlOutput,
    ) {
    }
}
