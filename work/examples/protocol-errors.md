# Protocol errors examples

Okay, here are four examples of protocols exhibiting common concurrency errors (like deadlock and protocol mismatch) that a well-formed session type system should prevent at compile time.

These examples use the same sketched Rust session type definitions from the previous answer. The key idea is that the type system, primarily through checking **duality** and **type states**, would make these erroneous protocols impossible to compile or run.

**Sketched Setup (Assumed from previous example)**


```rust
// Assume the sketched types: Protocol, Send, Recv, Choose, Offer, End, Chan,
// session_channel, HasDual, etc., are defined as before.
// ... (previous sketch code) ...
use std::marker::PhantomData;
trait Protocol: Send + 'static {}
struct Send<T: Send + 'static, P: Protocol>(PhantomData<(T, P)>); impl<T: Send + 'static, P: Protocol> Protocol for Send<T, P> {}
struct Recv<T: Send + 'static, P: Protocol>(PhantomData<(T, P)>); impl<T: Send + 'static, P: Protocol> Protocol for Recv<T, P> {}
struct End; impl Protocol for End {}
struct Chan<P: Protocol, IO = ()> { io: IO, _phantom_p: PhantomData<P> }
fn session_channel<P: Protocol + HasDual, IO1, IO2>(io1: IO1, io2: IO2) -> (Chan<P, IO1>, Chan<P::Dual, IO2>) {
     (Chan { io: io1, _phantom_p: PhantomData }, Chan { io: io2, _phantom_p: PhantomData })
}
trait HasDual: Protocol { type Dual: Protocol; }
impl<T: Send + 'static, P: Protocol + HasDual> HasDual for Send<T, P> { type Dual = Recv<T, P::Dual>; }
impl<T: Send + 'static, P: Protocol + HasDual> HasDual for Recv<T, P> { type Dual = Send<T, P::Dual>; }
impl HasDual for End { type Dual = End; }
// Dummy ops for illustration
impl<T: Send + 'static, P: Protocol, IO> Chan<Recv<T, P>, IO> { async fn recv(self) -> (T, Chan<P, IO>) { (unsafe { std::mem::zeroed() }, Chan { io: self.io, _phantom_p: PhantomData }) } }
impl<T: Send + 'static, P: Protocol, IO> Chan<Send<T, P>, IO> { async fn send(self, _value: T) -> Chan<P, IO> { Chan { io: self.io, _phantom_p: PhantomData } } }
impl<IO> Chan<End, IO> { fn close(self) {} }

// --- Error Examples ---

// These functions show protocols that SHOULD NOT COMPILE in a real session type library.
```

**Error Example 1: Deadlock (Recv/Recv)**

- **Intent:** Both client and server try to receive an `i32` first. Neither sends.
- **Types:**
    - Client: `Recv<i32, End>`
    - Server: `Recv<i32, End>`
- **Why Session Types Prevent It:**
    - The types are not duals. The dual of `Recv<i32, End>` is `Send<i32, End>`.
    - The `session_channel` function (or any connection setup function) requires the two channel types to be duals. Trying to create `Chan<Recv<i32, End>>` and `Chan<Recv<i32, End>>` would fail the type check.

Rust

```rust
async fn error_1_recv_recv_deadlock() {
    type ClientProto = Recv<i32, End>;
    type ServerProto = Recv<i32, End>; // Problem: Not the dual of ClientProto

    // Compile-Time Error Expected Here:
    // The type checker would fail because ServerProto is not ClientProto::Dual.
    // For example, a trait bound like `where ServerProto: Eq<ClientProto::Dual>`
    // or similar mechanism in session_channel would fail.
    /*
    let (client_chan, server_chan): (Chan<ClientProto>, Chan<ServerProto>) =
         session_channel((), ()); // <-- This line would fail type checking
    */

    println!("Error 1: This code block would not compile due to non-dual types.");

    // If compilation somehow proceeded (e.g., types weren't checked at setup),
    // the following would deadlock at runtime:
    /*
    let client_task = async {
        println!("Client: Trying to receive...");
        let (_val, _chan) = client_chan.recv().await; // Blocks forever
        println!("Client: Should never reach here.");
    };
    let server_task = async {
        println!("Server: Trying to receive...");
        let (_val, _chan) = server_chan.recv().await; // Blocks forever
         println!("Server: Should never reach here.");
    };
    tokio::join!(client_task, server_task);
    */
}
```

**Error Example 2: Deadlock (Send/Send)**

- **Intent:** Both client and server try to send an `i32` first. Neither receives.
- **Types:**
    - Client: `Send<i32, End>`
    - Server: `Send<i32, End>`
- **Why Session Types Prevent It:**
    - Again, the types are not duals. The dual of `Send<i32, End>` is `Recv<i32, End>`.
    - The `session_channel` function would fail the duality type check.

Rust

