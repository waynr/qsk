use std::fs::File;
use std::path::PathBuf;
use std::io::LineWriter;

use async_std::prelude::StreamExt;
use async_std::channel::unbounded;
use async_std::channel::Receiver;
use async_std::channel::Sender;
use async_std::task::block_on;

use serde::ser::{
    Serializer, SerializeSeq,
};
use serde_yaml;
use log::error;

use crate::layers::{
    InputTransformer, ControlCode,
};
use crate::events::InputEvent;
use crate::errors::Result;

pub struct Recorder{
    receiver: Receiver<ControlCode>,
}

impl Recorder {
    pub fn wrap(it: Box<dyn InputTransformer + Send>) -> (Self, Listener) {
        let (sender, receiver) = unbounded();
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
    sender: Sender<ControlCode>,
}

impl Listener {
    fn send_ie(&mut self, ie: &InputEvent) -> Result<()> {
        block_on(self.sender.send(ControlCode::InputEvent(ie.clone())))?;
        Ok(())
    }

    fn send_cc(&mut self, cc: &ControlCode) -> Result<()> {
        block_on(self.sender.send(cc.clone()))?;
        Ok(())
    }
}

impl InputTransformer for Listener {
    fn transform(&mut self, ie: InputEvent) -> Option<Vec<ControlCode>> {
        if let Err(e) = self.send_ie(&ie) {
            error!("error sending: {:?}", e);
        }
        if let Some(vcc) = self.inner.transform(ie) {
            for cc in vcc.iter() {
                if let Err(e) = self.send_cc(cc) {
                    error!("error sending: {:?}", e);
                }
            }
            return Some(vcc);
        }
        None
    }
}
