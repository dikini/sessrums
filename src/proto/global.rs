use std::marker::PhantomData;
use super::roles::Role;
use crate::error::{Error, Result};

/// Trait representing a global protocol in a multiparty session.
///
/// A global protocol describes the communication behavior between multiple roles
/// in a distributed system. It specifies the sequence and types of messages
/// exchanged between participants, as well as control flow structures like
/// choices and recursion.
pub trait GlobalProtocol {
    /// Returns a string representation of the protocol for debugging.
    fn protocol_name(&self) -> &'static str;

    /// Validates the structure of the global protocol.
    ///
    /// This method checks for structural errors like choices with no branches,
    /// mismatched recursion labels, etc.
    ///
    /// **Note:** Implementations for specific protocol types (e.g., `GSend`, `GRec`, `GSeq`)
    /// may currently be placeholders and might not perform full validation. Refer to the
    /// specific type's documentation for its current validation status.
    fn validate(&self) -> Result<()>;

    /// Returns the roles involved in this protocol.
    ///
    /// This method returns a vector of unique role names that participate in the protocol.
    ///
    /// **Note:** Implementations for specific protocol types (e.g., `GSend`, `GRec`, `GSeq`)
    /// may currently be placeholders and might not collect roles from nested protocols.
    /// Refer to the specific type's documentation for its current status.
    fn involved_roles(&self) -> Vec<&'static str>;
}

/// Represents sending a value of type `T` from role `From` to role `To`,
/// then continuing with the protocol `Next`.
pub struct GSend<T, From: Role, To: Role, Next>(PhantomData<(T, From, To, Next)>);

impl<T, From: Role, To: Role, Next> GSend<T, From, To, Next> {
    /// Creates a new `GSend` protocol step.
    pub fn new() -> Self {
        GSend(PhantomData)
    }

    /// Attempts to project the global send protocol for a specific role `R`.
    ///
    /// This method is intended to determine the local protocol step for role `R`
    /// based on this global `GSend` step.
    ///
    /// # Current Implementation Status
    ///
    /// This method currently provides a *simplified* runtime check:
    /// *   If `R` is the sender (`From`), it returns `Ok(ProjectedSendRecv::Send)`.
    /// *   If `R` is the receiver (`To`), it returns `Ok(ProjectedSendRecv::Recv)`.
    /// *   If `R` is neither `From` nor `To`, it returns `Err(Error::UnknownRole)`.
    ///
    /// The return type `ProjectedSendRecv` is a temporary enum used only to
    /// represent the immediate send/receive action and does *not* include the
    /// projected continuation (`Next`).
    ///
    /// **Note:** The full, type-safe projection logic is handled by the
    /// `sessrums::proto::projection::Project` trait at compile time, not by this
    /// runtime method. This method primarily serves to illustrate potential
    /// runtime errors if a role not involved tries to project.
    pub fn project_for_role<R: Role>() -> Result<
        ProjectedSendRecv<T, <Next as super::projection::Project<R>>::LocalProtocol>
    >
    where
        Next: super::projection::Project<R>,
    {
        let r_instance = R::default();
        let from_instance = From::default();
        let to_instance = To::default();

        if r_instance.name() == from_instance.name() {
            // Project for the 'From' role: Send
            Ok(ProjectedSendRecv::Send(PhantomData))
        } else if r_instance.name() == to_instance.name() {
            // Project for the 'To' role: Recv
            Ok(ProjectedSendRecv::Recv(PhantomData))
        } else {
            // Project for a third party: Conceptually handled by the Project trait.
            // This runtime check returns an error for uninvolved roles.
            Err(Error::UnknownRole(r_instance.name()))
        }
    }
}

impl<T, From: Role, To: Role, Next> GlobalProtocol for GSend<T, From, To, Next>
where
    Next: GlobalProtocol
{
    fn protocol_name(&self) -> &'static str {
        "GSend"
    }

    /// Validates the `GSend` protocol step.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It only checks that the
    /// `From` and `To` roles are different. It does **not** recursively validate
    /// the continuation protocol `Next`.
    fn validate(&self) -> Result<()> {
        // Validate that From and To are different roles
        let from_instance = From::default();
        let to_instance = To::default();

        if from_instance.name() == to_instance.name() {
            return Err(Error::InvalidProtocolStructure(
                "Sender and receiver must be different roles in GSend"
            ));
        }

        // TODO: Implement recursive validation for the 'Next' protocol.
        // Next::validate() should be called here.
        Ok(())
    }

    /// Returns the roles involved in this `GSend` step and its continuation.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It only returns the
    /// immediate sender (`From`) and receiver (`To`) roles. It does **not**
    /// recursively collect roles from the continuation protocol `Next`.
    fn involved_roles(&self) -> Vec<&'static str> {
        let from_instance = From::default();
        let to_instance = To::default();

        let roles = vec![from_instance.name(), to_instance.name()];

        // TODO: Implement recursive role collection for the 'Next' protocol.
        // Roles from Next::involved_roles() should be collected and merged here.
        roles
    }
}

