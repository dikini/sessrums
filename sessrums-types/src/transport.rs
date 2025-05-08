//! Transport layer abstraction for session type communication.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
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

//
// Multiparty Transport Abstraction
//

use crate::roles::Role;

/// Runtime identifier for roles in a multiparty session.
/// 
/// While the `Role` trait provides compile-time type safety,
/// `RoleIdentifier` allows for runtime identification of roles
/// for dynamic message routing in multiparty sessions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoleIdentifier {
    /// The name of the role
    name: String,
}

impl RoleIdentifier {
    /// Create a new role identifier with the given name
    pub fn new<S: Into<String>>(name: S) -> Self {
        RoleIdentifier { name: name.into() }
    }
    
    /// Get the name of the role
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for RoleIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Role({})", self.name)
    }
}

/// A transport channel for multiparty session type communication.
/// 
/// This trait extends the basic transport concept to support
/// sending messages to specific roles and receiving messages
/// from specific roles in a multiparty session.
pub trait MultipartyTransport {
    /// Send a serializable message to a specific role.
    fn send_to<M: Serialize>(&mut self, to: &RoleIdentifier, message: M) -> Result<(), SessionError>;
    
    /// Receive a deserializable message from a specific role.
    fn receive_from<M: DeserializeOwned>(&mut self, from: &RoleIdentifier) -> Result<M, SessionError>;
}

/// A message envelope for routing messages between participants.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageEnvelope {
    /// The sender of the message
    from: String,
    /// The recipient of the message
    to: String,
    /// The serialized message payload
    payload: Vec<u8>,
}

/// A broker for managing message routing between participants in a multiparty session.
/// 
/// The broker maintains a registry of participants and routes messages between them.
/// It provides a centralized point for message exchange in a multiparty session.
#[derive(Debug)]
pub struct MockMultipartyBroker {
    /// Mapping from role identifiers to participant message queues
    participants: Arc<Mutex<HashMap<String, Arc<Mutex<VecDeque<MessageEnvelope>>>>>>,
}

