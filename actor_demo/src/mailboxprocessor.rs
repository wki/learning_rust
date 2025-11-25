use std::ops::Deref;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

enum Message {
    Ok,
    NoOp
}

type AsyncHandler = fn(Message) -> Box<dyn Future<Output = ()>>;
type XxxHandler = dyn Fn(Message) -> dyn Future<Output=()>;

struct MailboxProcessor {
    mailbox: Sender<Message>,
    processor: JoinHandle<()>,
    actor: Box<dyn Actor>
}

impl MailboxProcessor {
    // FIXME: fn must be async. How to handle?
    fn new(handler: &AsyncHandler) -> Self {
        let (mailbox, mut receiver) = mpsc::channel::<Message>(100);
        let processor = tokio::task::spawn(async move {
            while let Some(message) = receiver.recv().await {
                // let x = handler(message); //.downcast_ref();
                // let x = (*handler) (message); // .await;
                // let x = *handler;
                // let r = x(message).as_ref().await;

                // let x = *handler(message);
                // handler(message).await;
            }
        });
        Self {
            mailbox,
            processor
        }
    }

    fn handle(&mut self, msg: &Message) {
        todo!()
    }

    fn test(&mut self) {
        self.handle(&Message::Ok);
        let x = |m: Message| -> () { self.handle(&m) };
    }
}

trait Actor {
    fn handle(&mut self, msg: &Message) -> ();
}
