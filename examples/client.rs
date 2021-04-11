use palantir_proto::palantir::apm::v1::action::ApmV1Action;
use palantir_proto::palantir::request::request::Message;
use palantir_proto::palantir::request::Request;
use palantir_proto::prost::bytes::BytesMut;
use palantir_proto::prost::Message as ProstMessage;
use std::net::UdpSocket;

// requires server running with udp listener on 5045 port
fn main() {
    let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    socket
        .connect("127.0.0.1:5545")
        .expect("connect function failed");

    let request = Request {
        message: Some(Message::ApmV1Action(ApmV1Action {
            realm: "example-realm".to_string(),
            application: "example-application".to_string(),
            application_hash: "3fde5".to_string(),
            action_kind: "http".to_string(),
            action_name: "controllers.example".to_string(),
            total_us: 55_000_000u64,
            additional_dimensions: vec![],
            measurements: vec![],
        })),
    };

    let mut buf = BytesMut::with_capacity(request.encoded_len());
    request.encode(&mut buf).unwrap();

    match socket.send(&buf) {
        Ok(bytes) => {
            println!("sent {} bytes", bytes)
        }
        Err(err) => {
            println!("error: {:?}", err)
        }
    };
}
