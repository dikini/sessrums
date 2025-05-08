//! Test for a protocol with nested recursion blocks.
//!
//! This test verifies that the macro can correctly parse and process a protocol
//! definition with nested recursion blocks, where one recursion block is nested
//! inside another.
//! 
//! The protocol demonstrates a complex interaction pattern where:
//! 1. An outer loop represents a session between Client and Server
//! 2. An inner loop represents a sub-session for handling multiple requests

use sessrums_macro::mpst;

// Define a protocol with nested recursion blocks where:
// - The outer recursion represents a session
// - The inner recursion represents a sub-session for handling multiple requests
// - The client can choose to start a new sub-session or end the session
mpst! {
    protocol NestedRecursionProtocol {
        // Define the participants
        participant Client;
        participant Server;
        participant Logger;

        // Outer recursion representing a session
        rec Session {
            // Client initiates a session
            Client -> Server: String;  // Session ID
            Server -> Logger: String;  // Log session start
            
            // Inner recursion representing a sub-session for handling requests
            rec RequestLoop {
                // Client sends a request
                Client -> Server: String;  // Request data
                Server -> Logger: String;  // Log request
                
                // Server processes the request and responds
                Server -> Client: String;  // Response data
                
                // Client decides whether to send another request or exit the inner loop
                choice at Client {
                    option ContinueRequests {
                        // Continue the inner recursion (send another request)
                        continue RequestLoop;
                    }
                    
                    option ExitRequestLoop {
                        // Exit the inner recursion
                        Client -> Server: String;  // "End requests"
                        Server -> Logger: String;  // Log end of requests
                        
                        // Client decides whether to start a new session or end
                        choice at Client {
                            option NewSession {
                                // Continue the outer recursion (start a new session)
                                continue Session;
                            }
                            
                            option EndSession {
                                // End the protocol
                                Client -> Server: String;  // "End session"
                                Server -> Logger: String;  // Log end of session
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    // This is just a compile-time test, so we don't need to do anything here
    // The test passes if the macro expansion compiles successfully
    let _protocol = NestedRecursionProtocol::new();
}