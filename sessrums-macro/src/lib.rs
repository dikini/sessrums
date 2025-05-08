//! Procedural macro implementation for the MPST DSL.
//!
//! This crate provides a macro for defining multiparty session type protocols
//! using a Mermaid-like Domain-Specific Language (DSL) syntax, making it easier
//! to create and understand complex communication protocols.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, Token, parse::{Parse, ParseStream}, Result as SynResult, Error, braced};
use syn::{Type};
use proc_macro2::Span;
use std::collections::{HashSet, HashMap};
use std::fmt;

/// Error categories for better organization and reporting of errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorCategory {
    Syntax,        // Basic syntax errors
    Participant,   // Errors related to participant definitions
    Recursion,     // Errors related to recursion labels
    Type,          // Errors related to message types
    Semantic,      // Higher-level semantic errors
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Syntax => write!(f, "Syntax error"),
            ErrorCategory::Participant => write!(f, "Participant error"),
            ErrorCategory::Recursion => write!(f, "Recursion error"),
            ErrorCategory::Type => write!(f, "Type error"),
            ErrorCategory::Semantic => write!(f, "Semantic error"),
        }
    }
}

/// Context for an error with additional information
struct ErrorContext {
    category: ErrorCategory,
    error: Error,
    suggestion: Option<String>, // Optional suggestion for fixing the error
}

impl ErrorContext {
    /// Create a new error context
    fn new(category: ErrorCategory, error: Error, suggestion: Option<String>) -> Self {
        Self {
            category,
            error,
            suggestion,
        }
    }
    
    /// Convert to a syn::Error with enhanced message including the suggestion if available
    fn to_error(&self) -> Error {
        let mut message = format!("{}: {}", self.category, self.error);
        if let Some(suggestion) = &self.suggestion {
            message = format!("{}\n= help: {}", message, suggestion);
        }
        Error::new(self.error.span(), message)
    }
}

/// Report an error with a specific span and message
fn report_error(span: Span, message: &str) -> Error {
    Error::new(span, message)
}

/// Report an error with category and optional suggestion
fn report_categorized_error(
    span: Span,
    category: ErrorCategory,
    message: &str,
    suggestion: Option<&str>
) -> Error {
    let error = Error::new(span, message);
    let context = ErrorContext::new(
        category,
        error,
        suggestion.map(|s| s.to_string())
    );
    context.to_error()
}

/// Helper functions for common error patterns

/// Report an undefined participant error
fn report_undefined_participant(span: Span, name: &str) -> Error {
    report_categorized_error(
        span,
        ErrorCategory::Participant,
        &format!("Undefined participant '{}'. All participants must be declared at the beginning of the protocol.", name),
        Some(&format!("Add 'participant {};' to the beginning of the protocol", name))
    )
}

/// Report an undefined recursion label error
fn report_undefined_recursion_label(span: Span, label: &str) -> Error {
    report_categorized_error(
        span,
        ErrorCategory::Recursion,
        &format!("Undefined recursion label '{}'. Labels must be defined with 'rec {}' before they can be referenced with 'continue {}'.", label, label, label),
        Some(&format!("Define the recursion label with 'rec {} {{ ... }}' before using 'continue {}'", label, label))
    )
}

/// Report a duplicate recursion label error
fn report_duplicate_recursion_label(span: Span, label: &str) -> Error {
    report_categorized_error(
        span,
        ErrorCategory::Recursion,
        &format!("Duplicate recursion label '{}'. Each recursion label must be unique within its scope.", label),
        Some("Choose a different label name for this recursion block")
    )
}

/// Report an invalid message type error
fn report_invalid_message_type(span: Span, type_str: &str) -> Error {
    report_categorized_error(
        span,
        ErrorCategory::Type,
        &format!("Invalid message type '{}'. Message types must be valid Rust types.", type_str),
        Some("Use a fully qualified path or import the type with 'use'")
    )
}

/// Report an invalid choice syntax error
fn report_invalid_choice_syntax(span: Span) -> Error {
    report_categorized_error(
        span,
        ErrorCategory::Syntax,
        "Invalid choice syntax. Expected 'choice at Role { option Label { ... } ... }'.",
        Some("Use 'choice at Role { ... }' syntax")
    )
}

/// Report an invalid role in choice branch error
fn report_invalid_role_in_choice(span: Span, role: &str, deciding_role: &str) -> Error {
    report_categorized_error(
        span,
        ErrorCategory::Semantic,
        &format!("Role '{}' cannot make a choice in a branch where '{}' is the deciding role.", role, deciding_role),
        Some("In a choice block, the first message in each branch must be sent by the deciding role")
    )
}

/// Report an unreachable code after continue error
fn report_unreachable_code(span: Span) -> Error {
    report_categorized_error(
        span,
        ErrorCategory::Semantic,
        "Unreachable code after 'continue' statement",
        Some("Remove code after 'continue' statement or move it before the 'continue'")
    )
}

/// Report a mismatched recursion label error
fn report_mismatched_recursion_label(span: Span, expected: &str, found: &str) -> Error {
    report_categorized_error(
        span,
        ErrorCategory::Recursion,
        &format!("Mismatched recursion labels: expected '{}', found '{}'", expected, found),
        Some(&format!("Change to 'rec {}' to match the label", expected))
    )
}

