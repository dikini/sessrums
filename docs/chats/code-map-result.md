# sessrums-types Codebase Map

## Symbol Index

### src/error.rs
Defines error types and handling for session operations using thiserror.

- enum SessionError : error.rs (6-22)
- impl From<bincode::Error> for SessionError : error.rs (24-28)
- impl<T> From<PoisonError<T>> for SessionError : error.rs (30-34)
- #[cfg(test)] mod tests : error.rs (36-56)
  - #[test] fn test_error_messages : error.rs (38-49)
  - #[test] fn test_error_conversion : error.rs (51-55)

### src/lib.rs
Crate root with module declarations and public API exports.

- pub mod roles : lib.rs (6-6)
- pub mod messages : lib.rs (7-7)
- pub mod error : lib.rs (8-8)
- pub mod transport : lib.rs (9-9)
- pub mod session_types : lib.rs (10-10)
- pub use error::SessionError : lib.rs (13-13)
- pub use transport::Transport : lib.rs (14-14)

### src/messages.rs
Defines serializable message types (PingMsg, PongMsg) using serde.

- struct PingMsg : messages.rs (8-11)
- struct PongMsg : messages.rs (13-17)
- #[cfg(test)] mod tests : messages.rs (19-43)
  - #[test] fn test_ping_serialization : messages.rs (24-30)
  - #[test] fn test_pong_serialization : messages.rs (32-41)

### src/roles.rs
Implements protocol roles (Client, Server) as zero-sized types with a sealed Role trait.

- trait Role : roles.rs (7-7)
- struct Client : roles.rs (14-15)
- struct Server : roles.rs (17-18)
- impl Role for Client : roles.rs (20-20)
- impl Role for Server : roles.rs (21-21)
- mod private : roles.rs (24-28)
- #[cfg(test)] mod tests : roles.rs (30-45)
  - #[test] fn roles_are_copy : roles.rs (33-44)

### src/session_types/mod.rs
Module organization and re-exports for session type components.

- pub mod binary : session_types/mod.rs (3-3)
- pub use binary::{End, Send, Receive, Session, Offer, Select, Dual, Either, ChoiceSignal} : session_types/mod.rs (5-5)

### src/session_types/binary.rs
Core implementation of binary session types using the typestate pattern.

- enum ChoiceSignal : binary.rs (28-33)
- enum Either<L, R> : binary.rs (79-87)
- struct Send<M, NextP> : binary.rs (188-189)
- struct Receive<M, NextP> : binary.rs (235-236)
- struct Offer<L, R> : binary.rs (304-305)
- struct Select<L, R> : binary.rs (365-366)
- struct Session<S, T: Transport> : binary.rs (413-416)
- trait Dual : binary.rs (907-911)
- impl Dual for End : binary.rs (913-917)
- impl<M, P> Dual for Send<M, P> : binary.rs (918-926)
- impl<M, P> Dual for Receive<M, P> : binary.rs (927-935)
- impl<L, R> Dual for Offer<L, R> : binary.rs (946-970)
- impl<L, R> Dual for Select<L, R> : binary.rs (992-1034)
- impl<S, T: Transport> Session<S, T> : binary.rs (419-429)
- impl<T: Transport> Session<End, T> : binary.rs (430-434)
- impl<M, NextP, T: Transport> Session<Send<M, NextP>, T> : binary.rs (444-456)
- impl<M, NextP, T: Transport> Session<Receive<M, NextP>, T> : binary.rs (457-469)
- impl<L, R, T: Transport> Session<Select<L, R>, T> : binary.rs (483-583)
- impl<L, R, T: Transport> Session<Offer<L, R>, T> : binary.rs (592-665)
- #[cfg(test)] mod tests : binary.rs (670-870)
  - #[test] fn test_ping_pong_protocol : binary.rs (678-699)
  - #[test] fn test_session_type_safety : binary.rs (701-710)
  - #[test] fn test_choice_protocol : binary.rs (731-806)
  - #[test] fn test_duality_relationships : binary.rs (817-859)

### src/transport.rs
Transport abstraction trait and mock implementation for testing.

- trait Transport : transport.rs (7-19)
- struct MockChannelEnd : transport.rs (21-27)
- impl MockChannelEnd : transport.rs (29-44)
- impl Transport for MockChannelEnd : transport.rs (46-61)
- #[cfg(test)] mod tests : transport.rs (62-131)
  - #[derive(Debug, Serialize, Deserialize, PartialEq)] struct TestMessage : transport.rs (65-68)
  - #[test] fn test_mock_channel : transport.rs (70-96)
  - #[test] fn test_queue_behavior : transport.rs (98-112)
  - #[test] fn test_choice_signal_transmission : transport.rs (114-128)
  - #[test] fn test_multiple_messages : transport.rs (130-131)