/// **Temporary Enum:** Represents the immediate projected local step for `GSend`/`GRecv`.
///
/// This enum is a **simplification** used only within the `project_for_role` methods
/// of `GSend` and `GRecv` to demonstrate the immediate send/receive nature for the
/// involved roles.
///
/// It does **not** represent the full local protocol projection, which also includes
/// the projection of the continuation protocol (`Next`). The complete projection is
/// handled at the type level by the `sessrums::proto::projection::Project` trait.
pub enum ProjectedSendRecv<T, Next> {
    /// Represents a local `Send` action.
    Send(PhantomData<(T, Next)>),
    /// Represents a local `Recv` action.
    Recv(PhantomData<(T, Next)>),
}

/// Represents receiving a value of type `T` by role `To` from role `From`,
/// then continuing with the protocol `Next`.
pub struct GRecv<T, From: Role, To: Role, Next>(PhantomData<(T, From, To, Next)>);

impl<T, From: Role, To: Role, Next> GRecv<T, From, To, Next> {
    /// Creates a new `GRecv` protocol step.
    pub fn new() -> Self {
        GRecv(PhantomData)
    }

    /// Attempts to project the global receive protocol for a specific role `R`.
    ///
    /// This method is intended to determine the local protocol step for role `R`
    /// based on this global `GRecv` step.
    ///
    /// # Current Implementation Status
    ///
    /// This method currently provides a *simplified* runtime check:
    /// *   If `R` is the sender (`From`), it returns `Ok(ProjectedSendRecv::Send)`.
    /// *   If `R` is the receiver (`To`), it returns `Ok(ProjectedSendRecv::Recv)`.
    /// *   If `R` is neither `From` nor `To`, it returns `Err(Error::UnknownRole)`.
    ///
    /// The return type `ProjectedSendRecv` is a temporary enum used only to
    /// represent the immediate send/receive action and does *not* include the
    /// projected continuation (`Next`).
    ///
    /// **Note:** The full, type-safe projection logic is handled by the
    /// `sessrums::proto::projection::Project` trait at compile time, not by this
    /// runtime method. This method primarily serves to illustrate potential
    /// runtime errors if a role not involved tries to project.
    pub fn project_for_role<R: Role>() -> Result<
        ProjectedSendRecv<T, <Next as super::projection::Project<R>>::LocalProtocol>
    >
    where
        Next: super::projection::Project<R>,
    {
        let r_instance = R::default();
        let from_instance = From::default();
        let to_instance = To::default();

        if r_instance.name() == from_instance.name() {
            // Project for the 'From' role: Send
            Ok(ProjectedSendRecv::Send(PhantomData))
        } else if r_instance.name() == to_instance.name() {
            // Project for the 'To' role: Recv
            Ok(ProjectedSendRecv::Recv(PhantomData))
        } else {
            // Project for a third party: Conceptually handled by the Project trait.
            // This runtime check returns an error for uninvolved roles.
            Err(Error::UnknownRole(r_instance.name()))
        }
    }
}

impl<T, From: Role, To: Role, Next> GlobalProtocol for GRecv<T, From, To, Next>
where
    Next: GlobalProtocol
{
    fn protocol_name(&self) -> &'static str {
        "GRecv"
    }

    /// Validates the `GRecv` protocol step.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It only checks that the
    /// `From` and `To` roles are different. It does **not** recursively validate
    /// the continuation protocol `Next`.
    fn validate(&self) -> Result<()> {
        // Validate that From and To are different roles
        let from_instance = From::default();
        let to_instance = To::default();

        if from_instance.name() == to_instance.name() {
            return Err(Error::InvalidProtocolStructure(
                "Sender and receiver must be different roles in GRecv"
            ));
        }

        // TODO: Implement recursive validation for the 'Next' protocol.
        // Next::validate() should be called here.
        Ok(())
    }

    /// Returns the roles involved in this `GRecv` step and its continuation.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It only returns the
    /// immediate sender (`From`) and receiver (`To`) roles. It does **not**
    /// recursively collect roles from the continuation protocol `Next`.
    fn involved_roles(&self) -> Vec<&'static str> {
        let from_instance = From::default();
        let to_instance = To::default();

        let roles = vec![from_instance.name(), to_instance.name()];

        // TODO: Implement recursive role collection for the 'Next' protocol.
        // Roles from Next::involved_roles() should be collected and merged here.
        roles
    }
}

/// Represents an external choice made by role `Chooser`, leading to one of the
/// protocols specified in the `Branches` tuple.
///
/// The `Chooser` role selects which branch of the protocol to follow. Other roles
/// will typically see this as an `GOffer`.
pub struct GChoice<Chooser: Role, Branches>(pub Branches, PhantomData<Chooser>);

impl<Chooser: Role, Branches> GChoice<Chooser, Branches> {
    /// Creates a new `GChoice` protocol step with the given branches.
    pub fn new(branches: Branches) -> Self {
        GChoice(branches, PhantomData)
    }
}