/// A macro for defining multiparty session type protocols using a Mermaid-like DSL syntax.
///
/// This macro transforms textual protocol definitions into Rust code that constructs
/// the `GlobalInteraction` enum structure. It provides a concise, readable syntax for
/// defining complex communication protocols while maintaining the strong type guarantees
/// of the underlying system.
///
/// # Features
///
/// - **Readability**: The DSL syntax resembles sequence diagrams, making protocols easier to understand
/// - **Conciseness**: Complex protocols can be defined with minimal boilerplate
/// - **Compile-time Verification**: Protocol errors are caught during compilation
/// - **Comprehensive Error Reporting**: Clear, actionable error messages for syntax and semantic issues
///
/// # Syntax
///
/// The basic syntax includes:
/// - Protocol declaration: `protocol Name { ... }`
/// - Participant declaration: `participant Role;` or `participant Role as Alias;`
/// - Message passing: `Role1 -> Role2: MessageType;`
/// - Choice: `choice at Role { option Label { ... } or { ... } }`
/// - Recursion: `rec Label { ... continue Label; }`
/// - End: `end;`
///
/// # Example
///
/// ```
/// use sessrums_macro::mpst;
/// use sessrums_types::roles::{Client, Server};
///
/// mpst! {
///     protocol PingPong {
///         participant Client;
///         participant Server;
///
///         Client -> Server: String;
///         Server -> Client: String;
///         end;
///     }
/// }
///
/// // Use the generated protocol type
/// let protocol = PingPong::new();
/// ```
///
/// # Advanced Example with Choice and Recursion
///
/// ```
/// use sessrums_macro::mpst;
/// use sessrums_types::roles::{Client, Server};
///
/// mpst! {
///     protocol FileTransfer {
///         participant Client;
///         participant Server;
///
///         rec Loop {
///             Client -> Server: String; // Filename
///
///             choice at Server {
///                 option FileExists {
///                     Server -> Client: Vec<u8>; // File content
///                     continue Loop;
///                 }
///                 or {
///                     Server -> Client: String; // Error message
///                     end;
///                 }
///             }
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn mpst(input: TokenStream) -> TokenStream {
    // Parse the input tokens into our protocol definition
    let protocol_def = match syn::parse::<ProtocolDefinition>(input.clone()) {
        Ok(def) => def,
        Err(err) => {
            // Handle parsing errors with improved error reporting
            let error = report_categorized_error(
                err.span(),
                ErrorCategory::Syntax,
                &format!("Failed to parse protocol definition: {}", err),
                Some("Check the syntax of your protocol definition")
            );
            return TokenStream::from(error.to_compile_error());
        }
    };
    
    // Validate the protocol definition
    match protocol_def.validate() {
        Ok(_) => {
            // Generate code for the protocol definition
            let output = protocol_def.generate_code();
            
            TokenStream::from(output)
        },
        Err(errors) => {
            // Format each error with its category and suggestion
            let formatted_errors: Vec<Error> = errors.into_iter()
                .map(|e| {
                    // If the error is already formatted (from our helper functions), use it as is
                    // Otherwise, wrap it in a basic error context
                    if e.to_string().contains("= help:") {
                        e
                    } else {
                        ErrorContext::new(
                            ErrorCategory::Semantic,
                            e,
                            None
                        ).to_error()
                    }
                })
                .collect();
            
            // Combine all errors into a single compile error
            let mut combined_error = None;
            
            for error in formatted_errors {
                if let Some(prev_error) = combined_error {
                    combined_error = Some(Error::new(
                        error.span(),
                        format!("{}\n{}", prev_error, error)
                    ));
                } else {
                    combined_error = Some(error);
                }
            }
            
            // Return the error as a compile error
            TokenStream::from(combined_error.unwrap_or_else(|| {
                Error::new(Span::call_site(), "Unknown validation error occurred")
            }).to_compile_error())
        }
    }
}

/// Track recursion context during parsing and validation
struct RecursionContext {
    /// Stack of active recursion labels to detect nested recursion
    active_labels: Vec<Ident>,
    /// Map of label to its scope (for validation)
    label_scopes: HashMap<String, Span>,
    /// Track recursion depth for potential optimizations
    depth: usize,
}

impl RecursionContext {
    /// Create a new recursion context
    fn new() -> Self {
        Self {
            active_labels: Vec::new(),
            label_scopes: HashMap::new(),
            depth: 0,
        }
    }

    /// Enter a recursion scope with the given label
    fn enter_recursion(&mut self, label: &Ident) -> Result<(), syn::Error> {
        // Check if label is already active (detect invalid nested recursion with same label)
        if self.active_labels.iter().any(|l| l.to_string() == label.to_string()) {
            return Err(report_categorized_error(
                label.span(),
                ErrorCategory::Recursion,
                &format!("Recursive label '{}' is already active in this scope. Nested recursion with the same label is not allowed.", label),
                Some("Use a different label for nested recursion blocks")
            ));
        }
        
        // Add label to active set
        self.active_labels.push(label.clone());
        self.label_scopes.insert(label.to_string(), label.span());
        self.depth += 1;
        
        Ok(())
    }
    
    /// Exit a recursion scope with the given label
    fn exit_recursion(&mut self, label: &Ident) -> Result<(), syn::Error> {
        // Ensure we're exiting the most recently entered recursion
        if let Some(last_label) = self.active_labels.last() {
            if last_label.to_string() != label.to_string() {
                return Err(report_categorized_error(
                    label.span(),
                    ErrorCategory::Recursion,
                    &format!("Mismatched recursion labels: expected '{}', found '{}'",
                        last_label, label),
                    Some(&format!("Change to 'rec {}' to match the label", last_label))
                ));
            }
            
            self.active_labels.pop();
            self.depth = self.depth.saturating_sub(1);
            Ok(())
        } else {
            Err(report_categorized_error(
                label.span(),
                ErrorCategory::Recursion,
                "Exiting recursion without matching 'rec' block",
                Some("Make sure each recursion block is properly opened with 'rec' before closing")
            ))
        }
    }
    
    /// Check if a continue statement refers to a valid active recursion label
    fn check_continue(&self, label: &Ident) -> Result<(), syn::Error> {
        // Check if the label is in the active set
        if !self.active_labels.iter().any(|l| l.to_string() == label.to_string()) {
            return Err(report_categorized_error(
                label.span(),
                ErrorCategory::Recursion,
                &format!("Continue statement refers to label '{}' which is not active in this scope", label),
                Some("Make sure the continue statement is within the scope of the referenced recursion label")
            ));
        }
        
        Ok(())
    }
    
    /// Clone the recursion context for use in a branch
    fn clone_for_branch(&self) -> Self {
        Self {
            active_labels: self.active_labels.clone(),
            label_scopes: self.label_scopes.clone(),
            depth: self.depth,
        }
    }
    
    /// Get the current recursion depth
    fn get_depth(&self) -> usize {
        self.depth
    }
    
