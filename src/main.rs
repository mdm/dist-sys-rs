use std::io::stdin;

use crate::message::{MsgEnvelope, MsgPayload};

mod message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut msg_id = 1;
    let mut buffer = String::new();
    loop {
        let len = stdin().read_line(&mut buffer)?;
        if len == 0 {
            eprintln!("Detected EOF, exiting.");
            break;
        }
        eprintln!("Received request ({} bytes): {}", len, buffer);
        let parsed = serde_json::from_str::<message::MsgEnvelope<'_>>(&buffer).unwrap();
        let response: MsgEnvelope<'_> = match parsed.body.payload {
            MsgPayload::Init { .. } => parsed.reply(msg_id, MsgPayload::InitOk),
            MsgPayload::Echo { echo } => parsed.reply(
                msg_id,
                MsgPayload::EchoOk {
                    echo: echo.to_string(),
                },
            ),
            _ => panic!("Unexpected message type"),
        };
        let serialized = serde_json::to_string(&response)?;
        println!("{}", serialized);
        msg_id += 1;
        buffer.clear();
    }
    Ok(())
}
