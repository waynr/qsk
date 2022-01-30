use std::fs::File;
use std::path::PathBuf;
use std::io::LineWriter;

use async_std::prelude::StreamExt;
use async_std::channel::bounded;
use async_std::channel::Receiver;
use async_std::channel::Sender;
use async_std::task::block_on;

use serde::{
    Serialize, Deserialize,
    ser::{
        Serializer, SerializeSeq,
    },
};
use serde_yaml;
use log::error;

use crate::layers::{
    InputTransformer, ControlCode,
};
use crate::events::InputEvent;
use crate::errors::Result;

#[derive(Serialize, Deserialize, Debug)]
pub enum Log {
    In(InputEvent),
    Out(ControlCode),
}

pub struct Recorder{
    receiver: Receiver<Log>,
}

impl Recorder {
    pub fn wrap(it: Box<dyn InputTransformer + Send>) -> (Self, Listener) {
        let (sender, receiver) = bounded(1);
        (Recorder{
            receiver,
        }, Listener {
            inner: it,
            sender,
        })
    }

    pub async fn record(&mut self, p: PathBuf) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let file = File::create(p)?;
        let file = LineWriter::new(file);

        let mut ser = serde_yaml::Serializer::new(file);
        let mut seq = ser.serialize_seq(None)?;
        while let Some(ie) = self.receiver.next().await {
            seq.serialize_element(&ie)?;
        }
        seq.end()?;
        Ok(())
    }

}

pub struct Listener{
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
                if let Err(e) = self.send(Log::Out(*cc)) {
                    error!("error sending: {:?}", e);
                }
            }
            return Some(vcc);
        }
        None
    }
}