impl<Chooser: Role, Branches> GlobalProtocol for GChoice<Chooser, Branches>
where
    Branches: GlobalProtocolBranches
{
    fn protocol_name(&self) -> &'static str {
        "GChoice"
    }

    /// Validates the `GChoice` protocol step.
    ///
    /// This validation delegates to the `validate_branches` method of the
    /// `GlobalProtocolBranches` trait implemented by the `Branches` tuple.
    /// It ensures that each branch protocol within the tuple is itself valid.
    fn validate(&self) -> Result<()> {
        // Validate that there are branches to choose from and each branch is valid.
        self.0.validate_branches()
    }

    /// Returns the roles involved in this `GChoice` step.
    ///
    /// This includes the `Chooser` role and all roles involved in any of the
    /// possible `Branches`. The list of roles is deduplicated.
    fn involved_roles(&self) -> Vec<&'static str> {
        let chooser_instance = Chooser::default();
        let mut roles = vec![chooser_instance.name()];

        // Add roles from each branch
        let branch_roles = self.0.involved_roles_in_branches();
        roles.extend(branch_roles);

        // Remove duplicates
        roles.sort();
        roles.dedup();

        roles
    }
}

/// Represents an internal choice (offer) received by role `Offeree`, presenting
/// different protocol continuations specified in the `Branches` tuple.
///
/// The `Offeree` role receives the choice made by another role (typically via `GChoice`)
/// and proceeds down the corresponding protocol branch.
pub struct GOffer<Offeree: Role, Branches>(pub Branches, PhantomData<Offeree>);

impl<Offeree: Role, Branches> GOffer<Offeree, Branches> {
    /// Creates a new `GOffer` protocol step with the given branches.
    pub fn new(branches: Branches) -> Self {
        GOffer(branches, PhantomData)
    }
}

impl<Offeree: Role, Branches> GlobalProtocol for GOffer<Offeree, Branches>
where
    Branches: GlobalProtocolBranches
{
    fn protocol_name(&self) -> &'static str {
        "GOffer"
    }

    /// Validates the `GOffer` protocol step.
    ///
    /// This validation delegates to the `validate_branches` method of the
    /// `GlobalProtocolBranches` trait implemented by the `Branches` tuple.
    /// It ensures that each branch protocol within the tuple is itself valid.
    fn validate(&self) -> Result<()> {
        // Validate that there are branches to offer and each branch is valid.
        self.0.validate_branches()
    }

    /// Returns the roles involved in this `GOffer` step.
    ///
    /// This includes the `Offeree` role and all roles involved in any of the
    /// possible `Branches`. The list of roles is deduplicated.
    fn involved_roles(&self) -> Vec<&'static str> {
        let offeree_instance = Offeree::default();
        let mut roles = vec![offeree_instance.name()];

        // Add roles from each branch
        let branch_roles = self.0.involved_roles_in_branches();
        roles.extend(branch_roles);

        // Remove duplicates
        roles.sort();
        roles.dedup();

        roles
    }
}

/// Helper trait for tuples of global protocols used as branches in `GChoice` and `GOffer`.
///
/// This trait provides methods to validate all protocols within the tuple and
/// collect all roles involved across all branches. It is implemented for tuples
/// containing types that implement `GlobalProtocol`.
pub trait GlobalProtocolBranches {
    /// Validates all protocol branches contained within the tuple.
    ///
    /// Each element in the tuple must implement `GlobalProtocol`, and this method
    /// calls `validate()` on each of them.
    fn validate_branches(&self) -> Result<()>;

    /// Returns a deduplicated list of all roles involved in any branch protocol.
    ///
    /// Calls `involved_roles()` on each protocol in the tuple and combines the results.
    fn involved_roles_in_branches(&self) -> Vec<&'static str>;
}

// Implement GlobalProtocolBranches for the empty tuple
impl GlobalProtocolBranches for () {
    /// Validates the branches (always Ok for an empty tuple).
    fn validate_branches(&self) -> Result<()> {
        Ok(())
    }

    /// Returns involved roles (empty for an empty tuple).
    fn involved_roles_in_branches(&self) -> Vec<&'static str> {
        vec![]
    }
}

// Implement GlobalProtocolBranches for a single-element tuple
impl<G: GlobalProtocol> GlobalProtocolBranches for (G,) {
    /// Validates the single branch by calling `validate()` on it.
    fn validate_branches(&self) -> Result<()> {
        self.0.validate()
    }

    /// Returns roles involved in the single branch.
    fn involved_roles_in_branches(&self) -> Vec<&'static str> {
        self.0.involved_roles()
    }
}

// Implement GlobalProtocolBranches for a two-element tuple
impl<G1: GlobalProtocol, G2: GlobalProtocol> GlobalProtocolBranches for (G1, G2) {
    /// Validates both branches by calling `validate()` on each.
    fn validate_branches(&self) -> Result<()> {
        self.0.validate()?;
        self.1.validate()?;
        Ok(())
    }