```rust
async fn error_2_send_send_deadlock() {
    type ClientProto = Send<i32, End>;
    type ServerProto = Send<i32, End>; // Problem: Not the dual of ClientProto

    // Compile-Time Error Expected Here:
    // Similar to Error 1, the type checker would reject creating this pair
    // because ServerProto is not ClientProto::Dual.
    /*
    let (client_chan, server_chan): (Chan<ClientProto>, Chan<ServerProto>) =
         session_channel((), ()); // <-- This line would fail type checking
    */

    println!("Error 2: This code block would not compile due to non-dual types.");

    // If compilation somehow proceeded, the following might deadlock or error
    // at runtime depending on the underlying channel buffering:
     /*
    let client_task = async {
        println!("Client: Trying to send...");
        let _chan = client_chan.send(1).await; // Might block/error if buffer full
        println!("Client: Send potentially blocked/errored.");
    };
    let server_task = async {
        println!("Server: Trying to send...");
         let _chan = server_chan.send(2).await; // Might block/error if buffer full
        println!("Server: Send potentially blocked/errored.");
    };
    tokio::join!(client_task, server_task);
    */
}
```

**Error Example 3: Protocol Mismatch (Type)**

- **Intent:** Client sends an `i32`, but the Server expects to receive a `String`.
- **Types:**
    - Client: `Send<i32, End>`
    - Server: `Recv<String, End>`
- **Why Session Types Prevent It:**
    - These types are not duals. Duality requires the message types to match (`Send<T, P>::Dual = Recv<T, P::Dual>`). Here, `T` is `i32` on one side and `String` on the other.
    - The `session_channel` type check would fail.

Rust

```rust
async fn error_3_type_mismatch() {
    type ClientProto = Send<i32, End>;
    // Problem: Expects String, but ClientProto sends i32.
    type ServerProto = Recv<String, End>;

    // Compile-Time Error Expected Here:
    // The duality check fails because the types `i32` and `String` do not match.
    // ServerProto is not ClientProto::Dual (which should be Recv<i32, End>).
    /*
    let (client_chan, server_chan): (Chan<ClientProto>, Chan<ServerProto>) =
         session_channel((), ()); // <-- This line would fail type checking
    */

    println!("Error 3: This code block would not compile due to type mismatch in dual.");
}
```

**Error Example 4: Protocol Mismatch (Unexpected End / Dangling)**

- **Intent:** Client sends `i32` and terminates. Server receives `i32` but then expects to _send_ a `bool` (which the client isn't expecting to receive).
- **Types:**
    - Client: `Send<i32, End>`
    - Server: `Recv<i32, Send<bool, End>>`
- **Why Session Types Prevent It:**
    - These types are not duals. The continuation protocol after the initial `Send<i32>`/`Recv<i32>` pair does not match duality.
    - Client expects `End` (whose dual is `End`).
    - Server's continuation is `Send<bool, End>`. The dual of this is `Recv<bool, End>`.
    - Since `End` is not the dual of `Send<bool, End>`, the overall types `Send<i32, End>` and `Recv<i32, Send<bool, End>>` are not duals.
    - The `session_channel` type check would fail.

Rust

```rust
async fn error_4_unexpected_end() {
    type ClientProto = Send<i32, End>;
    // Problem: Server expects to Send<bool, End> after receiving i32,
    // but Client expects End after sending i32.
    type ServerProto = Recv<i32, Send<bool, End>>;

    // Compile-Time Error Expected Here:
    // Duality check fails on the continuation protocols.
    // End::Dual is End.
    // Send<bool, End>::Dual is Recv<bool, End>.
    // Since End != Recv<bool, End>, the overall types are not duals.
    /*
    let (client_chan, server_chan): (Chan<ClientProto>, Chan<ServerProto>) =
         session_channel((), ()); // <-- This line would fail type checking
    */
    println!("Error 4: This code block would not compile due to mismatched continuations (duality check failure).");

    // If compilation proceeded, runtime error would likely occur:
    /*
    let client_task = async {
        println!("Client: Sending i32...");
        let chan = client_chan.send(99).await; // Type: Chan<End>
        println!("Client: Closing channel...");
        chan.close(); // Client finishes legally according to its type.
    };
     let server_task = async {
        println!("Server: Receiving i32...");
        let (_val, chan) = server_chan.recv().await; // Type: Chan<Send<bool, End>>
        println!("Server: Sending bool...");
        // Runtime Error: Client has closed its end. This send likely fails.
        let _chan = chan.send(true).await;
         println!("Server: Should not reach here if client closed.");
    };
    tokio::join!(client_task, server_task);
    */
}

// Dummy main to allow checking the code structure
#[tokio::main]
async fn main() {
    error_1_recv_recv_deadlock().await;
    error_2_send_send_deadlock().await;
    error_3_type_mismatch().await;
    error_4_unexpected_end().await;
}
```

In summary, session types prevent these kinds of errors primarily by enforcing that the two interacting protocols are **duals** of each other _before_ communication even begins. This check ensures that sends are matched with receives of the correct type, offers are matched with choices, and both ends agree on when the session terminates (`End`).
