//! Test for a protocol with recursion within choice branches.
//!
//! This test verifies that the macro can correctly parse and process a protocol
//! definition with recursion blocks inside choice branches.
//! 
//! The protocol demonstrates a client-server interaction where:
//! 1. The client can choose between different types of sessions
//! 2. Each session type has its own recursive interaction pattern

use sessrums_macro::mpst;

// Define a protocol with recursion within choice branches where:
// - Client can choose between a query session or an update session
// - Each session type has its own recursive interaction pattern
mpst! {
    protocol RecursionInChoiceProtocol {
        // Define the participants
        participant Client;
        participant Server;
        participant Database;

        // Client chooses the type of session
        choice at Client {
            // Option 1: Query session with its own recursion
            option QuerySession {
                Client -> Server: String;  // "Start query session"
                Server -> Database: String;  // Initialize query session
                
                // Recursion for the query session
                rec QueryLoop {
                    // Client sends a query
                    Client -> Server: String;  // Query string
                    Server -> Database: String;  // Forward query
                    
                    // Database processes the query and responds
                    Database -> Server: String;  // Query results
                    Server -> Client: String;  // Forward results
                    
                    // Client decides whether to send another query or end the session
                    choice at Client {
                        option ContinueQuery {
                            // Continue the query recursion
                            continue QueryLoop;
                        }
                        
                        option EndQuery {
                            // End the query session
                            Client -> Server: String;  // "End query session"
                            Server -> Database: String;  // Close query session
                        }
                    }
                }
            }
            
            // Option 2: Update session with its own recursion
            option UpdateSession {
                Client -> Server: String;  // "Start update session"
                Server -> Database: String;  // Initialize update session
                
                // Recursion for the update session
                rec UpdateLoop {
                    // Client sends an update
                    Client -> Server: String;  // Update data
                    Server -> Database: String;  // Forward update
                    
                    // Database processes the update and responds
                    Database -> Server: String;  // Update status
                    Server -> Client: String;  // Forward status
                    
                    // Client decides whether to send another update or end the session
                    choice at Client {
                        option ContinueUpdate {
                            // Continue the update recursion
                            continue UpdateLoop;
                        }
                        
                        option EndUpdate {
                            // End the update session
                            Client -> Server: String;  // "End update session"
                            Server -> Database: String;  // Close update session
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
    let _protocol = RecursionInChoiceProtocol::new();
}