//! Binary session types implementation.
//! 
//! This module provides the core typestate machinery for binary (two-party)
//! session types, including Send, Receive, and End states.

use std::marker::PhantomData;
use serde::{Serialize, de::DeserializeOwned};
use crate::{
    transport::Transport,
    error::SessionError,
};

#[derive(Debug, Default)]
pub struct End;

#[derive(Debug, Default)]
pub struct Send<M, NextP>(PhantomData<(M, NextP)>);

#[derive(Debug, Default)]
pub struct Receive<M, NextP>(PhantomData<(M, NextP)>);
/// A session with current protocol state S using transport T.
#[derive(Debug)]
pub struct Session<S, T: Transport> {
    state: S,
    channel: T,
}

// Session constructors and general methods
impl<S, T: Transport> Session<S, T> {
    /// Create a new session with the given transport channel.
    pub fn new(channel: T) -> Self 
    where S: Default {
        Session {
            state: S::default(),
            channel,
        }
    }

    /// Get the underlying transport channel, consuming the session.
    pub fn into_transport(self) -> T {
        self.channel
    }
}

// Implementation for End state
impl<T: Transport> Session<End, T> {
    /// Close the session, returning the transport channel.
    pub fn close(self) -> T {
        self.channel
    }
}

// Implementation for Send state
impl<M, NextP, T: Transport> Session<Send<M, NextP>, T> 
where
    M: Serialize + 'static,
    NextP: Default,
{
    /// Send a message and transition to the next protocol state.
    pub fn send(mut self, message: M) -> Result<Session<NextP, T>, SessionError> {
        self.channel.send_payload(&message)?;
        
        Ok(Session {
            state: NextP::default(),
            channel: self.channel,
        })
    }
}

// Implementation for Receive state
impl<M, NextP, T: Transport> Session<Receive<M, NextP>, T>
where
    M: DeserializeOwned + 'static,
    NextP: Default,
{
    /// Receive a message and transition to the next protocol state.
    pub fn receive(mut self) -> Result<(M, Session<NextP, T>), SessionError> {
        let message = self.channel.receive_payload()?;
        
        Ok((message, Session {
            state: NextP::default(),
            channel: self.channel,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        messages::{PingMsg, PongMsg},
        transport::MockChannelEnd,
    };

    #[test]
    fn test_ping_pong_protocol() -> Result<(), SessionError> {
        // Create mock channel pair
        let (client_chan, server_chan) = MockChannelEnd::new_pair();
    
        // Define protocol steps using type aliases
        type PingStep = Send<PingMsg, Receive<PongMsg, End>>;
        type PongStep = Receive<PingMsg, Send<PongMsg, End>>;
    
        // Create client and server sessions
        let client = Session::<PingStep, _>::new(client_chan);
        let server = Session::<PongStep, _>::new(server_chan);
    
        // Run the protocol
        let ping = PingMsg { seq: Some(1) };
        
        // Client sends ping
        let client = client.send(ping)?;
        
        // Server receives ping and sends pong
        let (received_ping, server) = server.receive()?;
        assert_eq!(received_ping.seq, Some(1));
        
        let pong = PongMsg { 
            seq: received_ping.seq, 
            timestamp: 0 
        };
        let server = server.send(pong)?;
        
        // Client receives pong
        let (received_pong, client) = client.receive()?;
        assert_eq!(received_pong.seq, Some(1));
        assert_eq!(received_pong.timestamp, 0);
        
        // Close both sessions
        let _client_chan = client.close();
        let _server_chan = server.close();
        
        Ok(())
    }

    #[test]
    fn test_session_type_safety() {
        let (client_chan, _) = MockChannelEnd::new_pair();
        let session = Session::<End, _>::new(client_chan);
        
        // Following line would not compile:
        // let _ = session.send(PingMsg { seq: None });
        
        // But we can close an End session:
        let _ = session.close();
    }
}