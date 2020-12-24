use async_std::prelude::FutureExt;
use async_std::prelude::StreamExt;
use async_std::sync::channel;
use async_std::sync::Receiver;
use async_std::sync::Sender;
use async_std::task;
use log::debug;
use log::error;

use qsk_events::KeyboardEvent;
use qsk_events::KeyboardEventSink;
use qsk_events::KeyboardEventSource;
use qsk_layers::ControlCode;
use qsk_layers::InputTransformer;

pub struct QSKEngine {
    input_transformer: Box<dyn InputTransformer + Send>,
}

impl QSKEngine {
    pub fn new(it: Box<dyn InputTransformer + Send>) -> Self {
        QSKEngine {
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

    pub async fn run(
        self,
        src: Box<dyn KeyboardEventSource>,
        snk: Box<dyn KeyboardEventSink>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let (input_sender, handler_receiver) = channel(1);
        let (handler_sender, mut output_receiver) = channel(1);

        debug!("creating handler task");
        let handler_task = task::Builder::new()
            .name("handler".to_string())
            .spawn(self.handle(handler_receiver, handler_sender))?;

        debug!("creating input task");
        let input_task = task::Builder::new()
            .name("input".to_string())
            .spawn(async move {
                loop {
                    let t = src.recv();
                    debug!("received KeyboardEvent from keyboard");
                    match t {
                        Ok(a) => input_sender.send(a).await,
                        Err(err) => error!("error reading from keyboard device: {:?}", err),
                    }
                    debug!("sent KeyboardEvent to handler");
                }
            })?;

        debug!("creating output task");
        let output_task = task::Builder::new()
            .name("output".to_string())
            .spawn(async move {
                while let Some(e) = output_receiver.next().await {
                    debug!("received KeyboardEvent from handler");
                    match snk.send(e) {
                        Ok(_) => (),
                        Err(err) => error!("error writing to keyboard device: {:?}", err),
                    }
                    debug!("sent InputEvent to virtual keyboard");
                }
            })?;

        input_task.race(output_task).race(handler_task).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
