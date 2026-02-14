use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize, de::Visitor};

#[derive(Debug, Clone, Copy)]
pub enum EntityId {
    Client(usize),
    Node(usize),
}

impl Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityId::Client(id) => write!(f, "c{}", id),
            EntityId::Node(id) => write!(f, "n{}", id),
        }
    }
}

impl Serialize for EntityId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl FromStr for EntityId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err("EntityId too short");
        }

        let (prefix, id) = s.split_at_checked(1).ok_or("Invalid EntityId format")?;

        let id = id
            .parse::<usize>()
            .map_err(|_| "Invalid number for EntityId")?;

        match prefix {
            "c" => Ok(EntityId::Client(id)),
            "n" => Ok(EntityId::Node(id)),
            _ => Err("Invalid EntityId prefix"),
        }
    }
}

struct EntityIdVisitor;

impl<'de> Visitor<'de> for EntityIdVisitor {
    type Value = EntityId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, r#"a string like "c42" or "n7""#)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(serde::de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for EntityId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(EntityIdVisitor)
    }
}

impl From<EntityId> for u64 {
    fn from(entity_id: EntityId) -> Self {
        match entity_id {
            EntityId::Client(id) => id as u64,
            EntityId::Node(id) => id as u64,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MsgPayload<'p> {
    Init {
        node_id: EntityId,
        node_ids: Vec<EntityId>,
    },
    InitOk,
    Echo {
        echo: &'p str,
    },
    EchoOk {
        echo: String,
    },
    Generate,
    GenerateOk {
        id: u64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgBody<'b> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) msg_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) in_reply_to: Option<usize>,
    #[serde(borrow, flatten)]
    pub(crate) payload: MsgPayload<'b>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MsgEnvelope<'e> {
    pub(crate) src: EntityId,
    pub(crate) dest: EntityId,
    #[serde(borrow)]
    pub(crate) body: MsgBody<'e>,
}

impl MsgEnvelope<'_> {
    pub fn reply<'r>(&self, msg_id: usize, payload: MsgPayload<'r>) -> MsgEnvelope<'r> {
        MsgEnvelope {
            src: self.dest,
            dest: self.src,
            body: MsgBody {
                msg_id: Some(msg_id),
                in_reply_to: self.body.msg_id,
                payload,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_init_request() {
        let envelope = serde_json::from_str::<MsgEnvelope<'_>>(
            r#"{
            "src": "c1",
            "dest": "n1",
            "body": {
                "type": "init",
                "msg_id": 1,
                "node_id": "n1",
                "node_ids": ["n1", "n2", "n3"]
            }
        }"#,
        );

        assert!(envelope.is_ok());
    }

    #[test]
    fn test_deserialize_init_response() {
        let envelope = serde_json::from_str::<MsgEnvelope<'_>>(
            r#"{
            "src": "n1",
            "dest": "c1",
            "body": {
                "type": "init_ok",
                "in_reply_to": 1
            }
        }"#,
        );

        assert!(envelope.is_ok());
    }

    #[test]
    fn test_deserialize_echo_request() {
        let envelope = serde_json::from_str::<MsgEnvelope<'_>>(
            r#"{
            "src": "c1",
            "dest": "n1",
            "body": {
                "type": "echo",
                "msg_id": 1,
                "echo": "Please echo 35"
            }
        }"#,
        );

        assert!(envelope.is_ok());
    }

    #[test]
    fn test_deserialize_echo_response() {
        let envelope = serde_json::from_str::<MsgEnvelope<'_>>(
            r#"{
            "src": "n1",
            "dest": "c1",
            "body": {
                "type": "echo_ok",
                "msg_id": 1,
                "in_reply_to": 1,
                "echo": "Please echo 35"
            }
        }"#,
        );

        assert!(envelope.is_ok());
    }
}
