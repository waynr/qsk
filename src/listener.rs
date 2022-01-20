use log::debug;
use log::trace;

use crate::events::InputEventSource;
use crate::events::EventCode;

use crate::device::linux::Device;

pub struct StdoutListener {
    d: Device,
}

impl StdoutListener {
    pub fn from_device(d: Device) -> Self {
        StdoutListener{ d }
    }

    pub fn listen(&mut self) {
        loop {
            match self.d.recv() {
                Ok(ie) => {
                    match ie.code {
                        EventCode::SynCode(_) => trace!("recv: {:?} {:?}", ie.code, ie.state),
                        _ => debug!("recv: {:?} {:?}", ie.code, ie.state),
                    };
                },
                _ => (),
            }
        }
    }
}