impl MockMultipartyBroker {
    /// Create a new multiparty broker
    pub fn new() -> Self {
        MockMultipartyBroker {
            participants: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Register a participant with the broker
    pub fn register_participant(&self, role: &RoleIdentifier) -> Result<(), SessionError> {
        let mut participants = self.participants.lock()?;
        if participants.contains_key(role.name()) {
            return Err(SessionError::ProtocolViolation(
                format!("Participant with role {} already registered", role.name())
            ));
        }
        
        participants.insert(
            role.name().to_string(),
            Arc::new(Mutex::new(VecDeque::new())),
        );
        
        Ok(())
    }
    
    /// Create a participant channel for the given role
    pub fn create_channel<R: Role>(&self, role: &RoleIdentifier) -> Result<ParticipantChannel<R>, SessionError> {
        let participants = self.participants.lock()?;
        
        if !participants.contains_key(role.name()) {
            return Err(SessionError::ProtocolViolation(
                format!("Participant with role {} not registered", role.name())
            ));
        }
        
        let queue = participants.get(role.name()).unwrap().clone();
        
        Ok(ParticipantChannel {
            role_id: role.clone(),
            queue,
            broker: self.participants.clone(),
            _role_type: std::marker::PhantomData,
        })
    }
    
    /// Get all registered role identifiers
    pub fn get_roles(&self) -> Result<Vec<RoleIdentifier>, SessionError> {
        let participants = self.participants.lock()?;
        
        Ok(participants.keys()
            .map(|name| RoleIdentifier::new(name.clone()))
            .collect())
    }
}

/// A channel for a participant in a multiparty session.
/// 
/// This channel implements the `MultipartyTransport` trait and provides
/// methods for sending messages to and receiving messages from other
/// participants in the session.
#[derive(Debug)]
pub struct ParticipantChannel<R: Role> {
    /// The runtime identifier for this participant's role
    role_id: RoleIdentifier,
    /// The queue for incoming messages
    queue: Arc<Mutex<VecDeque<MessageEnvelope>>>,
    /// Reference to the broker's participant registry
    broker: Arc<Mutex<HashMap<String, Arc<Mutex<VecDeque<MessageEnvelope>>>>>>,
    /// Phantom data for the compile-time role type
    _role_type: std::marker::PhantomData<R>,
}

impl<R: Role> ParticipantChannel<R> {
    /// Get the role identifier for this participant
    pub fn role_id(&self) -> &RoleIdentifier {
        &self.role_id
    }
}

impl<R: Role> MultipartyTransport for ParticipantChannel<R> {
    fn send_to<M: Serialize>(&mut self, to: &RoleIdentifier, message: M) -> Result<(), SessionError> {
        // Serialize the message
        let payload = bincode::serialize(&message)?;
        
        // Create the message envelope
        let envelope = MessageEnvelope {
            from: self.role_id.name().to_string(),
            to: to.name().to_string(),
            payload,
        };
        
        // Get the broker's participant registry
        let participants = self.broker.lock()?;
        
        // Find the recipient's queue
        let recipient_queue = participants.get(to.name()).ok_or_else(|| {
            SessionError::ProtocolViolation(format!("Recipient {} not found", to.name()))
        })?;
        
        // Add the message to the recipient's queue
        let mut queue = recipient_queue.lock()?;
        queue.push_back(envelope);
        
        Ok(())
    }
    
    fn receive_from<M: DeserializeOwned>(&mut self, from: &RoleIdentifier) -> Result<M, SessionError> {
        // Get our incoming queue
        let mut queue = self.queue.lock()?;
        
        // Find a message from the specified sender
        let position = queue.iter().position(|envelope| envelope.from == from.name());
        
        if let Some(pos) = position {
            // Remove the message from the queue
            let envelope = queue.remove(pos).unwrap();
            
            // Deserialize the message
            return bincode::deserialize(&envelope.payload)
                .map_err(SessionError::Serialization);
        }
        
        // No message found
        Err(SessionError::UnexpectedClose)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};
    use crate::roles::{Client, Server};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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
        let queue2 = end2.incoming.lock()?;
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

    // Tests for the multiparty transport implementation
    
    #[test]
    fn test_role_identifier() {
        let role1 = RoleIdentifier::new("Alice");
        let role2 = RoleIdentifier::new("Bob");
        let role1_clone = RoleIdentifier::new("Alice");
        
        assert_eq!(role1.name(), "Alice");
        assert_eq!(role2.name(), "Bob");
        assert_eq!(role1, role1_clone);
        assert_ne!(role1, role2);
    }
    
    #[test]
    fn test_multiparty_broker_registration() -> Result<(), SessionError> {
        let broker = MockMultipartyBroker::new();
        
        let alice = RoleIdentifier::new("Alice");
        let bob = RoleIdentifier::new("Bob");
        
        // Register participants
        broker.register_participant(&alice)?;
        broker.register_participant(&bob)?;
        
        // Try to register a duplicate participant
        let result = broker.register_participant(&alice);
        assert!(result.is_err());
        
        // Get registered roles
        let roles = broker.get_roles()?;
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&alice));
        assert!(roles.contains(&bob));
        
        Ok(())
    }
    
