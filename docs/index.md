# sessrums Documentation Index

This document provides an index of all documentation files for the sessrums session types library.

## Core Documentation

- [Introduction](introduction.md) - Consolidated guide covering core concepts, usage, and examples.
- [README.md](../README.md) - Overview of the library and its features.

## Detailed Guides

- [MPST Design Principles](mpst-design.md) - Design choices and rationale behind the MPST implementation.
- [MPST Macro Usage](mpst-macro.md) - Guide to using the `mpst_seq!` macro.
- [API Ergonomics Guide](api-ergonomics.md) - Guide to using the API ergonomics improvements.
- [Error Handling Guide](error-handling.md) - Detailed information about error handling.
- [Testing Protocols Guide](testing-protocols.md) - Examples and best practices for testing protocols.
- [MPST Research Notes](mpst-research-notes.md) - Notes related to MPST research and implementation details.

## Visual Aids

- [Session Types Diagram](session-types-diagram.svg) - Visual representation of a session type interaction.

## Examples

The library includes several examples that demonstrate how to use session types in practice:

- [Async Runtime Integration](../examples/async.rs) - Integration with async runtimes.
- [Recursion Example](../examples/recursion.rs) - Using recursive session types.
- [Async-std Integration](../examples/async_std_integration.rs) - Integration with the async-std runtime.
- Explore all [examples](../examples/).

### Test Examples

- Protocol Tests:
  - [Protocol 1: Simple Send/Recv Ping-Pong](../tests/integration/protocol_1.rs)
  - [Protocol 2: Offer and Choose](../tests/integration/protocol_2.rs)
  - [Protocol 3: Complex Protocol](../tests/integration/protocol_3.rs)
  - [Protocol 4: Error Handling](../tests/integration/protocol_4.rs)
  - [Protocol 5: Advanced Features](../tests/integration/protocol_5.rs)
- [Compile-Time Tests](../tests/compile_tests.rs) - Tests that verify compile-time behavior.
- [Final Integration Test](../tests/final_integration_test.rs) - A comprehensive test that uses all library features.
- Explore all [tests](../tests/).

## Getting Started

If you're new to session types, we recommend starting with the following resources:

1.  Read the [README.md](../README.md) for an overview of the library.
2.  Read the [Introduction](introduction.md) for a comprehensive guide.
3.  Explore the [examples](../examples/) to see how to use the library in practice.
4.  Check out the [tests](../tests/) to see how to test session type protocols.