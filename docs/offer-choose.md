# Offer and Choose Protocol Types

This document provides detailed information about the `Offer` and `Choose` protocol types in the SEZ session types library.

## Table of Contents

1. [Introduction](#introduction)
2. [Type Definitions](#type-definitions)
3. [Duality Relationship](#duality-relationship)
4. [API Methods](#api-methods)
   - [offer Method](#offer-method)
   - [choose_left Method](#choose_left-method)
   - [choose_right Method](#choose_right-method)
5. [Usage Examples](#usage-examples)
   - [Authentication Protocol](#authentication-protocol)
   - [Menu Selection Protocol](#menu-selection-protocol)
6. [Type Safety](#type-safety)
7. [Best Practices](#best-practices)

## Introduction

The `Offer<L, R>` and `Choose<L, R>` protocol types represent binary choices in communication protocols. They allow for branching behavior where the communication can proceed in one of two different ways:

- `Offer<L, R>` represents a protocol that offers a choice between continuing with protocol `L` or protocol `R`. The offering party waits for the other party to make a choice.
- `Choose<L, R>` represents a protocol that makes a choice between continuing with protocol `L` or protocol `R`. The choosing party decides which branch to take.

These types are essential for implementing protocols with conditional behavior, such as authentication protocols (success or failure paths) or menu-driven interactions.

## Type Definitions

The `Offer` and `Choose` types are defined as follows:

```rust
pub struct Offer<L, R> {
    _marker: PhantomData<(L, R)>,
}

pub struct Choose<L, R> {
    _marker: PhantomData<(L, R)>,
}
```

Both types are parameterized by two type parameters:
- `L`: The protocol to continue with if the left branch is chosen
- `R`: The protocol to continue with if the right branch is chosen

## Duality Relationship

The duality relationship between `Offer` and `Choose` is defined as follows:

```rust
impl<L: Protocol, R: Protocol> Protocol for Offer<L, R> {
    type Dual = Choose<L::Dual, R::Dual>;
}

impl<L: Protocol, R: Protocol> Protocol for Choose<L, R> {
    type Dual = Offer<L::Dual, R::Dual>;
}
```

This means:
- The dual of `Offer<L, R>` is `Choose<L::Dual, R::Dual>`
- The dual of `Choose<L, R>` is `Offer<L::Dual, R::Dual>`

This duality ensures that when one party offers a choice, the other party makes a choice, and the protocols for each branch are duals of each other.

## API Methods

### offer Method

The `offer` method is implemented for `Chan<Offer<L, R>, IO>` and allows the channel to offer a choice between two continuations:

```rust
impl<L: Protocol, R: Protocol, IO> Chan<Offer<L, R>, IO>
where
    IO: Receiver<bool>,
    <IO as Receiver<bool>>::Error: std::fmt::Debug,
{
    pub async fn offer(mut self) -> Result<Either<Chan<L, IO>, Chan<R, IO>>, Error> {
        // Receive a boolean value indicating which branch to take
        let choice = self.io_mut().recv().map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Offer error: {:?}", e),
            ))
        })?;
        
        // Return either the left or right branch based on the choice
        if choice {
            Ok(Either::Left(Chan {
                io: self.io,
                _marker: PhantomData,
            }))
        } else {
            Ok(Either::Right(Chan {
                io: self.io,
                _marker: PhantomData,
            }))
        }
    }
}
```

This method consumes the channel and returns either a `Chan<L, IO>` or a `Chan<R, IO>` based on the choice received from the other party.

### choose_left Method

The `choose_left` method is implemented for `Chan<Choose<L, R>, IO>` and allows the channel to choose the left branch:

```rust
impl<L: Protocol, R: Protocol, IO> Chan<Choose<L, R>, IO>
where
    IO: Sender<bool>,
    <IO as Sender<bool>>::Error: std::fmt::Debug,
{
    pub async fn choose_left(mut self) -> Result<Chan<L, IO>, Error> {
        // Send a boolean value indicating the left branch
        self.io_mut().send(true).map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Choose error: {:?}", e),
            ))
        })?;
        
        // Return the channel with the left protocol
        Ok(Chan {
            io: self.io,
            _marker: PhantomData,
        })
    }
}
```

This method consumes the channel and returns a `Chan<L, IO>` after sending a choice to the other party.

### choose_right Method

The `choose_right` method is implemented for `Chan<Choose<L, R>, IO>` and allows the channel to choose the right branch:

```rust
impl<L: Protocol, R: Protocol, IO> Chan<Choose<L, R>, IO>
where
    IO: Sender<bool>,
    <IO as Sender<bool>>::Error: std::fmt::Debug,
{
    pub async fn choose_right(mut self) -> Result<Chan<R, IO>, Error> {
        // Send a boolean value indicating the right branch
        self.io_mut().send(false).map_err(|e| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Choose error: {:?}", e),
            ))
        })?;
        
        // Return the channel with the right protocol
        Ok(Chan {
            io: self.io,
            _marker: PhantomData,
        })
    }
}
```

This method consumes the channel and returns a `Chan<R, IO>` after sending a choice to the other party.

## Usage Examples

### Authentication Protocol

This example demonstrates an authentication protocol with success and failure paths:

```rust
// Define the authentication types
struct Credentials {
    username: String,
    password: String,
}

enum AuthResult {
    Success,
    Failure,
}

// Define the success and failure protocols
type SuccessProtocol = Send<String, Recv<String, End>>;
type FailureProtocol = Send<String, End>;

// Define the client and server protocols
type ClientProtocol = Send<Credentials, Recv<AuthResult, Choose<SuccessProtocol, FailureProtocol>>>;
type ServerProtocol = Recv<Credentials, Send<AuthResult, Offer<SuccessProtocol, FailureProtocol>>>;

// Client implementation
async fn run_client(chan: Chan<ClientProtocol, IO>) -> Result<(), Error> {
    // Send credentials
    let credentials = Credentials {
        username: "user".to_string(),
        password: "pass".to_string(),
    };
    let chan = chan.send(credentials).await?;
    
    // Receive authentication result
    let (result, chan) = chan.recv().await?;
    
    match result {
        AuthResult::Success => {
            // Choose the success branch
            let chan = chan.choose_left().await?;
            
            // Send a message
            let chan = chan.send("Hello, I'm authenticated!".to_string()).await?;
            
            // Receive a response
            let (response, chan) = chan.recv().await?;
            println!("Server response: {}", response);
            
            // Close the channel
            chan.close()?;
        },
        AuthResult::Failure => {
            // Choose the failure branch
            let chan = chan.choose_right().await?;
            
            // Send an error acknowledgment
            let chan = chan.send("Authentication failed, goodbye.".to_string()).await?;
            
            // Close the channel
            chan.close()?;
        }
    }
    
    Ok(())
}

// Server implementation
async fn run_server(chan: Chan<ServerProtocol, IO>) -> Result<(), Error> {
    // Receive credentials
    let (credentials, chan) = chan.recv().await?;
    
    // Validate credentials
    let is_valid = validate_credentials(&credentials);
    
    if is_valid {
        // Send success result
        let chan = chan.send(AuthResult::Success).await?;
        
        // Offer a choice
        match chan.offer().await? {
            Either::Left(chan) => {
                // Client chose the success branch
                
                // Receive the client's message
                let (message, chan) = chan.recv().await?;
                println!("Client message: {}", message);
                
                // Send a response
                let chan = chan.send("Welcome, authenticated user!".to_string()).await?;
                
                // Close the channel
                chan.close()?;
            },
            Either::Right(_) => {
                // This should never happen if the client follows the protocol
                panic!("Client chose the failure branch after successful authentication");
            }
        }
    } else {
        // Send failure result
        let chan = chan.send(AuthResult::Failure).await?;
        
        // Offer a choice
        match chan.offer().await? {
            Either::Left(_) => {
                // This should never happen if the client follows the protocol
                panic!("Client chose the success branch after failed authentication");
            },
            Either::Right(chan) => {
                // Client chose the failure branch
                
                // Receive the client's acknowledgment
                let (message, chan) = chan.recv().await?;
                println!("Client message: {}", message);
                
                // Close the channel
                chan.close()?;
            }
        }
    }
    
    Ok(())
}

fn validate_credentials(credentials: &Credentials) -> bool {
    // In a real application, this would validate against a database
    credentials.username == "admin" && credentials.password == "password"
}
```

### Menu Selection Protocol

This example demonstrates a menu selection protocol:

```rust
// Define the menu options
enum MenuItem {
    Option1,
    Option2,
}

// Define the option protocols
type Option1Protocol = Send<String, Recv<String, End>>;
type Option2Protocol = Send<i32, Recv<i32, End>>;

// Define the client and server protocols
type ClientProtocol = Recv<Vec<MenuItem>, Choose<Option1Protocol, Option2Protocol>>;
type ServerProtocol = Send<Vec<MenuItem>, Offer<Option1Protocol, Option2Protocol>>;

// Client implementation
async fn run_client(chan: Chan<ClientProtocol, IO>) -> Result<(), Error> {
    // Receive the menu options
    let (menu, chan) = chan.recv().await?;
    
    // Display the menu to the user
    println!("Menu options:");
    for (i, item) in menu.iter().enumerate() {
        println!("{}. {:?}", i + 1, item);
    }
    
    // Let the user choose an option (simulated here)
    let choice = MenuItem::Option1;
    
    match choice {
        MenuItem::Option1 => {
            // Choose the first option
            let chan = chan.choose_left().await?;
            
            // Send a string
            let chan = chan.send("Hello, Option 1!".to_string()).await?;
            
            // Receive a response
            let (response, chan) = chan.recv().await?;
            println!("Server response: {}", response);
            
            // Close the channel
            chan.close()?;
        },
        MenuItem::Option2 => {
            // Choose the second option
            let chan = chan.choose_right().await?;
            
            // Send an integer
            let chan = chan.send(42).await?;
            
            // Receive a response
            let (response, chan) = chan.recv().await?;
            println!("Server response: {}", response);
            
            // Close the channel
            chan.close()?;
        }
    }
    
    Ok(())
}

// Server implementation
async fn run_server(chan: Chan<ServerProtocol, IO>) -> Result<(), Error> {
    // Send the menu options
    let menu = vec![MenuItem::Option1, MenuItem::Option2];
    let chan = chan.send(menu).await?;
    
    // Offer a choice
    match chan.offer().await? {
        Either::Left(chan) => {
            // Client chose option 1
            
            // Receive the client's message
            let (message, chan) = chan.recv().await?;
            println!("Client message: {}", message);
            
            // Send a response
            let chan = chan.send("You chose Option 1!".to_string()).await?;
            
            // Close the channel
            chan.close()?;
        },
        Either::Right(chan) => {
            // Client chose option 2
            
            // Receive the client's number
            let (number, chan) = chan.recv().await?;
            println!("Client number: {}", number);
            
            // Send a response
            let chan = chan.send(number * 2).await?;
            
            // Close the channel
            chan.close()?;
        }
    }
    
    Ok(())
}
```

## Type Safety

The `Offer` and `Choose` types provide strong type safety guarantees:

1. **Protocol Adherence**: The type system ensures that the client and server follow the agreed-upon protocol. For example, if the client chooses the left branch, it must follow the protocol specified by the left branch.

2. **Exhaustive Matching**: When handling an `offer` result, the compiler ensures that both branches are handled, preventing bugs where one branch is forgotten.

3. **Correct Duality**: The duality relationship ensures that when one party offers a choice, the other party makes a choice, and the protocols for each branch are duals of each other.

4. **No Deadlocks**: The type system prevents deadlocks by ensuring that the client and server agree on the protocol structure, including the branching behavior.

## Best Practices

1. **Use Meaningful Branch Types**: Choose meaningful types for the left and right branches that clearly indicate their purpose.

2. **Handle Both Branches**: Always handle both branches when using `offer`, even if you expect only one branch to be taken.

3. **Validate Before Choosing**: Validate any conditions before choosing a branch to ensure that the choice is valid.

4. **Document Protocol Branches**: Clearly document the purpose and behavior of each branch in your protocol.

5. **Consider Using Enums**: Use enums to represent choices more explicitly than just `true` and `false`.

6. **Error Handling**: Include appropriate error handling in both branches, especially for unexpected choices.

7. **Testing**: Test both branches of your protocol to ensure that they work correctly.

8. **Protocol Composition**: Consider composing protocols with `Offer` and `Choose` to create more complex protocols.

By following these best practices, you can create robust and type-safe protocols with branching behavior using the `Offer` and `Choose` types.