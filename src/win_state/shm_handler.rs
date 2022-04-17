use sctk::{
    delegate_shm,
    shm::{ShmHandler, ShmState},
};

use super::WindowState;

delegate_shm!(WindowState);

impl ShmHandler for WindowState {
    fn shm_state(&mut self) -> &mut ShmState {
        &mut self.protocols.shm_state
    }
}