    /// Returns roles involved in both branches, deduplicated.
    fn involved_roles_in_branches(&self) -> Vec<&'static str> {
        let mut roles = self.0.involved_roles();
        roles.extend(self.1.involved_roles());
        // Remove duplicates
        roles.sort();
        roles.dedup();
        roles
    }
}

// TODO: Implement GlobalProtocolBranches for larger tuples (3, 4, etc.) if needed,
// potentially using a macro_rules approach.

/// Represents a recursive protocol definition identified by a `Label`.
///
/// The `GRec<Label, Protocol>` type allows defining protocols that can refer back
/// to themselves using a corresponding `GVar<Label>`. This enables modeling
/// protocols with loops or repeated interactions.
///
/// The actual protocol logic is defined in the `Protocol` type parameter, which
/// typically contains one or more `GVar<Label>` instances at the points of recursion.
///
/// # Type Parameters
///
/// * `Label` - A unique type (often an empty struct or enum) used as an identifier
///             for this recursive definition.
/// * `Protocol` - The body of the recursive protocol, which must implement `GlobalProtocol`.
///                It should contain `GVar<Label>` where recursion occurs.
///
/// # Examples
///
/// ```rust
/// use sessrums::proto::global::*;
/// use sessrums::proto::roles::{RoleA, RoleB};
/// use std::marker::PhantomData;
///
/// // Define roles
/// #[derive(Default)] struct Ping; impl Role for Ping { fn name(&self) -> &'static str { "Ping" } }
/// #[derive(Default)] struct Pong; impl Role for Pong { fn name(&self) -> &'static str { "Pong" } }
///
/// // Define a label for the recursion
/// struct PingPongLoop;
///
/// // Define the recursive protocol: Ping sends u32 to Pong, Pong sends u32 to Ping, repeat.
/// type PingPongProto = GRec<PingPongLoop, GSend<u32, Ping, Pong, GRecv<u32, Pong, Ping, GVar<PingPongLoop>>>>;
///
/// let protocol = PingPongProto::new();
/// assert_eq!(protocol.protocol_name(), "GRec");
///
/// // Note: Validation and role involvement for GRec/GVar are currently placeholders.
/// // assert!(protocol.validate().is_ok()); // Placeholder: Currently returns Ok
/// // let roles = protocol.involved_roles(); // Placeholder: Currently returns []
/// // assert!(roles.contains(&"Ping"));
/// // assert!(roles.contains(&"Pong"));
/// ```
pub struct GRec<Label, Protocol>(PhantomData<(Label, Protocol)>);

impl<Label, Protocol> GRec<Label, Protocol> {
    /// Creates a new `GRec` protocol step.
    pub fn new() -> Self {
        GRec(PhantomData)
    }

    /// Conceptually returns the inner protocol definition.
    ///
    /// **Note:** Due to the use of `PhantomData` for type-level representation,
    /// this method cannot actually return a reference to the `Protocol` instance.
    /// It exists to illustrate the concept of accessing the recursive protocol body.
    pub fn inner_protocol(&self) -> Option<&Protocol> {
        // Cannot return a real reference due to PhantomData.
        None
    }
}

impl<Label, Protocol: GlobalProtocol> GlobalProtocol for GRec<Label, Protocol> {
    fn protocol_name(&self) -> &'static str {
        "GRec"
    }

    /// Validates the `GRec` protocol step.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It always returns `Ok(())`.
    ///
    /// A complete implementation would need to:
    /// 1.  Recursively validate the inner `Protocol`.
    /// 2.  Verify that all `GVar<Label>` references within `Protocol` correctly
    ///     refer back to this `GRec<Label, _>`.
    /// 3.  Ensure the recursion is *productive* (i.e., doesn't loop infinitely
    ///     without progress, typically by requiring an action like send/recv
    ///     before a `GVar`). This often requires context or specific algorithms
    ///     not implemented here.
    fn validate(&self) -> Result<()> {
        // TODO: Implement full recursive validation for GRec.
        // This requires accessing the inner Protocol type and performing
        // checks for well-formedness and productivity, likely needing
        // changes to how protocols are represented or validated.
        Ok(())
    }

    /// Returns the roles involved in the recursive protocol.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It always returns an
    /// empty vector (`vec![]`).
    ///
    /// A complete implementation would need to recursively call `involved_roles()`
    /// on the inner `Protocol` definition. This requires access to the `Protocol`
    /// type, which is currently not possible due to `PhantomData`.
    fn involved_roles(&self) -> Vec<&'static str> {
        // TODO: Implement recursive role collection for GRec.
        // This requires accessing the inner Protocol type, potentially
        // via Default or other means, and calling its involved_roles().
        vec![]
    }
}

