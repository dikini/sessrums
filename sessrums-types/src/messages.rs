//! Message types for the session type protocols.
//! 
//! This module defines the message types that can be sent between roles,
//! along with their serialization behavior.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PingMsg {
    pub seq: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PongMsg {
    pub seq: Option<i32>,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{serialize, deserialize};

    #[test]
    fn test_ping_serialization() {
        let ping = PingMsg { seq: Some(1) };
        let encoded = serialize(&ping).unwrap();
        let decoded: PingMsg = deserialize(&encoded).unwrap();
        assert_eq!(ping, decoded);
    }

    #[test]
    fn test_pong_serialization() {
        let pong = PongMsg { 
            seq: Some(1), 
            timestamp: 12345 
        };
        let encoded = serialize(&pong).unwrap();
        let decoded: PongMsg = deserialize(&encoded).unwrap();
        assert_eq!(pong, decoded);
    }
}