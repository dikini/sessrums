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

## Task 3.1: Add Branching and Choice

### Challenges with Implementing Branching in Rust

Implementing branching for multiparty session types in Rust presented several interesting challenges:

1. **Type-Level Representation of Branches**: Representing branches at the type level required careful design. We used tuples to represent multiple branches, which worked well for binary choices but would need a more general solution for n-ary choices.

2. **Projection of Branching Constructs**: Projecting `GChoice` and `GOffer` was particularly challenging because the projection depends on the role. For the chooser/offeree role, the projection is different than for other roles. This required specialized implementations of the `Project` trait.

3. **Role Comparison at the Type Level**: Determining whether a role is the chooser/offeree at the type level was difficult. Rust's type system doesn't provide a straightforward way to compare types for equality at the type level. We had to use specialized implementations for specific role combinations.

### Solutions and Workarounds

To overcome these challenges, we used several techniques:

1. **Specialized Implementations**: We implemented specialized versions of the `Project` trait for specific role combinations. This allowed us to handle the different projection rules for the chooser/offeree role and other roles.

2. **Binary Branching**: We focused on implementing binary branching (choices between two options) as a starting point. This simplified the implementation while still providing the core functionality.

3. **Integration Tests**: We wrote comprehensive integration tests that verify the branching functionality with various scenarios, including nested branching and multiparty interactions.

### Lessons Learned

1. **Start with Binary Choices**: When implementing branching, it's best to start with binary choices and then extend to n-ary choices if needed. This simplifies the implementation and allows for faster progress.

2. **Role-Based Projection**: The projection of branching constructs depends on the role, which adds complexity to the implementation. Clear rules for how each role projects a branching construct are essential.

3. **Type-Level Programming Limitations**: Rust's type system has limitations when it comes to type-level programming, particularly with type equality and specialization. Finding workarounds for these limitations is a key part of implementing complex type-level functionality.

4. **Test Complex Scenarios**: Branching enables more complex communication patterns, so it's important to test various scenarios, including nested branching and multiparty interactions, to ensure the implementation works correctly.