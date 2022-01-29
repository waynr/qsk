use std::fs::File;
use std::path::PathBuf;
use std::io::LineWriter;

use serde::ser::{
    Serializer, SerializeSeq,
};
use serde_yaml;

use crate::errors::Result;
use crate::layers::{
    InputTransformer, ControlCode,
};
use crate::events::InputEvent;

pub struct Recorder<W>{
    inner: Box<dyn InputTransformer + Send>,
    ser: serde_yaml::Serializer<W>,
    record: Box<dyn FnMut(InputEvent)>,
}

impl Recorder<LineWriter<File>> {
    fn new(p: PathBuf, it: Box<dyn InputTransformer + Send>) -> Result<Self> {
        let file = File::create(p)?;
        let mut file = LineWriter::new(file);

        let mut ser = serde_yaml::Serializer::new(file);
        let mut seq = ser.serialize_seq(None)?;
        let record = |ie| {
                seq.serialize_element(&ie);
        };
        Ok(Recorder{
            inner: it,
            ser,
            record: Box::new(record),
        })
    }
}

impl InputTransformer for Recorder<LineWriter<File>> {
    fn transform(&mut self, e: InputEvent) -> Option<Vec<ControlCode>> {
        self.inner.transform(e)
    }
}

