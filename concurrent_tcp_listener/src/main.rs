use std::borrow::BorrowMut;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot,mpsc};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
enum Request {
    Unknown,
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

    // build a channel to a handler processing each request in turn in order to prevent concurrency issues
    let (tx, rx) = mpsc::channel::<RequestTransport>(100);
    tokio::spawn(async move {
        handle_single_request(rx).await;
    });

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            handle_client_connection(socket, &tx_clone.clone().borrow_mut()).await;
        });
    }
}

async fn handle_client_connection(socket: TcpStream, tx: &Sender<RequestTransport>) {
    println!("Connection from {}", socket.peer_addr().unwrap());
    let mut stream = BufStream::new(socket);

    loop {
        let line = &mut String::new();
        let nr_bytes = stream.read_line(line).await.unwrap();
        if nr_bytes == 0 {
            // println!("0 bytes read, closing connection");
            stream.write(b"bye\r\n").await.unwrap();
            stream.flush().await.unwrap();
            break;
        } else {
            stream.consume(nr_bytes);
            // println!("read {} bytes -> {}", nr_bytes, line);
            stream.write(format!("consumed {} bytes\r\n", nr_bytes).as_bytes()).await.unwrap();
            stream.flush().await.unwrap();

            let parts = line.split_whitespace().collect::<Vec<&str>>();
            // println!("parts {:?}", parts);
            let maybe_request: Result<Request, String> = match parts[0].to_lowercase().as_str() {
                "set" => if parts.len() >= 3 {
                    Ok(Request::Set(parts[1].to_string(), parts[2..].join(" ").to_string()))
                } else {
                    Err(format!("set: expected min 3 parts but received {}", parts.len()))
                },
                "get" => if parts.len() == 2 {
                    Ok(Request::Get(parts[1].to_string()))
                } else {
                    Err(format!("get: expected 2 parts but received {}", parts.len()))
                }
                _ => Err(format!("not a valid request: {}", parts[0]))
            };

            match maybe_request {
                Ok(request) => {
                    // send a request plus back channel through our channel
                    let (response_tx, response_rx) = oneshot::channel::<Response>();
                    tx.send((request, response_tx)).await.unwrap();

                    // "wait" for the response
                    let response = response_rx.await.unwrap();
                    stream.write(format!("response: {:?}\r\n", response).as_bytes()).await.unwrap();
                    stream.flush().await.unwrap();
                },
                Err(message) => {
                    stream.write(format!("bad request: {}\r\n", message).as_bytes()).await.unwrap();
                    stream.flush().await.unwrap();
                }
            }
        }
    }
}

async fn handle_single_request(mut rx: Receiver<RequestTransport>) {
    while let Some((command, response_channel)) = rx.recv().await {
        println!("Service received: {:?}", command);
        // TODO: do calculation
        response_channel.send(Response::Ok()).unwrap();
    }
}
