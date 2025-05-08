# Recursive Protocol: Data Streaming Example

This example demonstrates a recursive protocol for streaming data with termination using the MPST DSL. It shows how to use the `rec` and `continue` constructs to create protocols with repetitive behavior.

## Protocol Definition

```rust
use sessrums_macro::{mpst, project};
use sessrums_types::roles::{Producer, Consumer};
use sessrums_types::transport::MockChannelEnd;
use sessrums_types::session_types::{Session, Either};
use sessrums_types::error::SessionError;
use serde::{Serialize, Deserialize};

// Define custom message types for the protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataChunk {
    data: Vec<u8>,
    sequence_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ack {
    sequence_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EndOfStream;

// Define a protocol for streaming data with termination
mpst! {
    protocol DataStream {
        // Define the participants
        participant Producer;
        participant Consumer;

        // Define the recursive interaction
        rec Stream {
            // Producer decides whether to send more data or end the stream
            choice at Producer {
                // Send more data
                option SendData {
                    Producer -> Consumer: DataChunk;
                    Consumer -> Producer: Ack;
                    continue Stream;  // Continue the recursion
                }
                
                // End the stream
                option EndStream {
                    Producer -> Consumer: EndOfStream;
                    end;
                }
            }
        }
    }
}
```

## Projection to Local Protocols

Once we have defined the global protocol, we can project it to local protocols for each participant:

```rust
// Project the global protocol to local protocols for each role
type ProducerProtocol = project!(DataStream, Producer);
type ConsumerProtocol = project!(DataStream, Consumer);
```

The projection results in the following local protocols:

- `ProducerProtocol`: A recursive protocol where the Producer chooses between sending a DataChunk and receiving an Ack before continuing the recursion, or sending an EndOfStream and ending the protocol
- `ConsumerProtocol`: A recursive protocol where the Consumer offers a choice from the Producer, either receiving a DataChunk and sending an Ack before continuing the recursion, or receiving an EndOfStream and ending the protocol

## Implementation

Here's how we can implement the producer and consumer behaviors using the projected protocols:

```rust
// Producer implementation
async fn run_producer(
    session: Session<ProducerProtocol, MockChannelEnd>,
    data: Vec<Vec<u8>>
) -> Result<(), SessionError> {
    // Get a recursive session
    let mut session = session.enter_rec().await?;
    
    // Stream each chunk of data
    for (i, chunk) in data.iter().enumerate() {
        // Decide to send data
        println!("Producer: Sending chunk {}", i);
        let session_next = session.select_left().await?;
        
        // Send the data chunk
        let data_chunk = DataChunk {
            data: chunk.clone(),
            sequence_number: i as u32,
        };
        let session_next = session_next.send(data_chunk).await?;
        
        // Receive acknowledgment
        let (ack, session_next) = session_next.receive().await?;
        println!("Producer: Received ACK for chunk {}", ack.sequence_number);
        
        // Continue the recursion
        session = session_next.continue_rec().await?;
    }
    
    // End the stream
    println!("Producer: Ending stream");
    let session = session.select_right().await?;
    
    // Send end of stream marker
    let session = session.send(EndOfStream).await?;
    
    // End the session
    session.close().await?;
    
    Ok(())
}

// Consumer implementation
async fn run_consumer(
    session: Session<ConsumerProtocol, MockChannelEnd>
) -> Result<(), SessionError> {
    // Get a recursive session
    let mut session = session.enter_rec().await?;
    
    // Keep receiving data until the stream ends
    loop {
        // Offer a choice from the producer
        let session_branch = session.offer().await?;
        
        match session_branch {
            // Receive data
            Either::Left(session_next) => {
                // Receive the data chunk
                let (chunk, session_next) = session_next.receive().await?;
                println!("Consumer: Received chunk {} with {} bytes", 
                         chunk.sequence_number, chunk.data.len());
                
                // Send acknowledgment
                let ack = Ack {
                    sequence_number: chunk.sequence_number,
                };
                let session_next = session_next.send(ack).await?;
                
                // Continue the recursion
                session = session_next.continue_rec().await?;
            },
            
            // End of stream
            Either::Right(session_next) => {
                // Receive the end of stream marker
                let (_, session_next) = session_next.receive().await?;
                println!("Consumer: Received end of stream");
                
                // End the session
                session_next.close().await?;
                
                // Exit the loop
                break;
            }
        }
    }
    
    Ok(())
}
```