/// Represents a recursive variable, referring back to an enclosing `GRec<Label, _>`.
///
/// `GVar<Label>` acts as a jump point within a recursive protocol definition.
/// It indicates that the protocol should continue from the beginning of the
/// `GRec` definition that shares the same `Label`.
///
/// # Type Parameters
///
/// * `Label` - The unique type identifying the `GRec` definition to jump back to.
///             Must match the `Label` used in the corresponding `GRec`.
///
/// # Examples
///
/// See the example for `GRec`. `GVar<PingPongLoop>` is used inside the `GRec`
/// definition to create the loop.
pub struct GVar<Label>(PhantomData<Label>);

impl<Label> GVar<Label> {
    /// Creates a new `GVar` protocol step.
    pub fn new() -> Self {
        GVar(PhantomData)
    }

    /// Conceptually returns the label type name for debugging.
    ///
    /// **Note:** This method uses `std::any::type_name` to get a string representation
    /// of the `Label` type. It doesn't provide access to the label instance itself.
    pub fn label_type(&self) -> &'static str {
        std::any::type_name::<Label>()
    }
}

impl<Label> GlobalProtocol for GVar<Label> {
    fn protocol_name(&self) -> &'static str {
        "GVar"
    }

    /// Validates the `GVar` protocol step.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It always returns `Ok(())`.
    ///
    /// A complete implementation, likely performed during the validation of an
    /// enclosing `GRec`, would need to:
    /// 1.  Check that there is a corresponding `GRec<Label, _>` with a matching
    ///     `Label` in an enclosing scope.
    /// 2.  Contribute to the overall validation of recursion well-formedness
    ///     (e.g., productivity checks).
    fn validate(&self) -> Result<()> {
        // TODO: Implement validation for GVar.
        // This validation typically happens in the context of validating the
        // enclosing GRec. It needs to ensure the GVar refers to a valid,
        // defined GRec label.
        Ok(())
    }

    /// Returns the roles involved in the protocol referenced by this variable.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It always returns an
    /// empty vector (`vec![]`).
    ///
    /// A complete implementation would need to:
    /// 1.  Identify the corresponding `GRec<Label, Protocol>`.
    /// 2.  Return the result of calling `involved_roles()` on that `Protocol`.
    /// This requires context about the enclosing `GRec` definitions.
    fn involved_roles(&self) -> Vec<&'static str> {
        // TODO: Implement role collection for GVar.
        // This requires context to find the corresponding GRec<Label, P>
        // and then returning P::involved_roles().
        vec![]
    }
}

/// Represents sequential composition: execute `First`, then execute `Second`.
///
/// The `GSeq<First, Second>` type models protocols where one sequence of interactions
/// (`First`) must complete before the next sequence (`Second`) begins.
///
/// # Type Parameters
///
/// * `First` - The global protocol to execute first. Must implement `GlobalProtocol`.
/// * `Second` - The global protocol to execute after `First` completes. Must implement `GlobalProtocol`.
///
/// # Examples
///
/// ```rust
/// use sessrums::proto::global::*;
/// use sessrums::proto::roles::{RoleA, RoleB};
/// use std::marker::PhantomData;
///
/// // Define roles
/// #[derive(Default)] struct Client; impl Role for Client { fn name(&self) -> &'static str { "Client" } }
/// #[derive(Default)] struct Server; impl Role for Server { fn name(&self) -> &'static str { "Server" } }
///
/// // Protocol: Client sends String, then Server sends i32.
/// type RequestResponse = GSeq<
///     GSend<String, Client, Server, GEnd>, // First part: Client -> Server
///     GRecv<i32, Server, Client, GEnd>     // Second part: Server -> Client
/// >;
/// // Note: GEnd is used here because each part conceptually ends before the next starts.
/// // A more accurate representation might involve linking the 'Next' types,
/// // but GSeq focuses on composing *complete* sub-protocols sequentially.
/// // The builder `seq` method handles this more naturally.
///
/// let builder = GlobalProtocolBuilder::new();
/// let protocol = builder.seq(
///     builder.send::<String, Client, Server, GEnd>(), // Send request
///     builder.recv::<i32, Server, Client, GEnd>()     // Receive response
/// );
///
/// assert_eq!(protocol.protocol_name(), "GSeq");
/// // Note: Validation and role involvement for GSeq are currently placeholders.
/// ```
pub struct GSeq<First, Second>(PhantomData<(First, Second)>);

impl<First, Second> GSeq<First, Second> {
    /// Creates a new `GSeq` protocol step.
    pub fn new() -> Self {
        GSeq(PhantomData)
    }
}

impl<First, Second> GlobalProtocol for GSeq<First, Second>
where
    First: GlobalProtocol,
    Second: GlobalProtocol,
{
    fn protocol_name(&self) -> &'static str {
        "GSeq"
    }

    /// Validates the `GSeq` protocol step.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It always returns `Ok(())`.
    ///
    /// A complete implementation would need to recursively call `validate()` on
    /// both the `First` and `Second` protocols.
    fn validate(&self) -> Result<()> {
        // TODO: Implement recursive validation for GSeq.
        // Should call First::validate() and Second::validate().
        Ok(())
    }

    /// Returns the roles involved in the sequential composition.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It always returns an
    /// empty vector (`vec![]`).
    ///
    /// A complete implementation would need to:
    /// 1.  Call `involved_roles()` on both `First` and `Second`.
    /// 2.  Combine the results into a single, deduplicated list of roles.
    fn involved_roles(&self) -> Vec<&'static str> {
        // TODO: Implement recursive role collection for GSeq.
        // Should combine roles from First::involved_roles() and Second::involved_roles().
        vec![]
    }
}

