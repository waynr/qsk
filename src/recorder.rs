use std::fs::File;
use std::io::LineWriter;
use std::path::PathBuf;

use async_std::channel::unbounded;
use async_std::channel::Receiver;
use async_std::channel::Sender;
use async_std::prelude::StreamExt;
use async_std::task::block_on;

use log::error;
use serde::{
    ser::{SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use serde_json;

use qsk_types::control_code::ControlCode;
use qsk_types::layer_composer::InputTransformer;
use crate::errors::Result;
use crate::events::InputEvent;

#[derive(Serialize, Deserialize, Debug)]
pub enum Log {
    In(InputEvent),
    Out(ControlCode),
}

pub struct Recorder {
    receiver: Receiver<Log>,
}

impl Recorder {
    pub fn wrap(it: Box<dyn InputTransformer + Send>) -> (Self, Listener) {
        let (sender, receiver) = unbounded();
        (Recorder { receiver }, Listener { inner: it, sender })
    }

    pub async fn record(
        &mut self,
        p: PathBuf,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let file = File::create(p)?;
        let file = LineWriter::with_capacity(1, file);

        let mut ser = serde_json::Serializer::pretty(file);
        let mut seq = ser.serialize_seq(None)?;
        while let Some(ie) = self.receiver.next().await {
            seq.serialize_element(&ie)?;
        }
        seq.end()?;
        Ok(())
    }
}

pub struct Listener {
    inner: Box<dyn InputTransformer + Send>,
    sender: Sender<Log>,
}

impl Listener {
    fn send(&mut self, le: Log) -> Result<()> {
        block_on(self.sender.send(le))?;
        Ok(())
    }
}

impl InputTransformer for Listener {
    fn transform(&mut self, ie: InputEvent) -> Option<Vec<ControlCode>> {
        if let Err(e) = self.send(Log::In(ie)) {
            error!("error sending: {:?}", e);
        }
        if let Some(vcc) = self.inner.transform(ie) {
            for cc in vcc.iter() {
                if let Err(e) = self.send(Log::Out(cc.clone())) {
                    error!("error sending: {:?}", e);
                }
            }
            return Some(vcc);
        }
        None
    }
}
