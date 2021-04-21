use crate::config::defs::UDPConfig;
use log::{error, info, trace, warn};
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use palantir_proto::palantir::request::Request;
use palantir_proto::prost::bytes::BytesMut;
use palantir_proto::prost::Message;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::mpsc::Sender;
use std::thread;

pub struct UDPListener {
    socket: UdpSocket,
    buffer_size: usize,
    tx: Sender<ProtoMessage>,
}

impl UDPListener {
    /// Ok  -> successfully bound to socket
    /// Err -> was unable to bind to socket
    pub fn new(config: &UDPConfig, tx: Sender<ProtoMessage>) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(SocketAddr::new(
            IpAddr::from(Ipv4Addr::new(127, 0, 0, 1)),
            config.port,
        ))?;

        Ok(Self {
            socket,
            tx,
            buffer_size: config.buffer_size as usize,
        })
    }

    /// blocks current thread in socket reading loop
    pub fn run(&self) {
        info!(
            "Starting UDP listener thread with id: {:?}, listening to {:?}",
            thread::current().id(),
            self.socket.local_addr().unwrap()
        );
        let overflow_size = self.buffer_size + 1;
        loop {
            let mut buf = BytesMut::with_capacity(overflow_size);
            buf.resize(overflow_size, 0);

            match self.socket.recv_from(&mut buf) {
                Ok((bytes_read, origin)) => {
                    trace!("Read {} bytes from {}", bytes_read, origin);
                    if bytes_read == overflow_size {
                        warn!("Request size too large, skipping");
                        continue;
                    }

                    buf.resize(bytes_read, 0);
                    match Request::decode(buf) {
                        Ok(request) => {
                            match request.message {
                                Some(msg) => {
                                    match self.tx.send(msg) {
                                        Ok(_) => trace!("Message sent to channel"),
                                        Err(_) => {
                                            error!("Message can't be sent to channel");
                                            // we are unable to operate normally with dropped receiver
                                            // and there is also no way to re-init whole data pipeline
                                            // TODO introduce mechanism of re-creating data pipeline
                                            std::process::exit(1);
                                        }
                                    }
                                }
                                None => {
                                    warn!("got empty message")
                                }
                            }
                        }
                        Err(err) => warn!("Unable to parse request, {:?}", err),
                    }
                }
                Err(err) => {
                    warn!("Unable to read from socket {:?}", err)
                }
            }
        }
    }
}
