//! Test for a comprehensive protocol combining multiple complex features.
//!
//! This test verifies that the macro can correctly parse and process a protocol
//! definition that combines multiple complex features including:
//! - Multiple participants
//! - Nested recursion blocks
//! - Choice blocks with multiple options
//! - Recursion within choice branches
//! - Complex message types
//! 
//! The protocol demonstrates a sophisticated distributed system interaction
//! with multiple roles and complex communication patterns.

use sessrums_macro::mpst;

// Define custom types for the protocol
struct AuthRequest {
    username: String,
    password: String,
}

struct AuthResponse {
    success: bool,
    token: Option<String>,
    error: Option<String>,
}

struct QueryRequest {
    query: String,
    parameters: Vec<String>,
}

struct QueryResponse {
    results: Vec<String>,
    metadata: String,
}

struct UpdateRequest {
    id: u32,
    data: String,
}

struct UpdateResponse {
    success: bool,
    affected_rows: u32,
}

struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

// Define a comprehensive protocol that combines multiple complex features
mpst! {
    protocol ComplexDistributedSystem {
        // Define the participants
        participant Client;
        participant AuthService;
        participant QueryService;
        participant DataService;
        participant Logger;

        // Authentication phase
        Client -> AuthService: AuthRequest;
        AuthService -> Logger: LogEntry;  // Log authentication attempt
        
        // Authentication result with choice
        choice at AuthService {
            // Authentication success path
            option AuthSuccess {
                AuthService -> Client: AuthResponse;  // Success response with token
                AuthService -> Logger: LogEntry;  // Log successful authentication
                
                // Main session loop after successful authentication
                rec SessionLoop {
                    // Client chooses the operation type
                    choice at Client {
                        // Query operation with its own recursion
                        option QueryOperation {
                            Client -> QueryService: QueryRequest;
                            QueryService -> Logger: LogEntry;  // Log query request
                            
                            // Query processing loop
                            rec QueryLoop {
                                QueryService -> DataService: String;  // Data request
                                DataService -> QueryService: String;  // Data response
                                
                                // QueryService decides if more data is needed
                                choice at QueryService {
                                    option NeedMoreData {
                                        // Continue querying for more data
                                        continue QueryLoop;
                                    }
                                    
                                    option QueryComplete {
                                        // Query is complete, return results to client
                                        QueryService -> Client: QueryResponse;
                                        QueryService -> Logger: LogEntry;  // Log query completion
                                        
                                        // Return to the main session loop
                                        continue SessionLoop;
                                    }
                                }
                            }
                        }
                        
                        // Update operation with its own recursion
                        option UpdateOperation {
                            Client -> DataService: UpdateRequest;
                            DataService -> Logger: LogEntry;  // Log update request
                            
                            // Update processing
                            DataService -> Client: UpdateResponse;
                            DataService -> Logger: LogEntry;  // Log update completion
                            
                            // Return to the main session loop
                            continue SessionLoop;
                        }
                        
                        // End session
                        option EndSession {
                            Client -> AuthService: String;  // "End session"
                            AuthService -> Logger: LogEntry;  // Log session end
                        }
                    }
                }
            }
            
            // Authentication failure path
            option AuthFailure {
                AuthService -> Client: AuthResponse;  // Failure response with error
                AuthService -> Logger: LogEntry;  // Log failed authentication
            }
        }
    }
}

fn main() {
    // This is just a compile-time test, so we don't need to do anything here
    // The test passes if the macro expansion compiles successfully
    let _protocol = ComplexDistributedSystem::new();
}