    #[test]
    fn test_multiparty_message_passing() -> Result<(), SessionError> {
        // Create a broker
        let broker = MockMultipartyBroker::new();
        
        // Define roles
        let alice_id = RoleIdentifier::new("Alice");
        let bob_id = RoleIdentifier::new("Bob");
        let charlie_id = RoleIdentifier::new("Charlie");
        
        // Register participants
        broker.register_participant(&alice_id)?;
        broker.register_participant(&bob_id)?;
        broker.register_participant(&charlie_id)?;
        
        // Create channels for each participant
        let mut alice = broker.create_channel::<Client>(&alice_id)?;
        let mut bob = broker.create_channel::<Server>(&bob_id)?;
        let mut charlie = broker.create_channel::<Server>(&charlie_id)?;
        
        // Alice sends a message to Bob
        let msg_to_bob = TestMessage { data: "Hello Bob".to_string() };
        alice.send_to(&bob_id, msg_to_bob.clone())?;
        
        // Alice sends a message to Charlie
        let msg_to_charlie = TestMessage { data: "Hello Charlie".to_string() };
        alice.send_to(&charlie_id, msg_to_charlie.clone())?;
        
        // Bob receives the message from Alice
        let received_by_bob: TestMessage = bob.receive_from(&alice_id)?;
        assert_eq!(received_by_bob, msg_to_bob);
        
        // Charlie receives the message from Alice
        let received_by_charlie: TestMessage = charlie.receive_from(&alice_id)?;
        assert_eq!(received_by_charlie, msg_to_charlie);
        
        // Bob tries to receive another message from Alice (should fail)
        let result = bob.receive_from::<TestMessage>(&alice_id);
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_multiparty_error_handling() -> Result<(), SessionError> {
        // Create a broker
        let broker = MockMultipartyBroker::new();
        
        // Define roles
        let alice_id = RoleIdentifier::new("Alice");
        let bob_id = RoleIdentifier::new("Bob");
        let eve_id = RoleIdentifier::new("Eve");
        
        // Register only Alice and Bob
        broker.register_participant(&alice_id)?;
        broker.register_participant(&bob_id)?;
        
        // Create channels for Alice and Bob
        let mut alice = broker.create_channel::<Client>(&alice_id)?;
        let _bob = broker.create_channel::<Server>(&bob_id)?;
        
        // Alice tries to send a message to Eve (should fail)
        let msg = TestMessage { data: "Secret message".to_string() };
        let result = alice.send_to(&eve_id, msg);
        assert!(result.is_err());
        
        // Try to create a channel for Eve (should fail)
        let result = broker.create_channel::<Client>(&eve_id);
        assert!(result.is_err());
        
        Ok(())
    }
    
    #[test]
    fn test_concurrent_message_passing() -> Result<(), SessionError> {
        use std::thread;
        
        // Create a broker
        let broker = Arc::new(MockMultipartyBroker::new());
        
        // Define roles
        let alice_id = RoleIdentifier::new("Alice");
        let bob_id = RoleIdentifier::new("Bob");
        
        // Clone the role identifiers for use in threads
        let alice_id_clone1 = alice_id.clone();
        let bob_id_clone1 = bob_id.clone();
        let alice_id_clone2 = alice_id.clone();
        let bob_id_clone2 = bob_id.clone();
        
        // Register participants
        broker.register_participant(&alice_id)?;
        broker.register_participant(&bob_id)?;
        
        // Create channels for each participant
        let _alice = broker.create_channel::<Client>(&alice_id)?;
        let _bob = broker.create_channel::<Server>(&bob_id)?;
        
        // Shared broker for threads
        let broker_alice = broker.clone();
        let broker_bob = broker.clone();
        
        // Thread for Alice
        let alice_thread = thread::spawn(move || -> Result<(), SessionError> {
            let mut alice = broker_alice.create_channel::<Client>(&alice_id_clone1)?;
            
            // Send 10 messages to Bob
            for i in 0..10 {
                let msg = TestMessage { data: format!("Message {}", i) };
                alice.send_to(&bob_id_clone1, msg)?;
            }
            
            Ok(())
        });
        
        // Thread for Bob
        let bob_thread = thread::spawn(move || -> Result<(), SessionError> {
            let mut bob = broker_bob.create_channel::<Server>(&bob_id_clone2)?;
            
            // Receive 10 messages from Alice
            for i in 0..10 {
                let msg: TestMessage = bob.receive_from(&alice_id_clone2)?;
                assert_eq!(msg.data, format!("Message {}", i));
            }
            
            Ok(())
        });
        
        // Wait for threads to complete
        alice_thread.join().unwrap()?;
        bob_thread.join().unwrap()?;
        
        Ok(())
    }
}