use sctk::shm::{Shm, ShmHandler};

use super::Window;

impl ShmHandler for Window {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}
