use std::marker::PhantomData;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

// one message type for all actors -- CON: need to handle all cases...
enum Message {
    Ok,
    Failed(String)
}

trait ActorState {}

pub struct ActorData<State: ActorState = Initial> {
    mailbox: Sender<Message>, // TODO: build via macro
    processor: JoinHandle<()>,
    phantom: PhantomData<State>
}

// our 3 actor states
struct Initial {}
struct Running {}
struct Stopped {}

impl ActorState for Initial {}
impl ActorState for Running {}
impl ActorState for Stopped {}

impl ActorData<Initial> {
    fn new() -> ActorData<Running> {
        ActorData:: <Running> {
            mailbox: (),
            processor: (),
            phantom: Default::default(),
        }
    }
}

impl ActorData<Running> {
    async fn run(mut &self) {
        let (mailbox, mut rx) = channel::<Message>(100);
        let processor = tokio::task::spawn(async move {
            while let Some(message) = rx.recv().await {
                self.handle
            }
        });
    }
}

//
// trait Actor {
//     async fn tell(&mut self, message: Message);
//     async fn handle(&mut self, message: Message);
// }
//
// struct SampleActor {
//     mailbox: Sender<Message>, // TODO: build via macro
//     processor: JoinHandle<()>,
//     counter: u16
// }
//
// impl SampleActor { // TODO: fully build via macro
//     fn new() -> Self {
//         let (mailbox, mut rx) = channel::<Message>(100);
//         let processor = tokio::task::spawn(async move {
//             while let Some(message) = rx.recv().await {
//                 todo!(); // handle message
//             }
//         });
//
//         Self {
//             mailbox,
//             processor,
//             counter: 0,
//         }
//     }
// }
//
// impl Actor for SampleActor {
//     async fn tell(&mut self, message: Message) {
//         self.mailbox.send(message).await;
//     }
//
//     async fn handle(&mut self, message: Message) {
//         todo!()
//     }
// }

// trait MailboxProcessor {
//     type Item;
//
//     fn tell(&mut self, message: Self::Item);
//     fn handle(&mut self, message: Self::Item);
// }
//
// struct Mailbox<TMessage> {
//     mailbox: mpsc::Sender<Envelope<TMessage>>,
// }
//
// impl<TMessage> MailboxProcessor for Mailbox<TMessage> {
//     type Item = TMessage;
//
//     fn tell(&mut self, message: Self::Item) {
//
//         todo!()
//     }
//
//     fn handle(&mut self, message: Self::Item) {
//         todo!()
//     }
// }
//
// impl<TMessage> Mailbox<TMessage> {
//     fn new() -> Self {
//         let (mailbox, mut rx) = channel::<Envelope<TMessage>>(100);
//
//         Self {
//             mailbox,
//         }
//     }
// }
//
// struct Envelope<TMessage> {
//     sender: Box<dyn Actor<Message = TMessage>>,
//     message: TMessage
// }
//
// trait Actor {
//     type Message;
// }


// /// an envelope holds a message plus its sender
// struct Envelope<'mbox, TMessage>  {
//     sender: &'mbox MailboxProcessor<'mbox, TMessage>, // FIXME: actually the sender could have another message type
//     message: TMessage,
// }

// /// a mailbox Processor processes messages in turn
// struct MailboxProcessor<'mbox, TMessage> {
//     mailbox: mpsc::Sender<Envelope<'mbox, TMessage>>,
//     processor: JoinHandle<()>
// }
//
// impl<'mbox, TMessage> MailboxProcessor<'mbox, TMessage> {
//     fn tell(&'mbox mut self, sender: &'mbox MailboxProcessor<'mbox, TMessage>, message: TMessage) {
//         let envelope = Envelope {
//             sender,
//             message
//         };
//         self.mailbox.send(envelope);
//     }
//
//     async fn handle(&self, message: TMessage) {
//         todo!()
//     }
// }

// trait MailboxProcessor<TMessage> {
//     fn tell(&mut self, message: TMessage);
//     async fn handle(&self, message: TMessage);
// }

// impl<TMessage> dyn MailboxProcessor<TMessage> {
//     fn tell(&self, message: TMessage) {
//         todo!()
//     }
//     fn handle(&self, message: TMessage) {
//         todo!()
//     }
// }

// unsafe impl<TMessage> Send for Envelope<TMessage> {}
// unsafe impl<TMessage> Sync for Envelope<TMessage> {}

// struct Actor<'mbox, TData, TMessage> {
//     data: TData,
//     mailbox: Sender<Envelope<'mbox, TMessage>>,
//     processor: JoinHandle<()>
// }
//
//
// impl<TData, TMessage: 'static> Actor<TData, TMessage> {
//     fn new(data: TData) -> Self {
//         let (mailbox, mut receiver) = mpsc::channel::<Envelope<TMessage>>(100);
//
//         let processor = tokio::task::spawn(async move {
//             while let Some(message) = receiver.recv().await {
//                 message.receiver.handle(message.message).await
//             }
//         });
//
//         let mut me = Self {
//             data,
//             mailbox,
//             processor
//         };
//
//         me
//     }
// }
