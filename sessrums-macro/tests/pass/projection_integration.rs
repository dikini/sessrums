use sessrums_macro::{mpst, project};
use sessrums_types::roles::{Client, Server};

mpst! {
    protocol PingPong {
        participant Client;
        participant Server;
        
        Client -> Server: String;
        Server -> Client: String;
        end;
    }
}

fn main() {
    // Project the global protocol to local protocols for each role
    type ClientProtocol = project!(PingPong, Client, String);
    type ServerProtocol = project!(PingPong, Server, String);
    
    // Use the projected protocols
    println!("Client protocol: {:?}", std::any::type_name::<ClientProtocol>());
    println!("Server protocol: {:?}", std::any::type_name::<ServerProtocol>());
}