    /// Check if a label is active in the current scope
    fn is_label_active(&self, label: &str) -> bool {
        self.active_labels.iter().any(|l| l.to_string() == label)
    }
}

/// Validation context to track state during validation.
struct ValidationContext {
    /// Set of defined participant names
    participants: HashSet<String>,
    /// Set of defined recursion labels
    recursion_labels: HashSet<String>,
    /// Map of recursion label to its span for error reporting
    label_spans: HashMap<String, Span>,
    /// Stack of active recursion labels to track scope
    active_recursion_labels: Vec<String>,
    /// Specialized recursion context for complex nested structures
    recursion_context: RecursionContext,
}

impl ValidationContext {
    /// Create a new validation context with the given participants
    fn new(participants: &[ParticipantDefinition]) -> Self {
        let mut context = ValidationContext {
            participants: HashSet::new(),
            recursion_labels: HashSet::new(),
            label_spans: HashMap::new(),
            active_recursion_labels: Vec::new(),
            recursion_context: RecursionContext::new(),
        };
        
        // Add all participants to the context
        for participant in participants {
            context.participants.insert(participant.name.to_string());
        }
        
        context
    }
    
    /// Check if a participant is defined
    fn is_participant_defined(&self, name: &str) -> bool {
        self.participants.contains(name)
    }
    
    /// Check if a recursion label is defined
    fn is_recursion_label_defined(&self, label: &str) -> bool {
        self.recursion_labels.contains(label)
    }
    
    /// Add a recursion label to the context
    fn add_recursion_label(&mut self, label: &Ident) -> SynResult<()> {
        let label_str = label.to_string();
        
        // Check if the label is already defined
        if self.recursion_labels.contains(&label_str) {
            return Err(report_duplicate_recursion_label(label.span(), &label_str));
        }
        
        // Add the label to the context
        self.recursion_labels.insert(label_str.clone());
        self.label_spans.insert(label_str.clone(), label.span());
        self.active_recursion_labels.push(label_str);
        
        // Also update the specialized recursion context
        self.recursion_context.enter_recursion(label)?;
        
        Ok(())
    }
    
    /// Remove a recursion label from the active set (when exiting a recursion block)
    fn remove_recursion_label(&mut self, label: &Ident) -> SynResult<()> {
        let label_str = label.to_string();
        
        // Check if the label is the most recently added one
        if let Some(active_label) = self.active_recursion_labels.last() {
            if active_label != &label_str {
                return Err(report_mismatched_recursion_label(
                    label.span(),
                    active_label,
                    &label_str
                ));
            }
            
            self.active_recursion_labels.pop();
            
            // Also update the specialized recursion context
            self.recursion_context.exit_recursion(label)?;
            
            Ok(())
        } else {
            Err(report_categorized_error(
                label.span(),
                ErrorCategory::Recursion,
                "Exiting recursion without matching 'rec' block",
                Some("Make sure each recursion block is properly opened with 'rec' before closing")
            ))
        }
    }
    
    /// Check if a continue statement refers to a valid recursion label
    fn check_continue(&self, label: &Ident) -> SynResult<()> {
        let label_str = label.to_string();
        
        // Check if the label is defined
        if !self.recursion_labels.contains(&label_str) {
            return Err(report_undefined_recursion_label(label.span(), &label_str));
        }
        
        // Check if the label is in the active set using the specialized recursion context
        self.recursion_context.check_continue(label)
    }
    
    /// Create a clone of the context for use in a branch
    fn clone_for_branch(&self) -> Self {
        ValidationContext {
            participants: self.participants.clone(),
            recursion_labels: self.recursion_labels.clone(),
            label_spans: self.label_spans.clone(),
            active_recursion_labels: self.active_recursion_labels.clone(),
            recursion_context: self.recursion_context.clone_for_branch(),
        }
    }
}

/// Represents a protocol definition parsed from the DSL.
struct ProtocolDefinition {
    name: Ident,
    participants: Vec<ParticipantDefinition>,
    body: ProtocolBody,
}

impl ProtocolDefinition {
    /// Generate Rust code for the protocol definition
    fn generate_code(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let body = self.body.generate_code();
        
        // Extract all message types used in the protocol
        let message_types = self.collect_message_types();
        
        // If there are multiple message types, we'll need to use a common type
        // For now, we'll use the first message type or () if none are found
        let message_type = if message_types.is_empty() {
            quote! { () }
        } else {
            let first_type = &message_types[0];
            quote! { #first_type }
        };
        
        quote! {
            type #name = ::sessrums_types::session_types::global::GlobalInteraction<#message_type>;
            
            impl #name {
                pub fn new() -> Self {
                    #body
                }
            }
        }
    }
    
    /// Collect all message types used in the protocol
    fn collect_message_types(&self) -> Vec<Type> {
        let mut types = Vec::new();
        self.body.collect_message_types(&mut types);
        types
    }
}

/// Custom result type for validation that can return multiple errors
type ValidationResult = std::result::Result<(), Vec<Error>>;

impl ProtocolDefinition {
    /// Validate the protocol definition
    fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut context = ValidationContext::new(&self.participants);
        
        // Check for duplicate participant names
        let mut seen_participants = HashSet::new();
        for participant in &self.participants {
            let name = participant.name.to_string();
            if !seen_participants.insert(name.clone()) {
                errors.push(report_categorized_error(
                    participant.name.span(),
                    ErrorCategory::Participant,
                    &format!("Duplicate participant '{}'. Each participant must be declared exactly once.", name),
                    Some("Remove or rename the duplicate participant declaration")
                ));
            }
        }
        
        // Validate the protocol body
        if let Err(body_errors) = self.body.validate(&mut context) {
            errors.extend(body_errors);
        }
        
