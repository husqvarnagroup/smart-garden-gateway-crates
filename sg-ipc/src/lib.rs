#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate
)]

use anyhow::{Context, Error};
use std::future::Future;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Sender;
use tracing::Instrument;

const MESSAGE_BUFFER_INITIAL_CAPACITY_SIZE: usize = 2 * 1024 * 1024;
const MESSAGE_BUFFER_SIZE: usize = 16 * 1024 * 1024;

const CONNECT_RETRY_SLEEP_DURATION: tokio::time::Duration = tokio::time::Duration::from_secs(3);

const SEND_RECONNECT_SLEEP_DURATION: tokio::time::Duration = tokio::time::Duration::from_secs(3);

const SEND_RETRY_SLEEP_DURATION: tokio::time::Duration = tokio::time::Duration::from_millis(200);

/// A frame is a message with a newline at the end.
fn make_frame(msg: &str) -> String {
    // Escape newlines in the message to avoid breaking the protocol
    let msg = msg.replace('\n', "\\n");
    // Trim whitespace from the beginning and end of the message
    let msg = msg.trim();
    // Append a newline to the message to delimit the message
    format!("{msg}\n")
}

fn bind_domain_socket(url: &str) -> Result<UnixListener, Error> {
    std::fs::remove_file(url).map_or_else(
        |e| match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        },
        Ok,
    )?;
    let socket_dir = std::path::Path::new(url)
        .parent()
        .context("Could not get parent directory of Unix socket path")?;
    std::fs::create_dir_all(socket_dir).context("Could not create directory for Unix socket")?;

    let listener = UnixListener::bind(url).context(format!("Could not bind Unix Socket {url}"))?;
    Ok(listener)
}

