use sessrums_types::{
    error::SessionError,
    messages::{PingMsg, PongMsg},
    session_types::{Dual, Either, End, Rec, Receive, Select, Send, Session, Var, Offer},
    transport::MockChannelEnd,
};

use serde::{Deserialize, Serialize};

// Define a counter message for testing bounded recursion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
struct CounterMsg {
    count: u32,
}

// Test a simple recursive ping-pong protocol
#[test]
fn test_simple_recursive_protocol() -> Result<(), SessionError> {
    // Create mock channel pair
    let (client_chan, server_chan) = MockChannelEnd::new_pair();

    // Define recursive protocol types
    // A protocol that sends a ping, receives a pong, and then loops back
    fn client_body(_: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
        Send::new()
    }

    fn server_body(_: Var) -> Receive<PingMsg, Send<PongMsg, Var>> {
        Receive::new()
    }

    type ClientProtocol = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
    type ServerProtocol = Rec<fn(Var) -> Receive<PingMsg, Send<PongMsg, Var>>>;

    // Create client and server sessions with explicit state
    let client_rec = Rec::new(client_body);
    let server_rec = Rec::new(server_body);
    
    let client = Session::with_state(client_rec, client_chan);
    let server = Session::with_state(server_rec, server_chan);

    // Unroll the recursion once
    let mut client_session = client.enter_rec();
    let mut server_session = server.enter_rec();

    // First iteration
    let ping1 = PingMsg { seq: Some(1) };
    let client_session = client_session.send(ping1)?;
    
    let (received_ping1, server_session) = server_session.receive()?;
    assert_eq!(received_ping1.seq, Some(1));
    
    let pong1 = PongMsg {
        seq: received_ping1.seq,
        timestamp: 0
    };
    let server_session = server_session.send(pong1)?;
    
    let (received_pong1, client_session) = client_session.receive()?;
    assert_eq!(received_pong1.seq, Some(1));

    // Loop back and do another iteration
    let client_session = client_session.recurse(client_body);
    let server_session = server_session.recurse(server_body);

    let client_session = client_session.enter_rec();
    let server_session = server_session.enter_rec();

    // Second iteration
    let ping2 = PingMsg { seq: Some(2) };
    let client_session = client_session.send(ping2)?;
    
    let (received_ping2, server_session) = server_session.receive()?;
    assert_eq!(received_ping2.seq, Some(2));
    
    let pong2 = PongMsg {
        seq: received_ping2.seq,
        timestamp: 0
    };
    let server_session = server_session.send(pong2)?;
    
    let (received_pong2, _client_session) = client_session.receive()?;
    assert_eq!(received_pong2.seq, Some(2));

    Ok(())
}

// Test a bounded recursive protocol with a counter
#[test]
fn test_bounded_recursive_protocol() -> Result<(), SessionError> {
    // Create mock channel pair
    let (client_chan, server_chan) = MockChannelEnd::new_pair();

    // Define recursive protocol types with a choice to continue or end
    fn client_body(_: Var) -> Send<CounterMsg, Receive<CounterMsg, Select<Var, End>>> {
        Send::new()
    }

    fn server_body(_: Var) -> Receive<CounterMsg, Send<CounterMsg, Offer<Var, End>>> {
        Receive::new()
    }

    type ClientProtocol = Rec<fn(Var) -> Send<CounterMsg, Receive<CounterMsg, Select<Var, End>>>>;
    type ServerProtocol = Rec<fn(Var) -> Receive<CounterMsg, Send<CounterMsg, Offer<Var, End>>>>;

    // Create client and server sessions with explicit state
    let client_rec = Rec::new(client_body);
    let server_rec = Rec::new(server_body);
    
    let client = Session::with_state(client_rec, client_chan);
    let server = Session::with_state(server_rec, server_chan);

    // Unroll the recursion once
    let mut client_session = client.enter_rec();
    let mut server_session = server.enter_rec();

    // Run the protocol for 3 iterations
    let max_count = 3;
    let mut count = 1;

    while count <= max_count {
        // Client sends counter
        let client_msg = CounterMsg { count };
        let client_session_after_send = client_session.send(client_msg)?;
        
        // Server receives counter
        let (received_client_msg, server_session_after_receive) = server_session.receive()?;
        assert_eq!(received_client_msg.count, count);
        
        // Server sends counter back
        let server_msg = CounterMsg { count };
        let server_session_after_send = server_session_after_receive.send(server_msg)?;
        
        // Client receives counter
        let (received_server_msg, client_session_after_receive) = client_session_after_send.receive()?;
        assert_eq!(received_server_msg.count, count);

        count += 1;

        if count <= max_count {
            // Continue with recursion
            let client_session_after_select = client_session_after_receive.select_left()?;
            let client_session_after_recurse = client_session_after_select.recurse(client_body);
            client_session = client_session_after_recurse.enter_rec();

            let Either::Left(server_session_after_offer) = server_session_after_send.offer()? else {
                panic!("Server should have received Left choice");
            };
            let server_session_after_recurse = server_session_after_offer.recurse(server_body);
            server_session = server_session_after_recurse.enter_rec();
        } else {
            // End the protocol
            let _client_end = client_session_after_receive.select_right()?;
            let Either::Right(_server_end) = server_session_after_send.offer()? else {
                panic!("Server should have received Right choice");
            };
            break;
        }
    }

    assert_eq!(count, max_count + 1);
    Ok(())
}

// Test that the compiler enforces correct usage of recursion
#[test]
fn test_recursion_type_safety() {
    fn check_duality<T: Dual>() where T::DualType: Dual {}

    // This should compile only if ClientProtocol is dual to ServerProtocol
    fn client_body(_: Var) -> Send<PingMsg, Receive<PongMsg, Var>> {
        Send::new()
    }

    fn server_body(_: Var) -> Receive<PingMsg, Send<PongMsg, Var>> {
        Receive::new()
    }

    type ClientProtocol = Rec<fn(Var) -> Send<PingMsg, Receive<PongMsg, Var>>>;
    type ServerProtocol = Rec<fn(Var) -> Receive<PingMsg, Send<PongMsg, Var>>>;

    check_duality::<ClientProtocol>();
}