# Insights

## Task 2.3: Implement Projection

### Challenges with Type-Level Programming in Rust

Implementing the projection functionality for multiparty session types revealed several challenges with type-level programming in Rust:

1. **Overlapping Implementations**: Rust doesn't allow overlapping implementations of traits, which made it difficult to implement the `Project` trait for different roles. For example, we needed different implementations for the `From` role, the `To` role, and any other role, but Rust's trait system doesn't support this kind of specialization directly.

2. **Specialization**: While Rust has an unstable `specialization` feature that could help with overlapping implementations, it's not available in stable Rust. This forced us to find alternative approaches to implement the projection functionality.

3. **Type-Level Functions**: Implementing type-level functions in Rust requires careful design to avoid conflicts with the trait system. We had to use associated types and trait bounds to express the projection rules.

### Solutions and Workarounds

To overcome these challenges, we used several techniques:

1. **Simplified Implementation**: We focused on implementing the core projection functionality for the most important cases, leaving more complex cases for future work. This allowed us to make progress without getting stuck on the limitations of Rust's type system.

2. **Type-Level Traits**: We used traits with associated types to represent type-level functions, which allowed us to express the projection rules in a way that Rust's type system could understand.

3. **Integration Tests**: We wrote integration tests that directly use the projected local types, rather than trying to test the projection process itself. This allowed us to verify that the projection functionality works correctly without having to expose all the implementation details.

### Lessons Learned

1. **Start Simple**: When implementing complex type-level functionality, it's best to start with a simple implementation that covers the core cases, and then gradually extend it to handle more complex cases.

2. **Use Type-Level Traits**: Traits with associated types are a powerful tool for expressing type-level functions in Rust.

3. **Test the Interface, Not the Implementation**: When testing type-level functionality, focus on testing the interface (what users of the library will see) rather than the implementation details.

4. **Documentation is Key**: Clear documentation of the projection rules and examples of how to use the projection functionality is essential for users to understand how to use the library effectively.