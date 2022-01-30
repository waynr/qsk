use std::fs::File;
use std::path::PathBuf;
use std::io::LineWriter;

use async_std::prelude::StreamExt;
use async_std::sync::channel;
use async_std::sync::Receiver;
use async_std::sync::Sender;
use async_std::task::block_on;

use serde::ser::{
    Serializer, SerializeSeq,
};
use serde_yaml;

use crate::layers::{
    InputTransformer, ControlCode,
};
use crate::events::InputEvent;

pub struct Recorder{
    receiver: Receiver<ControlCode>,
}

impl Recorder {
    pub fn wrap(it: Box<dyn InputTransformer + Send>) -> (Self, Listener) {
        let (sender, receiver) = channel(1);
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
    fn send_ie(&mut self, ie: &InputEvent) {
        block_on(self.sender.send(ControlCode::InputEvent(ie.clone())))
    }

    fn send_cc(&mut self, cc: &ControlCode) {
        block_on(self.sender.send(cc.clone()))
    }
}

impl InputTransformer for Listener {
    fn transform(&mut self, ie: InputEvent) -> Option<Vec<ControlCode>> {
        self.send_ie(&ie);
        if let Some(vcc) = self.inner.transform(ie) {
            for cc in vcc.iter() {
                self.send_cc(cc);
            }
            return Some(vcc);
        }
        None
    }
}
