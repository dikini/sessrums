use sessrums_types::{
    messages::{PingMsg, PongMsg},
    session_types::{End, Send, Receive, Session},
    transport::MockChannelEnd,
    error::SessionError,
};

// Type aliases to make protocol steps more readable
type ClientProtocol = Send<PingMsg, Receive<PongMsg, End>>;
type ServerProtocol = Receive<PingMsg, Send<PongMsg, End>>;

#[test]
fn test_successful_ping_pong() -> Result<(), SessionError> {
    // Set up mock transport
    let (client_chan, server_chan) = MockChannelEnd::new_pair();

    // Initialize sessions
    let client = Session::<ClientProtocol, _>::new(client_chan);
    let server = Session::<ServerProtocol, _>::new(server_chan);

    // Client sends ping
    let ping = PingMsg { seq: Some(42) };
    let client = client.send(ping)?;

    // Server receives ping
    let (received_ping, server) = server.receive()?;
    assert_eq!(received_ping.seq, Some(42));

    // Server sends pong
    let pong = PongMsg {
        seq: received_ping.seq,
        timestamp: 12345,
    };
    let server = server.send(pong)?;

    // Client receives pong
    let (received_pong, client) = client.receive()?;
    assert_eq!(received_pong.seq, Some(42));
    assert_eq!(received_pong.timestamp, 12345);

    // Both ends close properly
    let _client_chan = client.close();
    let _server_chan = server.close();

    Ok(())
}

#[test]
fn test_session_type_safety() {
    let (chan, _) = MockChannelEnd::new_pair();
    let session = Session::<End, _>::new(chan);

    // Uncommenting any of these lines should cause a compile error:
    
    // Cannot send on End state:
    // session.send(PingMsg { seq: None });
    
    // Cannot receive on End state:
    // session.receive::<PongMsg>();
    
    // Can only close:
    let _ = session.close();
}

#[test]
fn test_transport_error_propagation() {
    // Create a single channel end that will fail on receive
    let (client_chan, _) = MockChannelEnd::new_pair();
    
    let client = Session::<Receive<PingMsg, End>, _>::new(client_chan);
    
    // Attempting to receive should fail with UnexpectedClose
    let result = client.receive();
    assert!(matches!(result, Err(SessionError::UnexpectedClose)));
}

#[test]
fn test_multiple_ping_pong_sequences() -> Result<(), SessionError> {
    let (client_chan, server_chan) = MockChannelEnd::new_pair();

    // Define a longer protocol with multiple ping-pongs
    type ClientLongProtocol = Send<PingMsg, Receive<PongMsg, Send<PingMsg, Receive<PongMsg, End>>>>;
    type ServerLongProtocol = Receive<PingMsg, Send<PongMsg, Receive<PingMsg, Send<PongMsg, End>>>>;

    let client = Session::<ClientLongProtocol, _>::new(client_chan);
    let server = Session::<ServerLongProtocol, _>::new(server_chan);

    // First ping-pong
    let client = client.send(PingMsg { seq: Some(1) })?;
    let (ping1, server) = server.receive()?;
    let server = server.send(PongMsg { seq: ping1.seq, timestamp: 1 })?;
    let (pong1, client) = client.receive()?;
    assert_eq!(pong1.seq, Some(1));

    // Second ping-pong
    let client = client.send(PingMsg { seq: Some(2) })?;
    let (ping2, server) = server.receive()?;
    let server = server.send(PongMsg { seq: ping2.seq, timestamp: 2 })?;
    let (pong2, client) = client.receive()?;
    assert_eq!(pong2.seq, Some(2));

    // Close both ends
    let _client_chan = client.close();
    let _server_chan = server.close();

    Ok(())
}