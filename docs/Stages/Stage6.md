# Stage 6 Implementation Plan: Multiparty Session Runtime

## Context

Stage 6 focuses on developing a robust runtime for multiparty session types. This stage builds upon the previous work of defining global protocols (Stage 3), local protocols (Stage 4), and projection mechanisms (Stage 5). The goal is to create a `MultipartySession` struct that provides a type-safe API for participants to execute their projected local protocols.

This stage represents a critical transition from the type-level representation of protocols to their actual execution. The `MultipartySession` will serve as the primary interface for developers using the library, allowing them to implement role-specific behaviors in a type-safe manner.

---

## Review of Stage 6 Requirements

**Strengths:**
- Provides a type-safe API for executing multiparty protocols
- Maintains the typestate pattern established in binary sessions
- Enables practical use of projected local protocols
- Abstracts communication details while preserving protocol guarantees

**Risks/Considerations:**
- Complex type transitions for recursive protocols
- Handling of choice/branching across multiple participants
- Ensuring correct role-to-role communication channels
- Maintaining type safety across all operations
- Error handling and recovery strategies
- Performance considerations for real-world usage

---

## Stage 6: Actionable Task Breakdown

### 1. Project Preparation
- **Short:** Ensure Stage 5 is complete and all tests pass.
    - **Implementation Prompt:**  
      Run all Stage 5 tests and resolve any failures. Ensure the codebase is clean and ready for extension.
    - **Documentation Prompt:**  
      Document in the changelog or project notes that Stage 5 is a prerequisite for Stage 6.

---

### 2. Define MultipartySession Struct
- **Short:** Define the core `MultipartySession` struct with appropriate type parameters.
    - **Implementation Prompt:**  
      Implement the `MultipartySession<R: Role, S, T: MultipartyTransport>` struct where:
      - `R` is the role this session represents
      - `S` is the current protocol state (from the local protocol)
      - `T` is the transport mechanism for communicating with other roles
      
      Ensure the struct is properly documented and includes appropriate lifetime parameters.
    - **Documentation Prompt:**  
      Add comprehensive doc comments explaining the purpose of `MultipartySession`, its type parameters, and how it relates to the overall MPST system. Include examples of how it would be instantiated.

---

### 3. Define MultipartyTransport Trait
- **Short:** Define or refine the `MultipartyTransport` trait for role-aware communication.
    - **Implementation Prompt:**  
      Implement or update the `MultipartyTransport` trait with methods for sending to and receiving from specific roles:
      ```rust
      pub trait MultipartyTransport {
          fn send_to<OtherRole: Role, M: Serialize>(&mut self, message: M) -> Result<(), SessionError>;
          fn receive_from<OtherRole: Role, M: DeserializeOwned>(&mut self) -> Result<M, SessionError>;
          // Additional methods as needed
      }
      ```
      
      Ensure the trait is properly documented and includes appropriate bounds on generic parameters.
    - **Documentation Prompt:**  
      Document the `MultipartyTransport` trait, explaining its role in the multiparty session system and how it differs from the binary `Transport` trait. Include examples of how it would be implemented.

---

### 4. Implement MockMultipartyTransport
- **Short:** Create a mock implementation of `MultipartyTransport` for testing.
    - **Implementation Prompt:**  
      Implement `MockMultipartyTransport` that simulates a network of participants:
      ```rust
      pub struct MockMultipartyTransport<R: Role> {
          role: PhantomData<R>,
          channels: HashMap<TypeId, Box<dyn Any + Send>>,
          // Additional fields as needed
      }
      ```
      
      Implement methods to create a network of mock transports for testing, ensuring messages are correctly routed between roles.
    - **Documentation Prompt:**  
      Document the mock transport implementation, explaining how it simulates a network of participants and how it can be used in tests. Include examples of setting up a test network.

---