## Running the Example

To run the example, we need to create a pair of channels and spawn the producer and consumer tasks:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create some test data
    let data = vec![
        b"First chunk of data".to_vec(),
        b"Second chunk of data".to_vec(),
        b"Third chunk of data".to_vec(),
        b"Fourth chunk of data".to_vec(),
        b"Fifth chunk of data".to_vec(),
    ];
    
    // Create a pair of channels
    let (producer_channel, consumer_channel) = MockChannelEnd::new_pair();
    
    // Create sessions for producer and consumer
    let producer_session = Session::<ProducerProtocol, _>::new(producer_channel);
    let consumer_session = Session::<ConsumerProtocol, _>::new(consumer_channel);
    
    // Spawn producer and consumer tasks
    let producer_task = tokio::spawn(run_producer(producer_session, data));
    let consumer_task = tokio::spawn(run_consumer(consumer_session));
    
    // Wait for both tasks to complete
    let _ = tokio::try_join!(producer_task, consumer_task)?;
    
    Ok(())
}
```

## Output

When running this example, you should see output similar to:

```
Producer: Sending chunk 0
Consumer: Received chunk 0 with 19 bytes
Producer: Received ACK for chunk 0
Producer: Sending chunk 1
Consumer: Received chunk 1 with 20 bytes
Producer: Received ACK for chunk 1
Producer: Sending chunk 2
Consumer: Received chunk 2 with 19 bytes
Producer: Received ACK for chunk 2
Producer: Sending chunk 3
Consumer: Received chunk 3 with 20 bytes
Producer: Received ACK for chunk 3
Producer: Sending chunk 4
Consumer: Received chunk 4 with 19 bytes
Producer: Received ACK for chunk 4
Producer: Ending stream
Consumer: Received end of stream
```

## Key Points

This example demonstrates several key concepts:

1. **Recursion**: Using the `rec Label` and `continue Label` constructs to create protocols with repetitive behavior
2. **Recursion with Choice**: Combining recursion with choice to create protocols that can repeat or terminate based on a decision
3. **Projection of Recursion**: How recursion is projected to different roles:
   - Both roles get a recursive protocol with the same structure
   - The `continue` statement is projected to a `continue` statement in the local protocol
4. **Type Safety**: The compiler ensures that the recursion is well-formed and that all participants handle the recursion correctly

Recursion is a powerful feature of the MPST system that allows you to model protocols with repetitive behavior. It ensures that all participants agree on when to continue or exit the recursion.

## Advanced Recursion Patterns

The MPST DSL supports more complex recursion patterns, such as:

### Nested Recursion

```rust
mpst! {
    protocol NestedRecursion {
        participant A;
        participant B;
        
        rec Outer {
            A -> B: String;
            
            rec Inner {
                B -> A: String;
                
                choice at A {
                    option ContinueInner {
                        continue Inner;
                    }
                    or {
                        choice at A {
                            option ContinueOuter {
                                continue Outer;
                            }
                            or {
                                end;
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Recursion in Choice Branches

```rust
mpst! {
    protocol RecursionInChoice {
        participant A;
        participant B;
        
        choice at A {
            option Branch1 {
                rec Loop1 {
                    A -> B: String;
                    B -> A: String;
                    
                    choice at A {
                        option Continue1 {
                            continue Loop1;
                        }
                        or {
                            end;
                        }
                    }
                }
            }
            or {
                rec Loop2 {
                    A -> B: i32;
                    B -> A: i32;
                    
                    choice at A {
                        option Continue2 {
                            continue Loop2;
                        }
                        or {
                            end;
                        }
                    }
                }
            }
        }
    }
}
```

These advanced recursion patterns allow you to model complex protocols with multiple levels of repetition and different paths through the protocol.