/// Represents parallel composition: execute `First` and `Second` concurrently.
///
/// The `GPar<First, Second>` type models protocols where two independent sequences
/// of interactions (`First` and `Second`) can happen concurrently or interleaved.
/// This requires that the sets of roles involved in `First` and `Second` are disjoint,
/// although this check is not currently implemented in `validate`.
///
/// # Type Parameters
///
/// * `First` - The first global protocol to execute in parallel. Must implement `GlobalProtocol`.
/// * `Second` - The second global protocol to execute in parallel. Must implement `GlobalProtocol`.
///
/// # Examples
///
/// ```rust
/// use sessrums::proto::global::*;
/// use sessrums::proto::roles::{RoleA, RoleB, RoleC};
/// use std::marker::PhantomData;
///
/// // Define roles
/// #[derive(Default)] struct A; impl Role for A { fn name(&self) -> &'static str { "A" } }
/// #[derive(Default)] struct B; impl Role for B { fn name(&self) -> &'static str { "B" } }
/// #[derive(Default)] struct C; impl Role for C { fn name(&self) -> &'static str { "C" } }
///
/// // Protocol: A sends String to B, *in parallel with* A sending i32 to C.
/// type ParallelSend = GPar<
///     GSend<String, A, B, GEnd>, // First parallel branch: A -> B
///     GSend<i32, A, C, GEnd>     // Second parallel branch: A -> C
/// >;
///
/// let builder = GlobalProtocolBuilder::new();
/// let protocol = builder.par(
///     builder.send::<String, A, B, GEnd>(),
///     builder.send::<i32, A, C, GEnd>()
/// );
///
/// assert_eq!(protocol.protocol_name(), "GPar");
/// // Note: Validation (incl. role disjointness) and role involvement for GPar
/// // are currently placeholders.
/// ```
pub struct GPar<First, Second>(PhantomData<(First, Second)>);

impl<First, Second> GPar<First, Second> {
    /// Creates a new `GPar` protocol step.
    pub fn new() -> Self {
        GPar(PhantomData)
    }
}

impl<First, Second> GlobalProtocol for GPar<First, Second>
where
    First: GlobalProtocol,
    Second: GlobalProtocol,
{
    fn protocol_name(&self) -> &'static str {
        "GPar"
    }

    /// Validates the `GPar` protocol step.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It always returns `Ok(())`.
    ///
    /// A complete implementation would need to:
    /// 1.  Recursively call `validate()` on both `First` and `Second`.
    /// 2.  **Crucially:** Check that the sets of roles involved in `First` and
    ///     `Second` are disjoint (except potentially for shared roles that only
    ///     participate passively, depending on the exact semantics). This check
    ///     requires access to the involved roles.
    fn validate(&self) -> Result<()> {
        // TODO: Implement recursive validation for GPar.
        // Should call First::validate() and Second::validate().
        // TODO: Implement role disjointness check between First and Second.
        // Requires involved_roles() to be implemented correctly first.
        Ok(())
    }

    /// Returns the roles involved in the parallel composition.
    ///
    /// # Current Implementation Status
    ///
    /// This implementation is currently a **placeholder**. It always returns an
    /// empty vector (`vec![]`).
    ///
    /// A complete implementation would need to:
    /// 1.  Call `involved_roles()` on both `First` and `Second`.
    /// 2.  Combine the results into a single, deduplicated list of roles.
    fn involved_roles(&self) -> Vec<&'static str> {
        // TODO: Implement recursive role collection for GPar.
        // Should combine roles from First::involved_roles() and Second::involved_roles().
        vec![]
    }
}

/// Represents the successful termination of a global protocol path.
///
/// `GEnd` signifies that a particular branch or the entire protocol has completed
/// its interactions according to the specification.
#[derive(Default)]
pub struct GEnd;

impl GEnd {
    /// Creates a new `GEnd` protocol step.
    pub fn new() -> Self {
        GEnd
    }
}

impl GlobalProtocol for GEnd {
    fn protocol_name(&self) -> &'static str {
        "GEnd"
    }

    /// Validates the `GEnd` protocol step.
    ///
    /// `GEnd` is always considered valid.
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Returns the roles involved in `GEnd`.
    ///
    /// `GEnd` involves no roles, so this returns an empty vector.
    fn involved_roles(&self) -> Vec<&'static str> {
        vec![]
    }
}

/// A helper struct providing methods to build global protocol types fluently.
///
/// This builder simplifies the construction of complex protocol types, which can
/// become deeply nested and verbose when written out manually.
pub struct GlobalProtocolBuilder;