### 5. Basic Session Methods
- **Short:** Implement basic methods on `MultipartySession` for session creation and termination.
    - **Implementation Prompt:**  
      Implement:
      ```rust
      impl<R: Role, S, T: MultipartyTransport> MultipartySession<R, S, T> {
          pub fn new(transport: T) -> Self { /* ... */ }
          // For End state
          pub fn close(self) -> T where S: IsEnd { /* ... */ }
      }
      ```
      
      Include appropriate trait bounds and helper traits (e.g., `IsEnd`) as needed.
    - **Documentation Prompt:**  
      Document these methods, explaining how sessions are created and terminated. Include examples of usage.

---

### 6. Send/Receive Methods
- **Short:** Implement methods for sending and receiving messages between roles.
    - **Implementation Prompt:**  
      Implement:
      ```rust
      impl<R: Role, To: Role, M: Serialize + 'static, Next, T: MultipartyTransport> 
          MultipartySession<R, Send<To, M, Next>, T> {
          pub fn send_to(self, message: M) -> Result<MultipartySession<R, Next, T>, SessionError> {
              // Implementation
          }
      }
      
      impl<R: Role, From: Role, M: DeserializeOwned + 'static, Next, T: MultipartyTransport> 
          MultipartySession<R, Receive<From, M, Next>, T> {
          pub fn receive_from(self) -> Result<(M, MultipartySession<R, Next, T>), SessionError> {
              // Implementation
          }
      }
      ```
      
      Ensure proper error handling and type transitions.
    - **Documentation Prompt:**  
      Document these methods, explaining how messages are sent and received between roles. Include examples of usage and error handling.

---

### 7. Choice Methods (Select/Offer)
- **Short:** Implement methods for making and offering choices in multiparty protocols.
    - **Implementation Prompt:**  
      Implement:
      ```rust
      impl<R: Role, Decider: Role, L, R2, T: MultipartyTransport> 
          MultipartySession<R, Select<Decider, L, R2>, T> {
          pub fn select_left(self) -> Result<MultipartySession<R, L, T>, SessionError> where R: IsSameAs<Decider> {
              // Implementation for when this role is the decider
          }
          
          pub fn select_right(self) -> Result<MultipartySession<R, R2, T>, SessionError> where R: IsSameAs<Decider> {
              // Implementation for when this role is the decider
          }
          
          pub fn follow_choice(self) -> Result<Either<MultipartySession<R, L, T>, MultipartySession<R, R2, T>>, SessionError> 
              where R: Not<IsSameAs<Decider>> {
              // Implementation for when this role follows another's choice
          }
      }
      
      impl<R: Role, Decider: Role, L, R2, T: MultipartyTransport> 
          MultipartySession<R, Offer<Decider, L, R2>, T> {
          pub fn offer(self) -> Result<Either<MultipartySession<R, L, T>, MultipartySession<R, R2, T>>, SessionError> {
              // Implementation
          }
      }
      ```
      
      Include appropriate helper traits (e.g., `IsSameAs`, `Not`) to enforce role constraints.
    - **Documentation Prompt:**  
      Document these methods, explaining how choices are made and offered in multiparty protocols. Include examples of usage for both the deciding role and other roles.

---

### 8. Recursion Methods
- **Short:** Implement methods for handling recursion in multiparty protocols.
    - **Implementation Prompt:**  
      Implement:
      ```rust
      impl<R: Role, F, Body, T: MultipartyTransport> 
          MultipartySession<R, Rec<F>, T> 
          where F: FnOnce(Var) -> Body {
          pub fn enter_rec(self) -> MultipartySession<R, Body, T> {
              // Implementation
          }
      }
      
      impl<R: Role, T: MultipartyTransport> 
          MultipartySession<R, Var, T> {
          pub fn recurse(self) -> Result<MultipartySession<R, ?, T>, SessionError> {
              // Implementation - the return type needs careful consideration
          }
      }
      ```
      
      The recursion implementation will require careful handling of type transitions.
    - **Documentation Prompt:**  
      Document these methods, explaining how recursion is handled in multiparty protocols. Include examples of usage and explain any limitations or considerations.

---

