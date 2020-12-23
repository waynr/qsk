use async_std::prelude::StreamExt;
use async_std::sync::Receiver;
use async_std::sync::Sender;
use log::debug;

use qsk_events::KeyboardEvent;
use qsk_layers::ControlCode;
use qsk_layers::InputTransformer;

pub struct QSKEngine {
    input_transformer: Box<dyn InputTransformer + Send>,
}

impl QSKEngine {
    pub fn new(it: Box<dyn InputTransformer + Send>) -> Self {
        QSKEngine{
            input_transformer: it,
        }
    }

    pub async fn handle(mut self, mut r: Receiver<KeyboardEvent>, s: Sender<KeyboardEvent>) {
        while let Some(e) = r.next().await {
            debug!("received KeyboardEvent from input task");
            if let Some(e_vec) = self.input_transformer.transform(e) {
                for cc in e_vec.iter() {
                    match cc {
                        ControlCode::KeyboardEvent(v) => s.send(v.clone()).await,
                        ControlCode::Exit => return,
                        _ => continue,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

