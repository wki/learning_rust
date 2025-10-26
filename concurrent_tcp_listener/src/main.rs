use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot,mpsc};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{signal, time};

#[derive(Debug)]
enum Request {
    Set(String, String),    // set key := value -> OK
    Get(String),            // get key -> value

    // management requests, used internally
    Persist(),              // persist hashmap to disk -> OK
    Close(),                // Close channel and terminate processing -> OK
}

#[derive(Debug)]
enum Response {
    Ok(),
    NotFound(String),
    Result(String),
}

/// data transferred over the channel to our service
type RequestTransport = (Request, oneshot::Sender<Response>);

async fn send_request_and_wait_for_response(r:Request, tx:&Sender<RequestTransport>) -> Response {
    let (response_tx, response_rx) = oneshot::channel::<Response>();
    tx.send((r, response_tx)).await.unwrap();
    response_rx.await.unwrap()
}

#[tokio::main]
async fn main() {
    println!("Listening on port 8000");
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    // build a channel to a handler processing each request in turn in order to prevent concurrency issues
    let (tx, rx) = mpsc::channel::<RequestTransport>(100);
    tokio::spawn(async move {
        handle_single_request(rx).await;
        println!("Handle Single Request Task isDone");
        std::process::exit(1);
    });

    // establish a Ctrl-C handler for a graceful shutdown
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();

        println!("CTRL-C received, Closing Service");
        send_request_and_wait_for_response(Request::Close(), &tx_clone).await;
    });


    // a timer triggering a Persist request every 20s
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        loop {
            time::sleep(time::Duration::from_secs(20)).await;
            send_request_and_wait_for_response(Request::Persist(), &tx_clone).await;
        };
    });

    // accept loop
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket.
        // The socket is moved to the new task and processed there.
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            handle_client_connection(socket, &tx_clone).await;
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
            stream.write(b"bye\r\n").await.unwrap();
            stream.flush().await.unwrap();
            break;
        } else {
            stream.consume(nr_bytes);
            stream.write(format!("consumed {} bytes\r\n", nr_bytes).as_bytes()).await.unwrap();
            stream.flush().await.unwrap();

            let parts = line.split_whitespace().collect::<Vec<&str>>();

            let maybe_request: Result<Request, String> = match parts[..] {
                ["set", key, value] =>
                    Ok(Request::Set(parts[1].to_string(), parts[2..].join(" ").to_string())),
                ["get", key] =>
                    Ok(Request::Get(parts[1].to_string())),
                _ => Err(format!("not a valid request: {:?}", parts))
            };

            match maybe_request {
                Ok(request) => {
                    let response = send_request_and_wait_for_response(request, tx).await;
                    stream.write(format!("response: {:?}\r\n", response).as_bytes()).await.unwrap();
                    stream.flush().await.unwrap();
                },
                Err(message) => {
                    stream.write(format!("bad request - {}\r\n", message).as_bytes()).await.unwrap();
                    stream.flush().await.unwrap();
                }
            }
        }
    }
}

async fn handle_single_request(mut rx: Receiver<RequestTransport>) {
    let mut storage = HashMap::new();
    // TODO: initialize or load hashmap from disk
    while let Some((command, response_channel)) = rx.recv().await {
        println!("Service received: {:?}", command);
        let response = match command {
            Request::Set(key, value) => {
                storage.insert(key, value);
                Response::Ok()
            },
            Request::Get(key) => {
                storage.get(&key)
                    .map(|v| Response::Result(v.clone()))
                    .unwrap_or(Response::NotFound(key))
            },

            // Maintenance requests
            Request::Close() => {
                rx.close();
                Response::Ok()
            },
            Request::Persist() => {
                // TODO: persist to disk if not changed
                println!("Persist request");
                Response::Ok()
            }
        };
        response_channel.send(response).unwrap();
    }

    println!("Service is finished. TODO: shut down");
    // TODO: finally persist hashmap to disk
}