async fn connect_domain_socket(url: &str) -> UnixStream {
    loop {
        match UnixStream::connect(&url).await {
            Ok(stream) => break stream,
            Err(e) => {
                log::error!("Failed to connect to Unix socket at {url}: {e}. Retrying...");
                tokio::time::sleep(CONNECT_RETRY_SLEEP_DURATION).await;
            }
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum SocketReadError {
    #[error("Socket peer disconnected")]
    PeerDisconnected,
    #[error("Could not read from socket")]
    ReadError,
}

async fn read_from_socket(
    reader: &mut OwnedReadHalf,
    url: &str,
) -> Result<String, SocketReadError> {
    let mut reader = BufReader::with_capacity(MESSAGE_BUFFER_INITIAL_CAPACITY_SIZE, reader);
    let mut buf = Vec::with_capacity(MESSAGE_BUFFER_INITIAL_CAPACITY_SIZE);
    let res = reader.read_until(b'\n', &mut buf).await;
    let msg = match res {
        Ok(0) => {
            log::info!("Peer has disconnected on {url}");
            Err(SocketReadError::PeerDisconnected)
        }
        Ok(_) => {
            if buf.len() > MESSAGE_BUFFER_SIZE {
                log::error!("Received message exceeds maximum expected size of {MESSAGE_BUFFER_SIZE} bytes on {url}");
                return Err(SocketReadError::ReadError);
            }
            let msg = std::str::from_utf8(&buf);
            match msg {
                Ok(msg) => {
                    log::debug!("Received message: {msg} on {url}");
                    Ok(msg.to_string())
                }
                Err(e) => {
                    log::error!("Received invalid UTF-8: {e} on {url}");
                    return Err(SocketReadError::ReadError);
                }
            }
        }
        Err(e) => {
            log::error!("Error reading from stream: {e} on {url}");
            return Err(SocketReadError::ReadError);
        }
    };
    msg
}

async fn send_to_socket(
    message: &str,
    writer: &mut OwnedWriteHalf,
    url: &str,
) -> std::io::Result<()> {
    let message = make_frame(message);
    log::debug!("Sending message: {message} to {url}");
    writer.write_all(message.as_bytes()).await
}

pub struct RepService {
    url: String,
}

impl RepService {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
        }
    }

    pub fn start<Fut>(
        self,
        callback: impl Fn(String) -> Fut + Clone + Send + Sync + 'static,
    ) -> Result<(), anyhow::Error>
    where
        Fut: Future<Output = Result<String, anyhow::Error>> + 'static + Send,
    {
        let listener = bind_domain_socket(&self.url)?;

        tokioutil::spawn_named("rep-service-accept", async move {
            self.accept(callback, listener)
                .await
                .expect("rep-service accept failed");
        });

        Ok(())
    }

    async fn accept<Fut>(
        &self,
        callback: impl Fn(String) -> Fut + Clone + Send + Sync + 'static,
        listener: UnixListener,
    ) -> Result<(), SocketReadError>
    where
        Fut: Future<Output = Result<String, Error>> + Send + 'static,
    {
        loop {
            log::info!("Waiting for connection on rep");
            let stream = match listener.accept().await {
                Ok((stream, addr)) => {
                    log::info!("Accepted connection on rep {addr:?}");
                    stream
                }
                Err(e) => {
                    log::error!("Failed to accept req connection on: {e}");
                    continue;
                }
            };
            let url = self.url.clone();
            let callback = callback.clone();
            log::debug!("Accepted connection on {url}");
            tokioutil::spawn_named("rep-accept-task", async move {
                let (mut reader, mut writer) = stream.into_split();
                loop {
                    let res = read_from_socket(&mut reader, &url).await;
                    let resp = match res {
                        Ok(msg) => {
                            log::debug!("Executing callback for received message on {url}: {msg}");
                            Self::execute_callback(callback.clone(), msg, url.clone()).await
                        }
                        Err(SocketReadError::PeerDisconnected) => {
                            log::debug!("Peer disconnected on {url}");
                            return;
                        }
                        Err(SocketReadError::ReadError) => {
                            log::error!("Error reading from socket on {url}, continuing...");
                            continue;
                        }
                    };

                    if let Some(response) = resp {
                        if let Err(err) = send_to_socket(&response, &mut writer, &url).await {
                            log::error!("Failed to send response on {url} ({err})");
                        }
                    }
                }
            });
        }
    }

    async fn execute_callback<Fut>(
        callback: impl Fn(String) -> Fut + Clone + Send + Sync,
        msg: String,
        url: String,
    ) -> Option<String>
    where
        Fut: Future<Output = Result<String, Error>> + 'static + Send,
    {
        match callback(msg).await {
            Ok(response) => Some(response),
            Err(e) => {
                log::error!("Callback error: {e} on {url}");
                None
            }
        }
    }
}

pub struct PubServiceBuilder {
    tx: broadcast::Sender<String>,
}

impl PubServiceBuilder {
    pub fn new() -> (Self, PubService) {
        let (tx, _rx) = broadcast::channel(16);
        (Self { tx: tx.clone() }, PubService { tx })
    }

    pub fn start(&self, url: &str) -> Result<(), Error> {
        let listener = bind_domain_socket(url)?;

        let url = url.to_string();
        let tx = self.tx.clone();
        tokioutil::spawn_named("pub-service-accept", async move {
            Self::accept(listener, url, tx).await;
        });

        Ok(())
    }

    async fn accept(listener: UnixListener, url: String, tx: Sender<String>) {
        loop {
            log::info!("Waiting for connection on {url}");
            let stream = match listener.accept().await {
                Ok((stream, _)) => stream,
                Err(e) => {
                    log::error!("Failed to accept connection on {url}: {e}");
                    continue;
                }
            };
            let tx = tx.clone();
            let url = url.clone();
            tokioutil::spawn_named(
                "pub-service-accept",
                async move {
                    let (mut reader, mut writer) = stream.into_split();
                    let mut rx = tx.subscribe();
                    loop {
                        tokio::select! {
                            msg = Self::read_from_channel(&mut rx) => {
                                if let Some(msg) = msg {
                                    if let Err(e) = send_to_socket(&msg, &mut writer, &url).await {
                                        log::error!("pub/sub: failed to write to stream: {}", e);
                                        tokio::time::sleep(SEND_RETRY_SLEEP_DURATION).await;

                                    }
                                }
                            },
                            read = read_from_socket(&mut reader, &url) => {
                                match read {
                                    Ok(_) => {
                                        // Client sent a message, which is unexpected in pub-sub.
                                    log::warn!("Received unexpected message from client on {url}");
                                    },
                                    Err(SocketReadError::ReadError) => {},
                                    Err(SocketReadError::PeerDisconnected) => {
                                        log::info!("Client disconnected on {url}");
                                        break;
                                    },
                                }
                        }
                           }
                    }
                }
                .instrument(tracing::info_span!(parent:None, "pub-service")),
            );
        }
    }

    async fn read_from_channel(rx: &mut broadcast::Receiver<String>) -> Option<String> {
        let msg = rx.recv().await;
        {
            match msg {
                Ok(msg) => Some(msg.clone()),
                Err(RecvError::Closed) => {
                    log::warn!("Publisher channel closed");
                    None
                }
                Err(e) => {
                    log::error!("Failed to receive message from publisher channel: {}", e);
                    None
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct PubService {
    tx: broadcast::Sender<String>,
}

impl PubService {
    pub fn publish(&self, msg: &str) -> Result<(), broadcast::error::SendError<String>> {
        // No loop needed here, if the channel is closed we should ignore it
        match self.tx.send(msg.to_string()) {
            Ok(_) => Ok(()),
            Err(e) if e.to_string().contains("closed") => {
                log::warn!("No active channel subscribers; message dropped: {}", e);
                Ok(()) // ignore if that's acceptable
            }
            Err(e) => Err(e),
        }
    }
}

pub struct ReqService {
    url: String,
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
}

impl ReqService {
    pub async fn new(url: &str) -> Result<Self, Error> {
        let stream = connect_domain_socket(url).await;

        let (reader, writer) = stream.into_split();
        Ok(Self {
            url: url.to_owned(),
            reader,
            writer,
        })
    }

    pub async fn reconnect(&mut self) -> Result<(), Error> {
        log::info!("Reconnecting to socket: {}", self.url);

        // Attempt to establish a new connection
        let stream = UnixStream::connect(&self.url).await?;

        // Split into read and write halves
        let (reader, writer) = stream.into_split();

        // Replace old halves
        self.reader = reader;
        self.writer = writer;

        log::info!("Successfully reconnected to {}", self.url);
        Ok(())
    }

    pub async fn send(&mut self, message: String) -> Result<String, Error> {
        // First attempt to send
        if let Err(e) = send_to_socket(&message, &mut self.writer, &self.url).await {
            log::error!(
                "req/res: failed to write to stream: {}. Trying to reconnect...",
                e
            );

            // Try to reconnect once
            if let Err(e) = self.reconnect().await {
                log::error!("req/res: reconnect failed: {}", e);
                return Err(e);
            }

            // Retry sending after reconnect
            if let Err(e) = send_to_socket(&message, &mut self.writer, &self.url).await {
                log::error!("req/res: failed again after reconnect: {}", e);
                return Err(Error::from(e));
            }
        }

        // Read the response
        let msg = read_from_socket(&mut self.reader, &self.url).await?;
        Ok(msg)
    }
}

pub struct SubService {
    url: String,
}

impl SubService {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
        }
    }

    pub async fn start<Fut>(
        &mut self,
        callback: impl Fn(String) -> Fut + Clone + Send + Sync + 'static,
    ) -> Result<(), Error>
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        loop {
            let stream = connect_domain_socket(&self.url).await;

            let (mut reader, _writer) = stream.into_split();

            let ret = self.handle_messages(callback.clone(), &mut reader).await;
            match ret {
                Ok(()) => return Ok(()),
                Err(e) => {
                    log::debug!("Error {e} on {}. Reconnecting...", &self.url);
                    tokio::time::sleep(SEND_RECONNECT_SLEEP_DURATION).await;
                }
            }
        }
    }

    async fn handle_messages<Fut>(
        &mut self,
        callback: impl Fn(String) -> Fut + Clone + Send + Sync,
        reader: &mut OwnedReadHalf,
    ) -> Result<(), SocketReadError>
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        loop {
            let msg = read_from_socket(reader, &self.url).await?;
            log::debug!(
                "Executing callback for received message {msg} on {}",
                &self.url
            );
            callback(msg.clone()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_make_frame() {
        let msg = "Hello, World!";
        let framed_msg = make_frame(msg);
        assert_eq!(framed_msg, "Hello, World!\n");
    }

    #[tokio::test]
    async fn test_make_frame_escaped() {
        let msg = "Hello\nWorld!";
        let framed_msg = make_frame(msg);
        assert_eq!(framed_msg, "Hello\\nWorld!\n");
    }
}
