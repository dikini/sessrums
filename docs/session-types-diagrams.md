# Session Types Visual Diagrams

This document provides visual representations of session types concepts to help understand how protocols work in the sessrums library.

## Protocol Communication Flow

```
+----------------+                      +----------------+
|     Client     |                      |     Server     |
| Send<i32,      |                      | Recv<i32,      |
| Recv<String,   |       Duality        | Send<String,   |
| End>>          | <------------------- | End>>          |
+----------------+                      +----------------+
        |                                       |
        |           Send(i32)                   |
        | -------------------------------------> |
        |                                       |
        |           Recv(String)                |
        | <------------------------------------- |
        |                                       |
        |              End                      |
        | - - - - - - - - - - - - - - - - - - - |
        |                                       |
```

## Protocol Type Composition

Session types can be composed to create complex protocols:

```
Send<i32, Recv<String, End>>
  |
  +-- Send<i32, ...>: First send an i32
       |
       +-- Recv<String, ...>: Then receive a String
            |
            +-- End: Then end the communication
```

## Duality Relationship

For every protocol, there exists a dual protocol that represents the complementary behavior:

```
Protocol                  Dual Protocol
---------                 -------------
Send<T, P>                Recv<T, P::Dual>
Recv<T, P>                Send<T, P::Dual>
Offer<L, R>               Choose<L::Dual, R::Dual>
Choose<L, R>              Offer<L::Dual, R::Dual>
End                       End
```

## Channel State Transitions

The `Chan<P, IO>` type changes its protocol type parameter as communication progresses:

```
Chan<Send<i32, Recv<String, End>>, IO>
  |
  | send(42)
  v
Chan<Recv<String, End>, IO>
  |
  | recv() -> ("Hello", chan)
  v
Chan<End, IO>
  |
  | close()
  v
()  // Protocol completed
```

## Client-Server Query-Response Protocol

```
Client                                Server
  |                                     |
  |------- Send(String) - Query ------->|
  |                                     |
  |<------ Recv(String) - Response -----|
  |                                     |
  |-------------- End ---------------->|
```

## More Complex Protocol with Choice

```
+----------------+                      +----------------+
|     Client     |                      |     Server     |
| Send<Auth,     |                      | Recv<Auth,     |
| Recv<          |                      | Send<          |
|   Choose<      |                      |   Offer<       |
|     Success,   |                      |     Success,   |
|     Failure    |                      |     Failure    |
|   >            |                      |   >            |
| >>             |                      | >>             |
+----------------+                      +----------------+
        |                                       |
        |           Send(Auth)                  |
        | -------------------------------------> |
        |                                       |
        |        Recv(Choose<...>)              |
        | <------------------------------------- |
        |                                       |
        |  If Success:                          |
        |  - Continue with successful protocol  |
        |                                       |
        |  If Failure:                          |
        |  - Handle authentication failure      |
        |                                       |
```

These diagrams illustrate the key concepts of session types and how they are implemented in the sessrums library. The type-level representation ensures that communication follows the specified protocol, preventing errors at compile time.