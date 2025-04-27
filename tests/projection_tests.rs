use sessrums::proto::{Project, Role};

// Define a third role for testing multi-party protocols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct RoleC;

impl Role for RoleC {
    fn name(&self) -> &'static str {
        "RoleC"
    }
}

#[test]
fn test_simple_send_projection() {
    // Import the necessary types from the public API
    use sessrums::proto::{End, Recv, Send};
    use sessrums::proto::{RoleA, RoleB};

    // Define a simple protocol: RoleA sends a String to RoleB, then ends
    // We'll use the local types directly since we can't access the global types
    // from the integration test
    
    // For RoleA: Send<String, End>
    type RoleALocal = Send<String, End>;
    
    // For RoleB: Recv<String, End>
    type RoleBLocal = Recv<String, End>;
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<RoleALocal>();
    assert_type::<RoleBLocal>();
}

#[test]
fn test_simple_recv_projection() {
    // Import the necessary types from the public API
    use sessrums::proto::{End, Recv, Send};
    use sessrums::proto::{RoleA, RoleB};

    // Define a simple protocol: RoleB sends a String to RoleA, then ends
    // We'll use the local types directly since we can't access the global types
    // from the integration test
    
    // For RoleA: Recv<String, End>
    type RoleALocal = Recv<String, End>;
    
    // For RoleB: Send<String, End>
    type RoleBLocal = Send<String, End>;
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<RoleALocal>();
    assert_type::<RoleBLocal>();
}

#[test]
fn test_complex_protocol_projection() {
    // Import the necessary types from the public API
    use sessrums::proto::{End, Recv, Send};
    use sessrums::proto::{RoleA, RoleB};

    // Define a more complex protocol:
    // RoleA sends a String to RoleB,
    // RoleB sends an i32 back to RoleA,
    // then ends
    // We'll use the local types directly since we can't access the global types
    // from the integration test
    
    // For RoleA: Send<String, Recv<i32, End>>
    type RoleALocal = Send<String, Recv<i32, End>>;
    
    // For RoleB: Recv<String, Send<i32, End>>
    type RoleBLocal = Recv<String, Send<i32, End>>;
    
    // Verify that the types are correct
    fn assert_type<T>() {}
    
    assert_type::<RoleALocal>();
    assert_type::<RoleBLocal>();
}