impl GlobalProtocolBuilder {
    /// Creates a new `GlobalProtocolBuilder`.
    pub fn new() -> Self {
        GlobalProtocolBuilder
    }

    /// Creates a `GSend` protocol step using type inference where possible.
    ///
    /// Note: The `Next` protocol type often needs to be specified or inferred
    /// from the context where the result is used.
    pub fn send<T, From: Role, To: Role, Next: GlobalProtocol>(&self) -> GSend<T, From, To, Next> {
        GSend::new()
    }

    /// Creates a `GRecv` protocol step using type inference where possible.
    ///
    /// Note: The `Next` protocol type often needs to be specified or inferred
    /// from the context where the result is used.
    pub fn recv<T, From: Role, To: Role, Next: GlobalProtocol>(&self) -> GRecv<T, From, To, Next> {
        GRecv::new()
    }

    /// Creates a `GChoice` protocol step with the given branches.
    pub fn choice<Chooser: Role, Branches: GlobalProtocolBranches>(&self, branches: Branches) -> GChoice<Chooser, Branches> {
        GChoice::new(branches)
    }

    /// Creates a `GOffer` protocol step with the given branches.
    pub fn offer<Offeree: Role, Branches: GlobalProtocolBranches>(&self, branches: Branches) -> GOffer<Offeree, Branches> {
        GOffer::new(branches)
    }

    /// Creates a `GRec` protocol step using type inference.
    ///
    /// The `Label` and `Protocol` types must be specified or inferred.
    pub fn rec<Label, Protocol: GlobalProtocol>(&self) -> GRec<Label, Protocol> {
        GRec::new()
    }

    /// Creates a `GVar` protocol step using type inference.
    ///
    /// The `Label` type must be specified or inferred.
    pub fn var<Label>(&self) -> GVar<Label> {
        GVar::new()
    }

    /// Creates a `GEnd` protocol step.
    pub fn end(&self) -> GEnd {
        GEnd::new()
    }

    /// Creates a `GSeq` protocol step for sequential composition.
    ///
    /// Takes two protocol instances (`_first`, `_second`) as arguments primarily
    /// to aid type inference for `First` and `Second`. The arguments themselves
    /// are not stored.
    pub fn seq<First: GlobalProtocol, Second: GlobalProtocol>(&self, _first: First, _second: Second) -> GSeq<First, Second> {
        GSeq::new()
    }

    /// Creates a `GPar` protocol step for parallel composition.
    ///
    /// Takes two protocol instances (`_first`, `_second`) as arguments primarily
    /// to aid type inference for `First` and `Second`. The arguments themselves
    /// are not stored.
    pub fn par<First: GlobalProtocol, Second: GlobalProtocol>(&self, _first: First, _second: Second) -> GPar<First, Second> {
        GPar::new()
    }
}

