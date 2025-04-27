# MPST Concepts

This document summarizes key concepts in Multiparty Session Types (MPST).

## Roles

In MPST, a **Role** represents a participant in a communication protocol. Each role has a unique identifier within the protocol. Protocols define interactions between specific roles.

## Global Protocols

A **Global Protocol** provides a high-level, centralized description of the entire communication flow among all participating roles. It specifies the sequence of messages exchanged, including who sends what to whom and under what conditions. Global protocols ensure that the overall interaction is well-defined and free from deadlocks or other communication errors.

## Projection

**Projection** is the process of deriving a local session type for a specific role from the global protocol. Each participant only needs to know its own local view of the interaction. The projection for a role describes the sequence of sends and receives that role is expected to perform according to the global protocol. If a global protocol is well-formed, its projections are guaranteed to be deadlock-free.

## Branching (Offer/Choose)

**Branching** allows a protocol to diverge based on a choice made by a participant.
- **Offer**: A role *offers* a choice of different subsequent behaviors to another role.
- **Choose**: A role *chooses* one of the offered branches, determining the path of the protocol execution.

This mechanism enables dynamic and conditional communication flows.

## Recursion

**Recursion** allows protocols to define repeating sequences of interactions. This is essential for modeling ongoing services or iterative processes where the same pattern of communication may occur multiple times. Recursive types are typically defined using a fixed-point operator.

## Composition

**Composition** refers to combining smaller, simpler protocols to build more complex ones. Protocols can be composed sequentially or in parallel, allowing for modular design and reusability of protocol definitions. Well-formed protocols can be composed while preserving properties like deadlock-freedom.