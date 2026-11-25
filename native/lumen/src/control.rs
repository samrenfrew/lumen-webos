use crate::{note, post};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

pub const PORT: u16 = 9997;
const ANSWER_WITHIN: Duration = Duration::from_secs(20);

pub type Answer = Sender<Vec<u8>>;

pub fn said(answer: &Answer, value: serde_json::Value) {
    let _ = answer.send(format!("{value}\n").into_bytes());
}

fn serve(stream: TcpStream) {
    let Ok(mut out) = stream.try_clone() else { return };
    for line in BufReader::new(stream).lines() {
        let Ok(line) = line else { return };
        let (answer, answered) = channel();
        post(move |app| app.control(line.trim(), answer));
        let bytes = answered.recv_timeout(ANSWER_WITHIN).unwrap_or_else(|_| b"{\"error\":\"no answer\"}\n".to_vec());
        if out.write_all(&bytes).is_err() {
            return;
        }
    }
}

pub fn start() {
    std::thread::spawn(|| match TcpListener::bind(("127.0.0.1", PORT)) {
        Ok(listener) => {
            note!("control: listening on 127.0.0.1:{}", PORT);
            for stream in listener.incoming().flatten() {
                std::thread::spawn(move || serve(stream));
            }
        }
        Err(e) => note!("control: not listening: {}", e),
    });
}
