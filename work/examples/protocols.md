#Protocol examples

Okay, here are five simple, non-recursive session type protocols sketched out in Rust. These examples demonstrate the basic concepts (`Send`, `Recv`, `Choose`, `Offer`, `End`) and how the type of the channel changes after each operation.

These examples use sketched session type definitions and channel operations. They won't perform real communication but illustrate the compile-time protocol enforcement.

**Sketch Setup (Common for all examples)**


```rust
use std::marker::PhantomData;

// --- Sketch of Session Type Primitives ---
// (These would be the core types of your library)

trait Protocol: Send + 'static {} // Marker trait

// Send<T, P>: Send value of type T, then continue protocol P
struct Send<T: Send + 'static, P: Protocol>(PhantomData<(T, P)>);
impl<T: Send + 'static, P: Protocol> Protocol for Send<T, P> {}

// Recv<T, P>: Receive value of type T, then continue protocol P
struct Recv<T: Send + 'static, P: Protocol>(PhantomData<(T, P)>);
impl<T: Send + 'static, P: Protocol> Protocol for Recv<T, P> {}

// Choose<P1, P2>: This endpoint chooses to continue with P1 or P2
struct Choose<P1: Protocol, P2: Protocol>(PhantomData<(P1, P2)>);
impl<P1: Protocol, P2: Protocol> Protocol for Choose<P1, P2> {}

// Offer<P1, P2>: This endpoint offers the peer a choice between P1 or P2
struct Offer<P1: Protocol, P2: Protocol>(PhantomData<(P1, P2)>);
impl<P1: Protocol, P2: Protocol> Protocol for Offer<P1, P2> {}

// End: Session terminates successfully
struct End;
impl Protocol for End {}

// --- Sketch of a Channel Type ---
// Represents one endpoint of a session-typed communication channel.
// The `IO` parameter would represent the underlying transport (TCP, channel, etc.)
// For this sketch, we'll use a unit type `()` for IO.
struct Chan<P: Protocol, IO = ()> {
    io: IO, // Placeholder for actual IO handle/state
    _phantom_p: PhantomData<P>,
}

// Helper to create a pair of connected channels (conceptual)
// In reality, this would involve creating actual OS pipes, channels, or sockets.
fn session_channel<P: Protocol, IO1, IO2>(io1: IO1, io2: IO2) -> (Chan<P, IO1>, Chan<P::Dual, IO2>)
where
    P: Protocol,
    P::Dual: Protocol,
    // Define Dual types (a real library needs this) - SKETCHED HERE
    <P as Protocol>::Dual: Protocol
{
     // This requires defining the Dual trait properly. Sketching requires assuming duality exists.
     // We'll manually define the types for the examples for now.
     let client_chan = Chan { io: io1, _phantom_p: PhantomData };
     let server_chan = Chan { io: io2, _phantom_p: PhantomData }; // Type P::Dual assigned manually below
     (client_chan, server_chan)
}

// --- Sketch of Channel Operations (async placeholders) ---
// These functions consume the channel and return a new channel with the updated type.
// WARNING: These use dummy implementations and unsafe code purely for sketching purposes.

impl<T: Send + 'static, P: Protocol, IO> Chan<Send<T, P>, IO> {
    async fn send(self, _value: T) -> Chan<P, IO> {
        println!("(Sketch) Sending value...");
        // Real impl: send over self.io
        Chan { io: self.io, _phantom_p: PhantomData }
    }
}

impl<T: Send + 'static, P: Protocol, IO> Chan<Recv<T, P>, IO> {
    async fn recv(self) -> (T, Chan<P, IO>) {
        println!("(Sketch) Receiving value...");
        // Real impl: receive from self.io.
        // UNSAFE: Creating a dummy value just for the sketch to compile. DO NOT DO THIS IN REAL CODE.
        let dummy_value = unsafe { std::mem::zeroed() };
        (dummy_value, Chan { io: self.io, _phantom_p: PhantomData })
    }
}

// Represents the outcome of an offer
enum Branch<L, R> { Left(L), Right(R) }

impl<P1: Protocol, P2: Protocol, IO> Chan<Choose<P1, P2>, IO> {
    async fn choose_left(self) -> Chan<P1, IO> {
        println!("(Sketch) Choosing left branch...");
        // Real impl: send choice signal (e.g., 0) over self.io
        Chan { io: self.io, _phantom_p: PhantomData }
    }
    async fn choose_right(self) -> Chan<P2, IO> {
        println!("(Sketch) Choosing right branch...");
        // Real impl: send choice signal (e.g., 1) over self.io
        Chan { io: self.io, _phantom_p: PhantomData }
    }
}

impl<P1: Protocol, P2: Protocol, IO> Chan<Offer<P1, P2>, IO> {
    async fn offer(self) -> Branch<Chan<P1, IO>, Chan<P2, IO>> {
        println!("(Sketch) Offering choice, assuming peer chose Left...");
        // Real impl: receive choice signal from self.io
        // For sketch, arbitrarily return Left.
        Branch::Left(Chan { io: self.io, _phantom_p: PhantomData })
    }
}

impl<IO> Chan<End, IO> {
    fn close(self) {
        println!("(Sketch) Closing channel.");
        // Real impl: close self.io, cleanup resources
    }
}

// --- Sketching Duality (Manual for Examples) ---
// A real library would use a trait: trait Protocol { type Dual: Protocol; }
type DualOf<P> = <P as Protocol>::Dual; // Placeholder syntax

// Manually define Dual relationships for the protocols below
trait HasDual: Protocol { type Dual: Protocol; }
impl<T: Send + 'static, P: Protocol + HasDual> HasDual for Send<T, P> { type Dual = Recv<T, P::Dual>; }
impl<T: Send + 'static, P: Protocol + HasDual> HasDual for Recv<T, P> { type Dual = Send<T, P::Dual>; }
impl<P1: Protocol + HasDual, P2: Protocol + HasDual> HasDual for Choose<P1, P2> { type Dual = Offer<P1::Dual, P2::Dual>; }
impl<P1: Protocol + HasDual, P2: Protocol + HasDual> HasDual for Offer<P1, P2> { type Dual = Choose<P1::Dual, P2::Dual>; }
impl HasDual for End { type Dual = End; }

// Example main function structure (needed for async)
#[tokio::main] // Or use #[async_std::main] etc.
async fn main() {
     println!("--- Protocol 1 ---");
     protocol_1().await;
     println!("\n--- Protocol 2 ---");
     protocol_2().await;
     println!("\n--- Protocol 3 ---");
     protocol_3().await;
     println!("\n--- Protocol 4 ---");
     protocol_4().await;
     println!("\n--- Protocol 5 ---");
     protocol_5().await;
}

// --- Example Protocols ---

// Protocol 1: Simple Send/Recv Ping-Pong
type PingPongClient = Send<i32, Recv<String, End>>;
// type PingPongServer = <PingPongClient as HasDual>::Dual; // Equivalent to Recv<i32, Send<String, End>>
type PingPongServer = Recv<i32, Send<String, End>>;

async fn protocol_1() {
    // Conceptual setup: Create connected channels with dual types
    let (client_chan, server_chan): (Chan<PingPongClient>, Chan<PingPongServer>) = (
        Chan { io: (), _phantom_p: PhantomData },
        Chan { io: (), _phantom_p: PhantomData },
    );

    let client_task = async {
        println!("Client: Sending i32...");
        let chan = client_chan.send(42).await; // Type is now Chan<Recv<String, End>>
        println!("Client: Receiving String...");
        let (msg, chan) = chan.recv().await; // Type is now Chan<End>
        println!("Client: Received '{}'", msg); // Note: msg will be dummy value in sketch
        chan.close();
        println!("Client: Done.");
    };

    let server_task = async {
        println!("Server: Receiving i32...");
        let (num, chan) = server_chan.recv().await; // Type is now Chan<Send<String, End>>
        println!("Server: Received {}, Sending String...", num); // Note: num will be dummy value
        let chan = chan.send("Hello!".to_string()).await; // Type is now Chan<End>
        chan.close();
        println!("Server: Done.");
    };

    // In a real scenario, you'd spawn these onto an executor
    tokio::join!(client_task, server_task);
}

// Protocol 2: Request/Response
type ReqResClient = Send<String, Recv<bool, End>>;
// type ReqResServer = <ReqResClient as HasDual>::Dual; // Equivalent to Recv<String, Send<bool, End>>
type ReqResServer = Recv<String, Send<bool, End>>;

async fn protocol_2() {
     let (client_chan, server_chan): (Chan<ReqResClient>, Chan<ReqResServer>) = (
        Chan { io: (), _phantom_p: PhantomData },
        Chan { io: (), _phantom_p: PhantomData },
    );

    let client_task = async {
        println!("Client: Sending request...");
        let chan = client_chan.send("GetStatus".to_string()).await; // Type: Chan<Recv<bool, End>>
        println!("Client: Receiving response...");
        let (status, chan) = chan.recv().await; // Type: Chan<End>
        println!("Client: Received status {}", status); // Note: status is dummy
        chan.close();
    };

     let server_task = async {
        println!("Server: Receiving request...");
        let (req, chan) = server_chan.recv().await; // Type: Chan<Send<bool, End>>
        println!("Server: Received request '{}', sending response...", req); // Note: req is dummy
        let chan = chan.send(true).await; // Type: Chan<End>
        chan.close();
    };

     tokio::join!(client_task, server_task);
}


// Protocol 3: Simple Choice (Send u64 or Recv f32)
type ChoiceClient = Choose<Send<u64, End>, Recv<f32, End>>;
// type ChoiceServer = <ChoiceClient as HasDual>::Dual; // Equivalent to Offer<Recv<u64, End>, Send<f32, End>>
type ChoiceServer = Offer<Recv<u64, End>, Send<f32, End>>;


async fn protocol_3() {
    let (client_chan, server_chan): (Chan<ChoiceClient>, Chan<ChoiceServer>) = (
        Chan { io: (), _phantom_p: PhantomData },
        Chan { io: (), _phantom_p: PhantomData },
    );

    let client_task = async {
        println!("Client: Choosing to send u64 (left branch)...");
        let chan = client_chan.choose_left().await; // Type: Chan<Send<u64, End>>
        println!("Client: Sending 1000...");
        let chan = chan.send(1000u64).await; // Type: Chan<End>
        chan.close();
    };

    let server_task = async {
        println!("Server: Offering choice...");
        match server_chan.offer().await { // Type Offer<Recv<u64, End>, Send<f32, End>>
            Branch::Left(chan) => { // Type: Chan<Recv<u64, End>>
                println!("Server: Peer chose Left. Receiving u64...");
                let (val, chan) = chan.recv().await; // Type: Chan<End>
                println!("Server: Received {}", val); // Note: val is dummy
                chan.close();
            }
            Branch::Right(chan) => { // Type: Chan<Send<f32, End>>
                println!("Server: Peer chose Right. Sending f32...");
                 let chan = chan.send(3.14f32).await; // Type: Chan<End>
                 chan.close();
            }
        }
    };

    tokio::join!(client_task, server_task);
}


// Protocol 4: Simple Authentication
type AuthClient = Send<String, Send<String, Recv<u128, End>>>;
// type AuthServer = <AuthClient as HasDual>::Dual; // Recv<String, Recv<String, Send<u128, End>>>
type AuthServer = Recv<String, Recv<String, Send<u128, End>>>;

async fn protocol_4() {
    let (client_chan, server_chan): (Chan<AuthClient>, Chan<AuthServer>) = (
        Chan { io: (), _phantom_p: PhantomData },
        Chan { io: (), _phantom_p: PhantomData },
    );

     let client_task = async {
        println!("Client: Sending username...");
        let chan = client_chan.send("user".to_string()).await; // Type: Chan<Send<String, Recv<u128, End>>>
        println!("Client: Sending password...");
        let chan = chan.send("pass".to_string()).await; // Type: Chan<Recv<u128, End>>
        println!("Client: Receiving token...");
        let (token, chan) = chan.recv().await; // Type: Chan<End>
        println!("Client: Received token {}", token); // Note: token is dummy
        chan.close();
    };

     let server_task = async {
        println!("Server: Receiving username...");
        let (user, chan) = server_chan.recv().await; // Type: Chan<Recv<String, Send<u128, End>>>
        println!("Server: Received user '{}'. Receiving password...", user); // Note: user is dummy
        let (pass, chan) = chan.recv().await; // Type: Chan<Send<u128, End>>
        println!("Server: Received pass '{}'. Sending token...", pass); // Note: pass is dummy
        let token = 12345678901234567890u128; // Dummy token
        let chan = chan.send(token).await; // Type: Chan<End>
        chan.close();
    };

    tokio::join!(client_task, server_task);
}

// Protocol 5: Data Query with Options (Server Chooses Response Type)
type QueryClient = Send<String, Offer<Recv<Vec<u8>, End>, Recv<i16, End>>>;
// type QueryServer = <QueryClient as HasDual>::Dual; // Recv<String, Choose<Send<Vec<u8>, End>, Send<i16, End>>>
type QueryServer = Recv<String, Choose<Send<Vec<u8>, End>, Send<i16, End>>>;

async fn protocol_5() {
     let (client_chan, server_chan): (Chan<QueryClient>, Chan<QueryServer>) = (
        Chan { io: (), _phantom_p: PhantomData },
        Chan { io: (), _phantom_p: PhantomData },
    );

     let client_task = async {
        println!("Client: Sending query...");
        let chan = client_chan.send("GetData:Item123".to_string()).await; // Type: Chan<Offer<Recv<Vec<u8>, End>, Recv<i16, End>>>
        println!("Client: Waiting for offer...");
        match chan.offer().await {
            Branch::Left(chan) => { // Type: Chan<Recv<Vec<u8>, End>>
                println!("Client: Server offered data. Receiving data...");
                 let (data, chan) = chan.recv().await; // Type: Chan<End>
                 println!("Client: Received {} bytes", data.len()); // Note: data is dummy (empty vec)
                 chan.close();
            }
            Branch::Right(chan) => { // Type: Chan<Recv<i16, End>>
                println!("Client: Server offered error code. Receiving code...");
                let (code, chan) = chan.recv().await; // Type: Chan<End>
                println!("Client: Received error code {}", code); // Note: code is dummy
                chan.close();
            }
        }
    };

     let server_task = async {
        println!("Server: Receiving query...");
        let (query, chan) = server_chan.recv().await; // Type: Chan<Choose<Send<Vec<u8>, End>, Send<i16, End>>>
        println!("Server: Received query '{}'.", query); // Note: query is dummy
        // Server decides to send data (left branch)
        println!("Server: Choosing to send data...");
        let chan = chan.choose_left().await; // Type: Chan<Send<Vec<u8>, End>>
        let data = vec![1, 2, 3, 4];
        println!("Server: Sending data...");
        let chan = chan.send(data).await; // Type: Chan<End>
        chan.close();
    };

     tokio::join!(client_task, server_task);
}
```

These examples demonstrate how the Rust type system, combined with the sketched session types, tracks the state of the communication protocol. Any attempt to perform an operation not allowed by the current type (e.g., trying to `send` on a `Chan<Recv<...>>` or `close` a `Chan<Send<...>>`) would result in a compile-time error in a fully implemented library.
