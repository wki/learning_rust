use std::borrow::BorrowMut;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot,mpsc};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
enum Request {
    Set(String, String),
    Get(String),
}

#[derive(Debug)]
enum Response {
    Ok(),
    Result(String)
}

/// data transferred over the channel to our service
type RequestTransport = (Request, oneshot::Sender<Response>);

#[tokio::main]
async fn main() {
    println!("Listening on port 8000");
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    // build a channel to a "service"
    let (mut tx, rx) = mpsc::channel::<RequestTransport>(100);
    tokio::spawn(async move {
        service(rx).await;
    });

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            process(socket, &tx_clone.clone().borrow_mut()).await;
        });
    }
}

/// process one client
async fn process(socket: TcpStream, tx: &Sender<RequestTransport>) {
    println!("Connection from {}", socket.peer_addr().unwrap());
    let mut stream = BufStream::new(socket);

    loop {
        let line = &mut String::new();
        let nr_bytes = stream.read_line(line).await.unwrap();
        if nr_bytes == 0 {
            println!("0 bytes read, closing connection");
            stream.write(b"bye\r\n").await.unwrap();
            stream.flush().await.unwrap();
            break;
        } else {
            stream.consume(nr_bytes);
            println!("read {} bytes -> {}", nr_bytes, line);
            stream.write(format!("consumed {} bytes\r\n", nr_bytes).as_bytes()).await.unwrap();
            stream.flush().await.unwrap();

            // send a request through our channel
            let (response_tx, response_rx) = oneshot::channel::<Response>();
            tx.send((Request::Get(line.clone()), response_tx)).await.unwrap();

            // "wait" for the response
            let response = response_rx.await.unwrap();
            stream.write(format!("response: {:?}\r\n", response).as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
        }
    }
}

/// sequentially handle one request
async fn service(mut rx: Receiver<RequestTransport>) {
    while let Some((command, response_channel)) = rx.recv().await {
        println!("Service received: {:?}", command);
        // TODO: do calculation
        response_channel.send(Response::Ok()).unwrap();
    }
}
