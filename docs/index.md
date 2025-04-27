# sessrums Documentation Index

This document provides an index of all documentation files for the sessrums session types library.

## Core Documentation

- [README.md](../README.md) - Overview of the library and its features
- [Session Types Documentation](session-types-documentation.md) - Comprehensive guide to the library

## Quick Reference

- [Quick Reference Guide](quick-reference.md) - Concise summary of key concepts and API methods

## Visual Aids

- [Session Types Diagrams](session-types-diagrams.md) - Visual representations of session types concepts

## Detailed Guides

- [Error Handling Guide](error-handling.md) - Detailed information about error handling
- [Testing Protocols Guide](testing-protocols.md) - Examples and best practices for testing protocols
- [Offer and Choose Guide](offer-choose.md) - Detailed information about the Offer and Choose protocol types
- [API Ergonomics Guide](api-ergonomics.md) - Guide to using the API ergonomics improvements

## Examples

The library includes several examples that demonstrate how to use session types in practice:

- [Simple Client-Server Protocol](../examples/simple.rs) - A simple query-response protocol
- [Async Runtime Integration](../examples/async.rs) - Integration with async runtimes
- [Complex Protocol Example](../examples/complex.rs) - A more complex protocol with choice
- [Connection Establishment](../examples/connect.rs) - Establishing connections between endpoints
- [Recursion Example](../examples/recursion.rs) - Using recursive session types
- [Tokio Integration](../examples/tokio_integration.rs) - Integration with the Tokio runtime
- [Async-std Integration](../examples/async_std_integration.rs) - Integration with the async-std runtime

### Test Examples

- Protocol Tests:
  - [Protocol 1: Simple Send/Recv Ping-Pong](../tests/integration/protocol_1.rs)
  - [Protocol 2: Offer and Choose](../tests/integration/protocol_2.rs)
  - [Protocol 3: Complex Protocol](../tests/integration/protocol_3.rs)
  - [Protocol 4: Error Handling](../tests/integration/protocol_4.rs)
  - [Protocol 5: Advanced Features](../tests/integration/protocol_5.rs)
- [Compile-Time Tests](../tests/compile_tests.rs) - Tests that verify compile-time behavior
- [Runtime Tests](../tests/runtime_tests.rs) - Tests that verify runtime behavior
- [Final Integration Test](../tests/final_integration_test.rs) - A comprehensive test that uses all library features

## Getting Started

If you're new to session types, we recommend starting with the following resources:

1. Read the [README.md](../README.md) for an overview of the library
2. Look at the [Session Types Diagrams](session-types-diagrams.md) to understand the concepts visually
3. Read the [Session Types Documentation](session-types-documentation.md) for a comprehensive guide
4. Read the [API Ergonomics Guide](api-ergonomics.md) to learn about the ergonomic API improvements
5. Explore the [examples](../examples/) to see how to use the library in practice
6. Check out the [tests](../tests/) to see how to test session type protocols

## Contributing

If you'd like to contribute to the sessrums library, please read the following resources:

- [Contributing Guidelines](../CONTRIBUTING.md) (if available)
- [Code of Conduct](../CODE_OF_CONDUCT.md) (if available)

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.