use async_std::channel::unbounded;
use async_std::channel::Receiver;
use async_std::channel::Sender;
use async_std::prelude::FutureExt;
use async_std::prelude::StreamExt;
use async_std::task;
use log::debug;
use log::error;
use log::trace;

use qsk_types::control_code::ControlCode;
use qsk_types::layers::InputTransformer;
use crate::events::EventCode;
use crate::events::InputEvent;
use crate::device::traits::InputEventSink;
use crate::device::traits::InputEventSource;

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
            match e.code {
                EventCode::SynCode(_) => trace!("recv: {:?} {:?}", e.code, e.state),
                _ => debug!("recv: {:?} {:?}", e.code, e.state),
            };
            if let Some(e_vec) = self.input_transformer.transform(e) {
                for cc in e_vec.iter() {
                    match cc {
                        ControlCode::InputEvent(v) => {
                            if let Err(e) = s.send(v.clone()).await {
                                error!("error sending: {:?}", e);
                                return;
                            }
                            match e.code {
                                EventCode::SynCode(_) => trace!("recv: {:?} {:?}", v.code, v.state),
                                _ => debug!("send: {:?} {:?}", v.code, v.state),
                            };
                        }
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
        let (input_sender, handler_receiver) = unbounded();
        let (handler_sender, mut output_receiver) = unbounded();

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
                        Ok(a) => match input_sender.send(a).await {
                            Err(async_std::channel::SendError(msg)) => {
                                debug!("channel closed, failed to send: {:?}", msg);
                                break;
                            }
                            _ => (),
                        },
                        Err(err) => {
                            error!("error reading from keyboard device: {:?}", err)
                        }
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
