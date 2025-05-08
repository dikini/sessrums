//! Transport layer abstraction for session type communication.

use std::io;
use serde::{Serialize, de::DeserializeOwned};
use crate::error::SessionError;

/// A transport channel for session type communication.
pub trait Transport {
    /// Send a serializable payload through the transport.
    fn send_payload<T: Serialize>(&mut self, payload: &T) -> Result<(), SessionError>;
    
    /// Receive a deserializable payload from the transport.
    fn receive_payload<T: DeserializeOwned>(&mut self) -> Result<T, SessionError>;
}

/// A mock transport implementation for testing.
#[derive(Debug)]
pub struct MockChannelEnd {
    /// Local queue for receiving messages
    incoming: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<Vec<u8>>>>,
    /// Reference to the other end's incoming queue
    other_incoming: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<Vec<u8>>>>,
}
impl MockChannelEnd {
    pub fn new_pair() -> (Self, Self) {
        let queue1 = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
        let queue2 = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
        
        (
            MockChannelEnd {
                incoming: queue1.clone(),
                other_incoming: queue2.clone(),
            },
            MockChannelEnd {
                incoming: queue2.clone(),
                other_incoming: queue1.clone(),
            },
        )
    }
}
impl Transport for MockChannelEnd {
    fn send_payload<T: Serialize>(&mut self, payload: &T) -> Result<(), SessionError> {
        let bytes = bincode::serialize(payload)?;
        let mut queue = self.other_incoming.lock()?;
        queue.push_back(bytes);
        Ok(())
    }

    fn receive_payload<T: DeserializeOwned>(&mut self) -> Result<T, SessionError> {
        // First, check our local incoming queue
        let mut queue = self.incoming.lock()?;
        if let Some(bytes) = queue.pop_front() {
            return bincode::deserialize(&bytes).map_err(SessionError::Serialization);
        }
    
        // If our incoming queue is empty, return UnexpectedClose
        Err(SessionError::UnexpectedClose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestMessage {
        data: String,
    }

    #[test]
    fn test_mock_channel() -> Result<(), SessionError> {
        let (mut end1, mut end2) = MockChannelEnd::new_pair();
        let msg = TestMessage { data: "hello".to_string() };
    
        // Assert that the queues are initially empty
        let queue1 = end1.other_incoming.lock()?;
        assert_eq!(queue1.len(), 0);
        drop(queue1);

        let queue2 = end2.incoming.lock()?;
        assert_eq!(queue2.len(), 0);
        drop(queue2);

        end1.send_payload(&msg)?;
        
        
        // Assert that the message was sent
        let mut queue2 = end2.incoming.lock()?;
        assert_eq!(queue2.len(), 1);
        drop(queue2);
    
        let received: TestMessage = end2.receive_payload()?;
        assert_eq!(received, msg);
    
        // Assert that the message was received and the queue is empty
        let queue2 = end2.incoming.lock()?;
        assert_eq!(queue2.len(), 0);
        drop(queue2);
    
        Ok(())
    }

    #[test]
    fn test_queue_behavior() -> Result<(), SessionError> {
        let (mut end1, mut end2) = MockChannelEnd::new_pair();
        
        let msg = TestMessage { data: "test".to_string() };
        end1.send_payload(&msg)?;
        
        // Test receiving on correct end
        let received: TestMessage = end2.receive_payload()?;  // Add type annotation
        assert_eq!(received, msg);
        
        // Test empty queue behavior
        return match end1.receive_payload::<TestMessage>() {
            Err(SessionError::UnexpectedClose) => Ok(()),
            Ok(_) => panic!("Expected UnexpectedClose, got success"),
            Err(e) => panic!("Expected UnexpectedClose, got different error: {:?}", e),
        };
    }
    
    #[test]
    fn test_choice_signal_transmission() -> Result<(), SessionError> {
        use crate::session_types::binary::ChoiceSignal;
        
        // Create a mock channel pair
        let (mut end1, mut end2) = MockChannelEnd::new_pair();
        
        // Test sending and receiving ChoiceSignal::Left
        end1.send_payload(&ChoiceSignal::Left)?;
        let received_left: ChoiceSignal = end2.receive_payload()?;
        assert!(matches!(received_left, ChoiceSignal::Left));
        
        // Test sending and receiving ChoiceSignal::Right
        end1.send_payload(&ChoiceSignal::Right)?;
        let received_right: ChoiceSignal = end2.receive_payload()?;
        assert!(matches!(received_right, ChoiceSignal::Right));
        
        Ok(())
    }

    #[test]
    fn test_multiple_messages() -> Result<(), SessionError> {
        // Arrange
        let (mut end1, mut end2) = MockChannelEnd::new_pair();
        let msg1 = TestMessage { data: "first".to_string() };
        let msg2 = TestMessage { data: "second".to_string() };

        // Act
        end1.send_payload(&msg1)?;
        end1.send_payload(&msg2)?;

        let received1: TestMessage = end2.receive_payload()?;
        let received2: TestMessage = end2.receive_payload()?;

        // Assert
        assert_eq!(received1, msg1);
        assert_eq!(received2, msg2);

        Ok(())
    }

}