### 9. Helper Traits and Type-Level Programming
- **Short:** Implement helper traits for type-level constraints and operations.
    - **Implementation Prompt:**  
      Implement traits such as:
      ```rust
      pub trait IsEnd {}
      impl IsEnd for End {}
      
      pub trait IsSameAs<R: Role> {}
      impl<R: Role> IsSameAs<R> for R {}
      
      pub trait Not<T> {}
      impl<R1: Role, R2: Role> Not<IsSameAs<R2>> for R1 where R1: NotSameRole<R2> {}
      
      pub trait NotSameRole<R: Role> {}
      // Implement for all distinct role pairs
      ```
      
      These traits will be used to enforce constraints at the type level.
    - **Documentation Prompt:**  
      Document these helper traits, explaining their purpose in enforcing protocol constraints. Include examples of how they are used in the implementation.

---

### 10. Integration with Projection
- **Short:** Ensure `MultipartySession` works seamlessly with projected local protocols.
    - **Implementation Prompt:**  
      Implement helper functions or methods to create a `MultipartySession` from a projected local protocol:
      ```rust
      pub fn create_session<R: Role, G: GlobalProtocol>(
          global: G,
          transport: impl MultipartyTransport
      ) -> MultipartySession<R, <G as Project<R>>::Output, _> {
          let local = global.project::<R>();
          // Create and return session
      }
      ```
      
      Ensure type compatibility between projected protocols and session states.
    - **Documentation Prompt:**  
      Document the integration between projection and session creation, explaining how developers can go from a global protocol to executing a role-specific session. Include comprehensive examples.

---

### 11. Testing
- **Short:** Create comprehensive tests for the multiparty session runtime.
    - **Implementation Prompt:**  
      Create tests in `tests/multiparty_session.rs` covering:
      - Basic send/receive between multiple roles
      - Choice/branching with multiple participants
      - Recursive protocols
      - Error handling and recovery
      - Integration with projection
      
      Use `MockMultipartyTransport` for testing.
    - **Documentation Prompt:**  
      Document each test, explaining the protocol being tested and what aspects of the runtime are being verified. Include diagrams or comments illustrating the expected message flow.

---

### 12. Documentation and Examples
- **Short:** Create comprehensive documentation and examples for the multiparty session runtime.
    - **Implementation Prompt:**  
      Add doc comments to all public items. Create example files demonstrating:
      - A simple three-party protocol (e.g., client-server-logger)
      - A protocol with choice (e.g., client-server-storage with success/failure paths)
      - A recursive protocol (e.g., streaming data with termination)
      
      Ensure examples are well-commented and illustrate best practices.
    - **Documentation Prompt:**  
      Create a dedicated documentation file (`docs/multiparty-session-runtime.md`) explaining:
      - The architecture of the multiparty session runtime
      - How to define and use multiparty protocols
      - Best practices for error handling and recovery
      - Performance considerations
      - Examples of common patterns

---

## Summary Table

| Task Group | Actions |
|------------|---------|
| Project Preparation | Ensure Stage 5 complete, dependencies |
| MultipartySession Struct | Define core struct with type parameters |
| MultipartyTransport Trait | Define transport abstraction for role-aware communication |
| MockMultipartyTransport | Implement mock transport for testing |
| Basic Session Methods | Implement session creation and termination |
| Send/Receive Methods | Implement type-safe message passing between roles |
| Choice Methods | Implement select/offer for multiparty choice |
| Recursion Methods | Implement recursion handling |
| Helper Traits | Implement type-level constraints and operations |
| Projection Integration | Ensure seamless integration with projection |
| Testing | Comprehensive test suite |
| Documentation/Examples | Doc comments, examples, best practices |

---

## Pre-conditions and Post-conditions

### Pre-conditions
- Stage 5 (Projection for Recursion & Full Multiparty Types) is complete and tested
- Global and local protocol representations are defined
- Projection mechanism is implemented and tested
- Transport abstraction is in place

### Post-conditions
- `MultipartySession` struct is implemented with typestate transitions
- Methods for all protocol operations (send, receive, select, offer, recursion) are implemented
- Mock transport for testing is implemented
- Integration with projection is verified
- Comprehensive test suite passes
- Documentation and examples are complete

The successful implementation of Stage 6 will provide a robust, type-safe runtime for multiparty session types, enabling practical use of the MPST system in Rust applications.