/// Validates the structure of a given global protocol.
///
/// This function delegates to the `validate` method of the provided `protocol`
/// instance, which implements the `GlobalProtocol` trait.
///
/// # Current Implementation Status
///
/// The effectiveness of this function depends entirely on the completeness of the
/// `validate` implementation for the specific type `G`. As noted in the documentation
/// for types like `GSend`, `GRec`, `GSeq`, etc., their `validate` methods may
/// currently be **placeholders** and not perform full validation.
pub fn validate_global_protocol<G: GlobalProtocol>(protocol: &G) -> Result<()> {
    protocol.validate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::roles::{RoleA, RoleB};
    
    // Define some test roles
    #[derive(Default)]
    struct Client;
    
    #[derive(Default)]
    struct Server;
    
    #[derive(Default)]
    struct Logger;
    
    impl Role for Client {
        fn name(&self) -> &'static str {
            "Client"
        }
    }
    
    impl Role for Server {
        fn name(&self) -> &'static str {
            "Server"
        }
    }
    
    impl Role for Logger {
        fn name(&self) -> &'static str {
            "Logger"
        }
    }
    
    #[test]
    fn test_gsend_protocol_name() {
        let protocol = GSend::<String, Client, Server, GEnd>::new();
        assert_eq!(protocol.protocol_name(), "GSend");
    }
    
    #[test]
    fn test_grecv_protocol_name() {
        let protocol = GRecv::<i32, Server, Client, GEnd>::new();
        assert_eq!(protocol.protocol_name(), "GRecv");
    }
    
    #[test]
    fn test_gchoice_protocol_name() {
        let protocol = GChoice::<Client, (GEnd,)>::new((GEnd::new(),));
        assert_eq!(protocol.protocol_name(), "GChoice");
    }
    
    #[test]
    fn test_goffer_protocol_name() {
        let protocol = GOffer::<Server, (GEnd,)>::new((GEnd::new(),));
        assert_eq!(protocol.protocol_name(), "GOffer");
    }
    
    #[test]
    fn test_grec_protocol_name() {
        let protocol = GRec::<(), GEnd>::new();
        assert_eq!(protocol.protocol_name(), "GRec");
    }
    
    #[test]
    fn test_gvar_protocol_name() {
        let protocol = GVar::<()>::new();
        assert_eq!(protocol.protocol_name(), "GVar");
    }
    
    #[test]
    fn test_gend_protocol_name() {
        let protocol = GEnd::new();
        assert_eq!(protocol.protocol_name(), "GEnd");
    }
    
    #[test]
    fn test_global_protocol_builder() {
        let builder = GlobalProtocolBuilder::new();
        
        // Build a simple protocol: Client sends a String to Server, then ends
        let protocol = builder.send::<String, Client, Server, GEnd>();
        assert_eq!(protocol.protocol_name(), "GSend");
        
        // Build a more complex protocol: Client sends a String to Server,
        // Server sends an i32 back to Client, then ends
        type ComplexProtocol = GSend<String, Client, Server, GRecv<i32, Server, Client, GEnd>>;
        
        // This would be built using the builder like:
        // let protocol: ComplexProtocol = builder.send::<String, Client, Server, _>(
        //     builder.recv::<i32, Server, Client, _>(
        //         builder.end()
        //     )
        // );
        
        // For now, we'll just create it directly
        let _protocol = GSend::<String, Client, Server, GRecv<i32, Server, Client, GEnd>>::new();
    }
    
    #[test]
    fn test_validate_global_protocol() {
        // Test a valid protocol
        let protocol = GSend::<String, Client, Server, GEnd>::new();
        assert!(validate_global_protocol(&protocol).is_ok());
        
        // Test validation of involved roles
        let roles = protocol.involved_roles();
        assert!(roles.contains(&"Client"));
        assert!(roles.contains(&"Server"));
    }
    
    #[test]
    fn test_gchoice_validation() {
        // Test that GChoice validation works correctly
        
        // Create a GChoice with valid branches
        let branches = (
            GSend::<String, Client, Server, GEnd>::new(),
            GSend::<i32, Client, Server, GEnd>::new()
        );
        let protocol = GChoice::<Client, _>::new(branches);
        
        // Validate it
        let result = validate_global_protocol(&protocol);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_goffer_validation() {
        // Test that GOffer validation works correctly
        
        // Create a GOffer with valid branches
        let branches = (
            GRecv::<String, Client, Server, GEnd>::new(),
            GRecv::<i32, Client, Server, GEnd>::new()
        );
        let protocol = GOffer::<Server, _>::new(branches);
        
        // Validate it
        let result = validate_global_protocol(&protocol);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_gchoice_involved_roles() {
        // Test that GChoice involved_roles works correctly
        
        // Create a GChoice with branches involving different roles
        let branches = (
            GSend::<String, Client, Server, GEnd>::new(),
            GSend::<i32, Client, Logger, GEnd>::new()
        );
        let protocol = GChoice::<Client, _>::new(branches);
        
        // Get involved roles
        let roles = protocol.involved_roles();
        
        // Should include Client (chooser), Server, and Logger
        assert!(roles.contains(&"Client"));
        assert!(roles.contains(&"Server"));
        assert!(roles.contains(&"Logger"));
        assert_eq!(roles.len(), 3); // No duplicates
    }
    
    #[test]
    fn test_goffer_involved_roles() {
        // Test that GOffer involved_roles works correctly
        
        // Create a GOffer with branches involving different roles
        let branches = (
            GRecv::<String, Client, Server, GEnd>::new(),
            GRecv::<i32, Logger, Server, GEnd>::new()
        );
        let protocol = GOffer::<Server, _>::new(branches);
        
        // Get involved roles
        let roles = protocol.involved_roles();
        
        // Should include Server (offeree), Client, and Logger
        assert!(roles.contains(&"Server"));
        assert!(roles.contains(&"Client"));
        assert!(roles.contains(&"Logger"));
        assert_eq!(roles.len(), 3); // No duplicates
    }
    
    #[test]
    fn test_gseq_protocol_name() {
        let protocol = GSeq::<GSend<String, Client, Server, GEnd>, GRecv<i32, Server, Client, GEnd>>::new();
        assert_eq!(protocol.protocol_name(), "GSeq");
    }
    
    #[test]
    fn test_gpar_protocol_name() {
        let protocol = GPar::<GSend<String, Client, Server, GEnd>, GSend<bool, Client, Logger, GEnd>>::new();
        assert_eq!(protocol.protocol_name(), "GPar");
    }
    
    #[test]
    fn test_global_protocol_builder_composition() {
        let builder = GlobalProtocolBuilder::new();
        
        // Test sequential composition
        let seq_protocol = builder.seq(
            GSend::<String, Client, Server, GEnd>::new(),
            GRecv::<i32, Server, Client, GEnd>::new()
        );
        assert_eq!(seq_protocol.protocol_name(), "GSeq");
        
        // Test parallel composition
        let par_protocol = builder.par(
            GSend::<String, Client, Server, GEnd>::new(),
            GSend::<bool, Client, Logger, GEnd>::new()
        );
        assert_eq!(par_protocol.protocol_name(), "GPar");
    }
}