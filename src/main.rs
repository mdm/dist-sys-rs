use std::{io::stdin, time::SystemTime};

use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::message::{MsgEnvelope, MsgPayload};

mod message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut msg_id = 1;
    let mut buffer = String::new();
    let mut my_node_id = None;
    let mut num_nodes = 0;
    let mut rng = None;
    loop {
        let len = stdin().read_line(&mut buffer)?;
        if len == 0 {
            eprintln!("Detected EOF, exiting.");
            break;
        }
        eprintln!("Received request ({} bytes): {}", len, buffer);
        let parsed = serde_json::from_str::<message::MsgEnvelope<'_>>(&buffer).unwrap();
        let response: MsgEnvelope<'_> = match parsed.body.payload {
            MsgPayload::Init {
                node_id,
                ref node_ids,
            } => {
                my_node_id = Some(node_id);
                num_nodes = node_ids.len();
                parsed.reply(msg_id, MsgPayload::InitOk)
            }
            MsgPayload::Echo { echo } => parsed.reply(
                msg_id,
                MsgPayload::EchoOk {
                    echo: echo.to_string(),
                },
            ),
            MsgPayload::Generate => {
                if let (Some(node_id), None) = (my_node_id, &rng) {
                    rng = Some(StdRng::seed_from_u64(node_id.into()));
                }
                if let (Some(node_id), Some(rng)) = (my_node_id, &mut rng) {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("system time is before UNIX_EPOCH")
                        .as_secs();
                    let node_id: u64 = node_id.into();
                    let random_part: u32 = rng.next_u32();
                    let id = (now & 0xFFFFFFFF) << 32
                        | (node_id & 0x3) << 30
                        | (random_part as u64 & 0xFFFFFFFC);
                    parsed.reply(msg_id, MsgPayload::GenerateOk { id })
                } else {
                    panic!("Received Generate message before Init");
                }
            }
            _ => panic!("Unexpected message type"),
        };
        let serialized = serde_json::to_string(&response)?;
        println!("{}", serialized);
        msg_id += 1;
        buffer.clear();
    }
    Ok(())
}