        // Check if any recursion labels were left open
        if !context.active_recursion_labels.is_empty() {
            for label in &context.active_recursion_labels {
                if let Some(span) = context.label_spans.get(label) {
                    errors.push(report_categorized_error(
                        *span,
                        ErrorCategory::Recursion,
                        &format!("Unclosed recursion block for label '{}'", label),
                        Some("Make sure each 'rec' block is properly closed")
                    ));
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Represents a participant in the protocol.
struct ParticipantDefinition {
    name: Ident,
    alias: Option<Ident>,
}

/// Represents the body of a protocol.
///
/// The protocol body consists of a sequence of interactions that define the communication
/// flow between participants. The interactions are parsed recursively, with each interaction
/// potentially containing nested interactions.
enum ProtocolBody {
    Interaction(Box<Interaction>),
}

impl ProtocolBody {
    /// Generate Rust code for the protocol body
    fn generate_code(&self) -> proc_macro2::TokenStream {
        match self {
            ProtocolBody::Interaction(interaction) => interaction.generate_code(),
        }
    }
}

impl ProtocolBody {
    /// Validate the protocol body
    fn validate(&self, context: &mut ValidationContext) -> ValidationResult {
        match self {
            ProtocolBody::Interaction(interaction) => {
                interaction.validate(context)
            }
        }
    }
    
    /// Collect all message types used in the protocol body
    fn collect_message_types(&self, types: &mut Vec<Type>) {
        match self {
            ProtocolBody::Interaction(interaction) => {
                interaction.collect_message_types(types);
            }
        }
    }
}

/// Parse implementation for ProtocolBody
///
/// Parses the body of a protocol, which consists of a sequence of interactions.
/// The parsing starts with the first interaction, which may contain continuations
/// that form the rest of the protocol.
///
/// Example (in DSL syntax, not valid Rust):
/// ```text
/// protocol PingPong {
///     participant Client;
///     participant Server;
///
///     Client -> Server: String;  // First interaction
///     Server -> Client: String;  // Continuation
///     end;                       // End of protocol
/// }
/// ```
impl Parse for ProtocolBody {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Parse the first interaction in the protocol body
        // The Interaction parser will recursively parse any continuations
        let interaction = input.parse::<Interaction>()?;
        
        Ok(ProtocolBody::Interaction(Box::new(interaction)))
    }
}

/// Represents an interaction in the protocol.
enum Interaction {
    Message {
        from: Ident,
        to: Ident,
        message_type: Type,
        continuation: Box<Interaction>,
    },
    Choice {
        participant: Ident,
        branches: Vec<Branch>,
    },
    Recursion {
        label: Ident,
        body: Box<Interaction>,
    },
    Continue(Ident),
    End,
}

impl Interaction {
    /// Generate Rust code for the interaction
    fn generate_code(&self) -> proc_macro2::TokenStream {
        // Use an empty HashMap for recursion labels by default
        let recursion_labels = HashMap::new();
        self.generate_code_with_context(&recursion_labels)
    }
    
    /// Generate Rust code for the interaction with recursion context
    fn generate_code_with_context(&self, recursion_labels: &HashMap<String, Ident>) -> proc_macro2::TokenStream {
        match self {
            Interaction::Message { from, to, message_type: _, continuation } => {
                let from_str = from.to_string();
                let to_str = to.to_string();
                let cont = continuation.generate_code_with_context(recursion_labels);
                
                quote! {
                    ::sessrums_types::session_types::global::GlobalInteraction::message(
                        #from_str,
                        #to_str,
                        #cont
                    )
                }
            },
            Interaction::Choice { participant, branches } => {
                let participant_str = participant.to_string();
                // Generate code for each branch
                // We can't use generate_code_with_context directly on branches
                // because it's not visible from this context
                let branches_code = branches.iter().map(|branch| {
                    // Instead, we'll create a new method that handles this internally
                    branch.generate_code()
                });
                
                quote! {
                    ::sessrums_types::session_types::global::GlobalInteraction::choice(
                        #participant_str,
                        vec![
                            #(#branches_code),*
                        ]
                    )
                }
            },
            Interaction::Recursion { label, body } => {
                let label_str = label.to_string();
                
                // Create a new recursion context with this label
                let mut new_recursion_labels = recursion_labels.clone();
                new_recursion_labels.insert(label_str.clone(), label.clone());
                
                // Generate the body with the updated recursion context
                let body_code = body.generate_code_with_context(&new_recursion_labels);
                
                quote! {
                    ::sessrums_types::session_types::global::GlobalInteraction::rec(
                        #label_str,
                        #body_code
                    )
                }
            },
            Interaction::Continue(label) => {
                let label_str = label.to_string();
                
                // Look up the label in the recursion context if available
                if let Some(recursion_label) = recursion_labels.get(&label_str) {
                    let recursion_label_str = recursion_label.to_string();
                    quote! {
                        ::sessrums_types::session_types::global::GlobalInteraction::var(#recursion_label_str)
                    }
                } else {
                    // Fall back to the original label if not found in context
                    quote! {
                        ::sessrums_types::session_types::global::GlobalInteraction::var(#label_str)
                    }
                }
            },
            Interaction::End => {
                quote! {
                    ::sessrums_types::session_types::global::GlobalInteraction::end()
                }
            }
        }
    }
    
    /// Get the span for error reporting
    fn span(&self) -> proc_macro2::Span {
        match self {
            Interaction::Message { from, .. } => from.span(),
            Interaction::Choice { participant, .. } => participant.span(),
            Interaction::Recursion { label, .. } => label.span(),
            Interaction::Continue(label) => label.span(),
            Interaction::End => Span::call_site(),
        }
    }
    
    /// Collect all message types used in the interaction
    fn collect_message_types(&self, types: &mut Vec<Type>) {
        match self {
            Interaction::Message { message_type, continuation, .. } => {
                types.push(message_type.clone());
                continuation.collect_message_types(types);
            },
            Interaction::Choice { branches, .. } => {
                for branch in branches {
                    branch.interaction.collect_message_types(types);
                }
            },
            Interaction::Recursion { body, .. } => {
                body.collect_message_types(types);
            },
            Interaction::Continue(_) | Interaction::End => {},
        }
    }
    
    /// Validate the interaction
    fn validate(&self, context: &mut ValidationContext) -> ValidationResult {
        self.validate_with_context(context)
    }
    
    /// Validate the interaction with specialized context handling for complex nested structures
    fn validate_with_context(&self, context: &mut ValidationContext) -> ValidationResult {
        let mut errors = Vec::new();
        
        match self {
            Interaction::Message { from, to, message_type: _, continuation } => {
                // Validate that the sender is a defined participant
                if !context.is_participant_defined(&from.to_string()) {
                    errors.push(report_undefined_participant(from.span(), &from.to_string()));
                }
                
                // Validate that the receiver is a defined participant
                if !context.is_participant_defined(&to.to_string()) {
                    errors.push(report_undefined_participant(to.span(), &to.to_string()));
                }
                
                // Message type validation is handled elsewhere
                
                // Validate the continuation
                if let Err(cont_errors) = continuation.validate_with_context(context) {
                    errors.extend(cont_errors);
                }
            },
            Interaction::Choice { participant, branches } => {
                // Validate that the deciding participant is defined
                if !context.is_participant_defined(&participant.to_string()) {
                    errors.push(report_undefined_participant(participant.span(), &participant.to_string()));
                }
                
                // Validate each branch with its own context
                // This allows each branch to have its own recursion state
                for branch in branches {
                    // Create a branch context that clones the current recursion context
                    let mut branch_context = context.clone_for_branch();
                    
                    if let Err(branch_errors) = branch.validate(&mut branch_context) {
                        errors.extend(branch_errors);
                    }
                    
                    // Validate that the first message in each branch is sent by the deciding participant
                    self.validate_choice_branch(participant, &branch.interaction, &mut errors);
                }
                
                // Validate that there is at least one branch
                if branches.is_empty() {
                    errors.push(report_categorized_error(
                        participant.span(),
                        ErrorCategory::Semantic,
                        "Choice must have at least one branch",
                        Some("Add at least one branch to the choice block")
                    ));
                }
            },
            Interaction::Recursion { label, body } => {
                // Add the recursion label to the context
                if let Err(e) = context.add_recursion_label(label) {
                    errors.push(e);
                } else {
                    // Validate the body with updated context
                    if let Err(body_errors) = body.validate_with_context(context) {
                        errors.extend(body_errors);
                    }
                    
                    // Remove the recursion label from the active set
                    if let Err(e) = context.remove_recursion_label(label) {
                        errors.push(e);
                    }
                }
            },
            Interaction::Continue(label) => {
                // Validate that the recursion label is defined
                if !context.is_recursion_label_defined(&label.to_string()) {
                    errors.push(report_undefined_recursion_label(label.span(), &label.to_string()));
                } else {
                    // Check if the label is in the active set using the specialized recursion context
                    if let Err(e) = context.check_continue(label) {
                        errors.push(e);
                    }
                }
            },
            Interaction::End => {
                // End is always valid
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validate that the first message in a choice branch is sent by the deciding participant
    fn validate_choice_branch(&self, deciding_participant: &Ident, interaction: &Interaction, errors: &mut Vec<Error>) {
        match interaction {
            Interaction::Message { from, .. } => {
                // Check if the sender is the deciding participant
                if from.to_string() != deciding_participant.to_string() {
                    errors.push(report_invalid_role_in_choice(
                        from.span(),
                        &from.to_string(),
                        &deciding_participant.to_string()
                    ));
                }
            },
            Interaction::Choice { .. } => {
                // Nested choice - this is valid but unusual
                // We don't need to check further as the nested choice will be validated separately
            },
            Interaction::Recursion { body, .. } => {
                // Check the first interaction in the recursion body
                self.validate_choice_branch(deciding_participant, body, errors);
            },
            Interaction::Continue(_) => {
                // Continue is valid as the first interaction in a branch
                // The target recursion will be validated separately
            },
            Interaction::End => {
                // End is valid as the first interaction in a branch
            }
        }
    }
}

// Add specialized handling for complex nested structures
impl Interaction {
    /// Check for valid recursion patterns
    pub fn validate_recursion_patterns(&self) -> ValidationResult {
        let mut errors = Vec::new();
        
        match self {
            Interaction::Message { continuation, .. } => {
                // Check the continuation
                if let Err(cont_errors) = (**continuation).validate_recursion_patterns() {
                    errors.extend(cont_errors);
                }
            },
            Interaction::Choice { branches, .. } => {
                // Check for recursion within choice branches
                for branch in branches {
                    if let Err(branch_errors) = branch.interaction.validate_recursion_patterns() {
                        errors.extend(branch_errors);
                    }
                    
                    // Check for tail recursion optimization opportunities
                    if matches!(*branch.interaction, Interaction::Continue(_)) {
                        // This is a direct tail recursion, which is good for optimization
                    } else {
                        // Check if the last interaction in the branch is a continue
                        let has_tail_recursion = branch.interaction.has_tail_recursion();
                        if !has_tail_recursion {
                            // Not an error, but could be optimized
                            // We don't add an error here, but this could be used for optimization hints
                        }
                    }
                }
            },
            Interaction::Recursion { label, body } => {
                // Check for mutual recursion
                if (**body).has_mutual_recursion(label) {
                    // This is mutual recursion, which is more complex but valid
                    // We don't add an error here, but this could be used for optimization hints
                }
                
                // Check the body
                if let Err(body_errors) = (**body).validate_recursion_patterns() {
                    errors.extend(body_errors);
                }
            },
            Interaction::Continue(_) | Interaction::End => {
                // These are terminal nodes, no further validation needed
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Check if an interaction has tail recursion
    pub fn has_tail_recursion(&self) -> bool {
        match self {
            Interaction::Message { continuation, .. } => {
                if let Interaction::Continue(_) = &**continuation {
                    true
                } else {
                    (**continuation).has_tail_recursion()
                }
            },
            Interaction::Choice { branches, .. } => {
                // All branches must have tail recursion for this to be true
                branches.iter().all(|branch| branch.interaction.has_tail_recursion())
            },
            Interaction::Recursion { body, .. } => {
                body.has_tail_recursion()
            },
            Interaction::Continue(_) => true,
            Interaction::End => false,
        }
    }
    
    /// Check if an interaction has mutual recursion
    pub fn has_mutual_recursion(&self, target_label: &Ident) -> bool {
        match self {
            Interaction::Message { continuation, .. } => {
                continuation.has_mutual_recursion(target_label)
            },
            Interaction::Choice { branches, .. } => {
                // Check if any branch has mutual recursion
                branches.iter().any(|branch| branch.interaction.has_mutual_recursion(target_label))
            },
            Interaction::Recursion { label, body } => {
                if label.to_string() != target_label.to_string() {
                    // Different label, check if the body references the target label
                    body.has_continue_to(target_label)
                } else {
                    // Same label, not mutual recursion
                    false
                }
            },
            Interaction::Continue(label) => {
                // Check if this continue references the target label
                label.to_string() != target_label.to_string()
            },
            Interaction::End => false,
        }
    }
    
    /// Check if an interaction has a continue to the specified label
    pub fn has_continue_to(&self, target_label: &Ident) -> bool {
        match self {
            Interaction::Message { continuation, .. } => {
                continuation.has_continue_to(target_label)
            },
            Interaction::Choice { branches, .. } => {
                // Check if any branch has a continue to the target label
                branches.iter().any(|branch| branch.interaction.has_continue_to(target_label))
            },
            Interaction::Recursion { label, body } => {
                if label.to_string() != target_label.to_string() {
                    // Different label, check if the body references the target label
                    body.has_continue_to(target_label)
                } else {
                    // Same label, this shadows the target label
                    false
                }
            },
            Interaction::Continue(label) => {
                // Check if this continue references the target label
                label.to_string() == target_label.to_string()
            },
            Interaction::End => false,
        }
    }
}

/// Parse implementation for Interaction
///
/// Parses different types of interactions:
/// - Message interactions (A -> B: Msg)
/// - Choice interactions (choice at Role { option Label { ... } ... })
/// - Recursion (rec X { ... })
/// - Continue (continue X)
/// - End (end)
///
/// The parser handles nested structures by recursively parsing interactions within
/// choice branches and recursion blocks. Each interaction type has its own parsing logic:
///
/// 1. Message interaction example (in DSL syntax, not valid Rust):
///    ```text
///    Client -> Server: String;
///    Server -> Client: Response;
///    ```
///
/// 2. Choice interaction example (in DSL syntax, not valid Rust):
///    ```text
///    choice at Server {
///        option Success {
///            Server -> Client: LoginSuccess;
///            end;
///        }
///        or {
///            Server -> Client: LoginFailure;
///            end;
///        }
///    }
///    ```
///
/// 3. Recursion interaction example (in DSL syntax, not valid Rust):
///    ```text
///    rec ChatLoop {
///        choice at Client {
///            option SendMessage {
///                Client -> Server: ChatMessage;
///                Server -> Client: Acknowledgment;
///                cont ChatLoop;
///            }
///            or {
///                Client -> Server: Disconnect;
///                end;
///            }
///        }
///    }
///    ```
///
/// 4. Continue interaction example (in DSL syntax, not valid Rust):
///    ```text
///    cont ChatLoop;
///    ```
///
/// 5. End interaction example (in DSL syntax, not valid Rust):
///    ```text
///    end;
///    ```
impl Parse for Interaction {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Check which type of interaction we're parsing based on the next token
        if input.peek(kw::end) {
            // Parse end interaction: end;
            // This represents the termination of a protocol or branch
            input.parse::<kw::end>()?;
            input.parse::<Token![;]>()?;
            
            Ok(Interaction::End)
        } else if input.peek(kw::choice) {
            // Parse choice interaction: choice at Role { option Label { ... } or { ... } }
            // This represents a point where a participant makes a choice between multiple branches
            input.parse::<kw::choice>()?;
            input.parse::<kw::at>()?;
            
            // The participant who makes the choice
            let participant: Ident = input.parse()?;
            
            // Parse the branches in braces
            let content;
            braced!(content in input);
            
            let mut branches = Vec::new();
            
            // Parse the first branch
            branches.push(content.parse::<Branch>()?);
            
            // Parse additional branches separated by 'or'
            while content.peek(kw::or) {
                content.parse::<kw::or>()?;
                branches.push(content.parse::<Branch>()?);
            }
            
            Ok(Interaction::Choice {
                participant,
                branches,
            })
        } else if input.peek(kw::rec) {
            // Parse recursion interaction: rec X { ... }
            // This defines a recursion point that can be referenced later with 'cont X'
            input.parse::<kw::rec>()?;
            
            // The label for this recursion point
            let label: Ident = input.parse()?;
            
            // Parse the recursion body in braces
            let content;
            braced!(content in input);
            
            // Parse the body of the recursion block
            let body = content.parse::<Interaction>()?;
            
            Ok(Interaction::Recursion {
                label,
                body: Box::new(body),
            })
        } else if input.peek(kw::cont) {
            // Parse continue interaction: cont X;
            // This jumps back to a previously defined recursion point
            input.parse::<kw::cont>()?;
            
            // The label of the recursion point to continue to
            let label: Ident = input.parse()?;
            input.parse::<Token![;]>()?;
            
            Ok(Interaction::Continue(label))
        } else {
            // Parse message interaction: A -> B: Msg;
            // This represents a message being sent from one participant to another
            
            // The sender of the message
            let from: Ident = input.parse()?;
            
            // The arrow token '->'
            input.parse::<Token![->]>()?;
            
            // The receiver of the message
            let to: Ident = input.parse()?;
            
            // The colon token ':'
            input.parse::<Token![:]>()?;
            
            // The type of the message
            let message_type: Type = input.parse()?;
            
            // The semicolon token ';'
            input.parse::<Token![;]>()?;
            
            // Check if there are more interactions
            // If the input is empty, the continuation is End
            // Otherwise, parse the next interaction
            let continuation = if input.is_empty() {
                Interaction::End
            } else {
                input.parse::<Interaction>()?
            };
            
            Ok(Interaction::Message {
                from,
                to,
                message_type,
                continuation: Box::new(continuation),
            })
        }
    }
}

/// Parse implementation for Branch
///
/// Parses a branch in a choice interaction:
/// - option Label { ... } (named branch)
/// - { ... } (unnamed branch)
///
/// Each branch contains an interaction that defines what happens when this branch is chosen.
/// Branches can be labeled with an optional identifier to make the protocol more readable.
///
/// Examples (in DSL syntax, not valid Rust):
///
/// 1. Named branch:
///    ```text
///    option Success {
///        Server -> Client: LoginSuccess;
///        end;
///    }
///    ```
///
/// 2. Unnamed branch:
///    ```text
///    {
///        Server -> Client: LoginFailure;
///        end;
///    }
///    ```
///
/// 3. Branch with nested choice:
///    ```text
///    option Register {
///        Client -> Server: RegistrationInfo;
///        choice at Server {
///            option Valid {
///                Server -> Client: RegistrationSuccess;
///                end;
///            }
///            or {
///                Server -> Client: RegistrationFailure;
///                end;
///            }
///        }
///    }
///    ```
///
/// 4. Branch with recursion:
///    ```text
///    option Retry {
///        Client -> Server: Credentials;
///        cont LoginLoop;
///    }
///    ```
impl Parse for Branch {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Check if this branch has a label
        let label = if input.peek(kw::option) {
            input.parse::<kw::option>()?;
            Some(input.parse()?)
        } else {
            None
        };
        
        // Parse the branch body in braces
        let content;
        braced!(content in input);
        
        let interaction = content.parse::<Interaction>()?;
        
        Ok(Branch {
            label,
            interaction: Box::new(interaction),
        })
    }
}

/// Represents a branch in a choice interaction.
struct Branch {
    label: Option<Ident>,
    interaction: Box<Interaction>,
}

impl Branch {
    /// Generate Rust code for the branch
    fn generate_code(&self) -> proc_macro2::TokenStream {
        // Use an empty HashMap for recursion labels by default
        let recursion_labels = HashMap::new();
        self.generate_code_with_context(&recursion_labels)
    }
    
    /// Generate Rust code for the branch with recursion context
    fn generate_code_with_context(&self, recursion_labels: &HashMap<String, Ident>) -> proc_macro2::TokenStream {
        let interaction_code = self.interaction.generate_code_with_context(recursion_labels);
        let label = match &self.label {
            Some(label) => label.to_string(),
            None => "".to_string(), // Default to empty string for unnamed branches
        };
        
        quote! {
            (#label.into(), #interaction_code)
        }
    }
    
    /// Collect all message types used in the branch
    fn collect_message_types(&self, types: &mut Vec<Type>) {
        self.interaction.collect_message_types(types);
    }
    
    /// Validate the branch
    fn validate(&self, context: &mut ValidationContext) -> ValidationResult {
        self.validate_with_context(context)
    }
    
    /// Validate the branch with specialized context handling for complex nested structures
    fn validate_with_context(&self, context: &mut ValidationContext) -> ValidationResult {
        let mut errors = Vec::new();
        
        // Validate the interaction in this branch
        if let Err(interaction_errors) = self.interaction.validate_with_context(context) {
            errors.extend(interaction_errors);
        }
        
        // Check for unreachable code after continue statements
        if self.has_unreachable_code_after_continue() {
            errors.push(report_unreachable_code(self.span()));
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Check if this branch has unreachable code after a continue statement
    fn has_unreachable_code_after_continue(&self) -> bool {
        Self::check_unreachable_after_continue(&self.interaction)
    }
    
    /// Helper method to check for unreachable code after continue
    fn check_unreachable_after_continue(interaction: &Interaction) -> bool {
        match interaction {
            Interaction::Message { from: _, to: _, message_type: _, continuation } => {
                match &**continuation {
                    Interaction::Continue(_) => {
                        // If there's anything after the Continue, it's unreachable
                        // We're already in a match for Interaction::Continue(_), so this is always false
                        false
                    },
                    Interaction::End => false,
                    _ => Self::check_unreachable_after_continue(continuation),
                }
            },
            Interaction::Choice { participant: _, branches } => {
                // Check each branch for unreachable code
                branches.iter().any(|branch| Self::check_unreachable_after_continue(&branch.interaction))
            },
            Interaction::Recursion { label: _, body } => {
                // Check the recursion body
                Self::check_unreachable_after_continue(body)
            },
            Interaction::Continue(_) => false,
            Interaction::End => false,
        }
    }
    
    /// Get the span for error reporting
    fn span(&self) -> proc_macro2::Span {
        if let Some(label) = &self.label {
            label.span()
        } else {
            self.interaction.span()
        }
    }
}

/// Custom keyword parsers for the DSL syntax
///
/// These keywords are used to parse the DSL syntax:
/// - protocol: For protocol definition (protocol Name { ... })
/// - participant: For participant declaration (participant Role;)
/// - alias: For participant alias (participant Role alias R;) - used instead of 'as' which is a Rust keyword
/// - choice: For choice interaction (choice at Role { ... })
/// - at: Used in choice (choice at Role { ... })
/// - option: For choice branch (option Label { ... })
/// - or: For separating choice branches (... or { ... })
/// - rec: For recursion (rec Label { ... })
/// - cont: For continue (cont Label;) - used instead of 'continue' which is a Rust keyword
/// - end: For end (end;)
mod kw {
    syn::custom_keyword!(protocol);
    syn::custom_keyword!(participant);
    syn::custom_keyword!(alias);
    syn::custom_keyword!(choice);
    syn::custom_keyword!(at);
    syn::custom_keyword!(option);
    syn::custom_keyword!(or);
    syn::custom_keyword!(rec);
    syn::custom_keyword!(cont);
    syn::custom_keyword!(end);
}

/// Parse implementation for ProtocolDefinition
///
/// Parses: protocol Identifier { ParticipantList InteractionList }
///
/// Example (in DSL syntax, not valid Rust):
/// ```text
/// protocol PingPong {
///     participant Client;
///     participant Server;
///
///     Client -> Server: String;
///     Server -> Client: String;
///     end;
/// }
/// ```
///
/// Example with more complex interactions (in DSL syntax, not valid Rust):
/// ```text
/// protocol ChatProtocol {
///     participant Client;
///     participant Server;
///
///     rec ChatLoop {
///         choice at Client {
///             option SendMessage {
///                 Client -> Server: ChatMessage;
///                 Server -> Client: Acknowledgment;
///                 cont ChatLoop;
///             }
///             or {
///                 Client -> Server: Disconnect;
///                 end;
///             }
///         }
///     }
/// }
/// ```
impl Parse for ProtocolDefinition {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Parse 'protocol' keyword
        input.parse::<kw::protocol>()?;
        
        // Parse protocol name
        let name: Ident = input.parse()?;
        
        // Parse protocol body in braces
        let content;
        braced!(content in input);
        
        // Parse participants
        let mut participants = Vec::new();
        while content.peek(kw::participant) {
            participants.push(content.parse()?);
        }
        
        // Parse the protocol body
        let body = content.parse::<ProtocolBody>()?;
        
        Ok(ProtocolDefinition {
            name,
            participants,
            body,
        })
    }
}

/// Parse implementation for ParticipantDefinition
///
/// Parses: participant Identifier (alias Identifier)? ;
///
/// Example (in DSL syntax, not valid Rust):
/// ```text
/// participant Client;
/// participant Server alias S;
/// ```
impl Parse for ParticipantDefinition {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // Parse 'participant' keyword
        input.parse::<kw::participant>()?;
        
        // Parse participant name
        let name: Ident = input.parse()?;
        
        // Parse optional alias
        let alias = if input.peek(kw::alias) {
            input.parse::<kw::alias>()?;
            Some(input.parse()?)
        } else {
            None
        };
        
        // Parse semicolon
        input.parse::<Token![;]>()?;
        
        Ok(ParticipantDefinition { name, alias })
    }
}

/// Documentation for the DSL syntax
///
/// This module provides a comprehensive documentation of the DSL syntax
/// with examples for each construct.
mod documentation {
    /// Protocol definition (in DSL syntax, not valid Rust)
    ///
    /// ```text
    /// protocol ProtocolName {
    ///     participant Role1;
    ///     participant Role2;
    ///
    ///     // Interactions
    /// }
    /// ```
    #[allow(dead_code)]
    struct ProtocolDefinitionDoc;
    
    /// Message interaction (in DSL syntax, not valid Rust)
    ///
    /// ```text
    /// Role1 -> Role2: MessageType;
    /// ```
    #[allow(dead_code)]
    struct MessageInteractionDoc;
    
    /// Choice interaction (in DSL syntax, not valid Rust)
    ///
    /// ```text
    /// choice at Role1 {
    ///     option Label1 {
    ///         // Interactions for branch 1
    ///     }
    ///     or {
    ///         // Interactions for branch 2
    ///     }
    /// }
    /// ```
    #[allow(dead_code)]
    struct ChoiceInteractionDoc;
    
    /// Recursion interaction (in DSL syntax, not valid Rust)
    ///
    /// ```text
    /// rec Label {
    ///     // Interactions
    ///     cont Label;
    /// }
    /// ```
    #[allow(dead_code)]
    struct RecursionInteractionDoc;
    
    /// End interaction (in DSL syntax, not valid Rust)
    ///
    /// ```text
    /// end;
    /// ```
    #[allow(dead_code)]
    struct EndInteractionDoc;
}

/// Input structure for the `project` macro.
///
/// This struct parses the input to the `project` macro, which should be in the form:
/// `project!(Protocol, Role)` where `Protocol` is the global protocol type and `Role`
/// is the role type to project for.
struct ProjectionInput {
    protocol: syn::Type,
    role: syn::Type,
    message_type: Option<syn::Type>,
}

impl syn::parse::Parse for ProjectionInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let protocol = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let role = input.parse()?;
        
        let message_type = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };
        
        Ok(ProjectionInput {
            protocol,
            role,
            message_type,
        })
    }
}

/// A macro for projecting a global protocol to a role-specific local protocol.
///
/// This macro integrates with the existing projection mechanism in the `sessrums-types`
/// crate. It takes a global protocol type and a role type, and returns the projected
/// local protocol type for that role. Projection is the process of extracting the local
/// behavior of a specific participant from a global protocol.
///
/// # Parameters
///
/// - `GlobalProtocol`: The global protocol type defined using the `mpst!` macro
/// - `Role`: The role type to project for (must be one of the participants in the protocol)
/// - `MessageType`: (Optional) The message type parameter for the protocol
///
/// # Projection Rules
///
/// The projection follows these rules:
///
/// 1. **Message Passing**:
///    - If the participant is the sender, the projection is a `Send` operation
///    - If the participant is the receiver, the projection is a `Receive` operation
///    - If the participant is neither the sender nor the receiver, the projection is the projection of the continuation
///
/// 2. **Choice**:
///    - If the participant is the decider, the projection is a `Choose` operation
///    - If the participant is not the decider but participates in the branches, the projection is an `Offer` operation
///    - If the participant does not participate in any branch, the projection is `End`
///
/// 3. **Recursion**:
///    - The projection of a recursion is a recursion with the same label and the projection of the body
///    - The projection of a continue is a continue with the same label
///
/// # Example
///
/// ```
/// use sessrums_macro::{mpst, project};
/// use sessrums_types::roles::{Client, Server};
///
/// mpst! {
///     protocol PingPong {
///         participant Client;
///         participant Server;
///
///         Client -> Server: String;
///         Server -> Client: String;
///         end;
///     }
/// }
///
/// // Project the global protocol to local protocols for each role
/// type ClientProtocol = project!(PingPong, Client, String);
/// type ServerProtocol = project!(PingPong, Server, String);
///
/// // The resulting types are equivalent to:
/// // type ClientProtocol = Send<String, Receive<String, End>>;
/// // type ServerProtocol = Receive<String, Send<String, End>>;
/// ```
#[proc_macro]
pub fn project(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input
    let input = syn::parse_macro_input!(input as ProjectionInput);
    
    // Generate code that calls the projection function
    let protocol = &input.protocol;
    let role = &input.role;
    
    let output = if let Some(message_type) = &input.message_type {
        quote! {
            <#protocol as ::sessrums_types::projection::Project<#role, #message_type>>::Output
        }
    } else {
        quote! {
            <#protocol as ::sessrums_types::projection::Project<#role>>::Output
        }
    };
    
    output.into()
}