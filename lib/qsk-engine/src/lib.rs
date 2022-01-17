use async_std::prelude::FutureExt;
use async_std::prelude::StreamExt;
use async_std::sync::channel;
use async_std::sync::Receiver;
use async_std::sync::Sender;
use async_std::task;
use log::debug;
use log::trace;
use log::error;

use qsk_events::InputEvent;
use qsk_events::InputEventSink;
use qsk_events::InputEventSource;
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

    pub async fn handle(mut self, mut r: Receiver<InputEvent>, s: Sender<InputEvent>) {
        while let Some(e) = r.next().await {
            debug!("recv: {:?} {:?}", e.code, e.state);
            if let Some(e_vec) = self.input_transformer.transform(e) {
                for cc in e_vec.iter() {
                    match cc {
                        ControlCode::InputEvent(v) => {
                            s.send(v.clone()).await;
                            debug!("send: {:?} {:?}", v.code, v.state);
                        },
                        ControlCode::Exit => return,
                        _ => continue,
                    }
                }
            }
        }
    }

    pub async fn run(
        self,
        mut src: Box<dyn InputEventSource>,
        mut snk: Box<dyn InputEventSink>,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let (input_sender, handler_receiver) = channel(1);
        let (handler_sender, mut output_receiver) = channel(1);

        trace!("creating handler task");
        let handler_task = task::Builder::new()
            .name("handler".to_string())
            .spawn(self.handle(handler_receiver, handler_sender))?;

        trace!("creating input task");
        let input_task = task::Builder::new()
            .name("input".to_string())
            .spawn(async move {
                loop {
                    let t = src.recv();
                    trace!("received InputEvent from keyboard");
                    match t {
                        Ok(Some(a)) => input_sender.send(a).await,
                        Ok(None) => (),
                        Err(err) => error!("error reading from keyboard device: {:?}", err),
                    }
                    trace!("sent InputEvent to handler");
                }
            })?;

        trace!("creating output task");
        let output_task = task::Builder::new()
            .name("output".to_string())
            .spawn(async move {
                while let Some(e) = output_receiver.next().await {
                    trace!("received InputEvent from handler");
                    match snk.send(e) {
                        Ok(_) => (),
                        Err(err) => error!("error writing to keyboard device: {:?}", err),
                    }
                    trace!("sent InputEvent to virtual keyboard");
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
