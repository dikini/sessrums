# Stage 7 Implementation Plan: DSL Macro System

## Context

Stage 7 focuses on implementing a procedural macro system for the Mermaid-like Domain-Specific Language (DSL) that allows users to define multiparty session type protocols in a concise, readable format. This stage is crucial as it bridges the gap between human-readable protocol specifications and the Rust type-level representations developed in previous stages.

Unlike the original plan that proposed using a dedicated parser (pest), this revised approach leverages Rust's procedural macro system to parse and validate the DSL syntax at compile time. This approach offers several advantages:

1. **Compile-time Verification**: Protocol errors are caught during compilation rather than at runtime
2. **Type Safety**: Direct integration with Rust's type system ensures type-safe protocol definitions
3. **IDE Support**: Better integration with Rust's tooling ecosystem (syntax highlighting, error reporting)
4. **Performance**: No runtime parsing overhead as protocols are expanded during compilation
5. **Simplified Deployment**: No additional runtime dependencies required

The procedural macro will transform textual protocol definitions directly into Rust code that constructs the `GlobalInteraction` enum structure. This enables users to define complex protocols using a familiar, diagram-like syntax while maintaining the strong type guarantees of the underlying system.

This stage represents a significant step toward making the MPST library more accessible and user-friendly, as it provides a more intuitive interface for protocol definition while maintaining the strong type guarantees of the underlying system.

---

## Review of Stage 7 Requirements

**Strengths:**
- Provides an intuitive, readable syntax for protocol definition
- Separates protocol specification from implementation details
- Enables non-Rust experts to define protocols
- Facilitates documentation and communication of protocol designs
- Leverages Rust's compile-time type checking for early error detection
- Eliminates runtime parsing overhead and dependencies
- Provides detailed compile-time error messages for protocol definition issues
- Integrates seamlessly with Rust's IDE tooling and error reporting

**Risks/Considerations:**
- Complexity of procedural macro implementation for nested structures (choice, recursion)
- Mapping between DSL syntax and Rust type system
- Providing clear and helpful error messages for syntax and semantic errors
- Ensuring the generated code can be efficiently compiled
- Handling of recursion variables and their scopes
- Supporting extensibility for future syntax additions
- Balancing expressiveness with simplicity in the DSL syntax
- Limitations of Rust's procedural macro system

---

## Stage 7: Actionable Task Breakdown

### 1. Project Preparation
- **Short:** Set up the project structure for the DSL macro system.
    - **Implementation Prompt:**  
      Update the `sessrums-macro` crate to include the necessary dependencies (`proc-macro2`, `quote`, `syn`) in `Cargo.toml`. Create the basic structure for the procedural macro in `src/lib.rs` with the `#[proc_macro]` attribute and necessary imports.
    - **Documentation Prompt:**  
      Document the purpose of the macro crate and its relationship to the existing codebase. Explain the choice of procedural macros for DSL implementation and how it integrates with the rest of the system.

---

### 2. Define DSL Syntax and Grammar
- **Short:** Define the formal syntax and grammar for the MPST DSL.
    - **Implementation Prompt:**  
      Create a document that formally defines the DSL syntax, including:
      - Protocol declaration and participants
      - Message passing (A -> B: Msg)
      - Choice blocks with multiple branches
      - Recursion (rec X { ... } and continue X)
      - Parallel composition (optional, if supported)
      - Comments and whitespace handling
      
      Ensure the grammar is well-structured, unambiguous, and handles nested constructs correctly.
    - **Documentation Prompt:**  
      Create comprehensive documentation of the DSL syntax with examples for each construct. Include a formal grammar specification and explain any syntactic restrictions or considerations.

---

### 3. Define Token Parser
- **Short:** Implement the token parser for the DSL syntax.
    - **Implementation Prompt:**  
      Using the `syn` crate, implement custom parsers for the DSL syntax:
      ```rust
      struct ProtocolDefinition {
          name: Ident,
          participants: Vec<ParticipantDefinition>,
          body: ProtocolBody,
      }
      
      struct ParticipantDefinition {
          name: Ident,
          alias: Option<Ident>,
      }
      
      enum ProtocolBody {
          Interaction(Box<Interaction>),
          // Other variants as needed
      }
      
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
      
      struct Branch {
          label: Option<Ident>,
          interaction: Box<Interaction>,
      }
      ```
      
      Implement the `Parse` trait for these structures to parse the DSL syntax from the token stream.
    - **Documentation Prompt:**  
      Document each parser structure and its role in parsing the DSL syntax. Include examples of how different DSL constructs are parsed into these structures.

---

### 4. Implement Syntax Validation
- **Short:** Implement validation for the parsed DSL syntax.
    - **Implementation Prompt:**  
      Implement validation functions that check the parsed syntax for correctness:
      ```rust
      impl ProtocolDefinition {
          fn validate(&self) -> Result<(), Vec<syn::Error>> {
              let mut errors = Vec::new();
              
              // Check that all participants are defined
              // Check that all recursion labels are defined before use
              // Check that all message types are valid
              // Additional semantic checks
              
              if errors.is_empty() {
                  Ok(())
              } else {
                  Err(errors)
              }
          }
      }
      ```
      
      Implement specific validation functions for different aspects of the protocol (participants, recursion, etc.).
    - **Documentation Prompt:**  
      Document the validation functions, explaining what semantic checks are performed and why they are important. Include examples of valid and invalid protocols and the corresponding validation results.

---

### 5. Implement Code Generation
- **Short:** Implement code generation for the validated DSL syntax.
    - **Implementation Prompt:**  
      Using the `quote` crate, implement code generation for the validated DSL syntax:
      ```rust
      impl ProtocolDefinition {
          fn generate_code(&self) -> proc_macro2::TokenStream {
              let name = &self.name;
              let body = self.body.generate_code();
              
              quote! {
                  type #name = #body;
              }
          }
      }
      
      impl Interaction {
          fn generate_code(&self) -> proc_macro2::TokenStream {
              match self {
                  Interaction::Message { from, to, message_type, continuation } => {
                      let cont = continuation.generate_code();
                      quote! {
                          ::sessrums_types::session_types::global::GlobalInteraction::message(
                              #from,
                              #to,
                              #cont
                          )
                      }
                  },
                  // Other variants
              }
          }
      }
      ```
      
      Ensure the generated code correctly constructs the `GlobalInteraction` enum structure.
    - **Documentation Prompt:**  
      Document the code generation functions, explaining how each DSL construct is mapped to the corresponding Rust code. Include examples of the generated code for different protocol constructs.

---

### 6. Implement Error Reporting
- **Short:** Implement detailed error reporting for the macro with user-friendly error messages.
    - **Implementation Prompt:**
      Enhance the validation and parsing functions to provide detailed error messages with span information:
      ```rust
      fn report_error(span: proc_macro2::Span, message: &str) -> syn::Error {
          syn::Error::new(span, message)
      }
      
      // Helper functions for common error patterns
      fn report_undefined_participant(span: proc_macro2::Span, name: &str) -> syn::Error {
          report_error(span, &format!("Undefined participant '{}'. All participants must be declared at the beginning of the protocol.", name))
      }
      
      fn report_undefined_recursion_label(span: proc_macro2::Span, label: &str) -> syn::Error {
          report_error(span, &format!("Undefined recursion label '{}'. Labels must be defined with 'rec {}' before they can be referenced with 'continue {}'.", label, label, label))
      }
      
      fn report_duplicate_recursion_label(span: proc_macro2::Span, label: &str) -> syn::Error {
          report_error(span, &format!("Duplicate recursion label '{}'. Each recursion label must be unique within its scope.", label))
      }
      
      fn report_invalid_message_type(span: proc_macro2::Span, type_str: &str) -> syn::Error {
          report_error(span, &format!("Invalid message type '{}'. Message types must be valid Rust types.", type_str))
      }
      
      fn report_invalid_choice_syntax(span: proc_macro2::Span) -> syn::Error {
          report_error(span, "Invalid choice syntax. Expected 'choice at Role { option Label { ... } ... }'.")
      }
      ```
      
      Use the `syn::Error` type to report errors with specific locations in the source code. Implement a comprehensive error reporting system that provides clear, actionable feedback to users.
      
      Create a categorized error system that groups errors by type:
      ```rust
      enum ErrorCategory {
          Syntax,        // Basic syntax errors
          Participant,   // Errors related to participant definitions
          Recursion,     // Errors related to recursion labels
          Type,          // Errors related to message types
          Semantic,      // Higher-level semantic errors
      }
      
      struct ErrorContext {
          category: ErrorCategory,
          error: syn::Error,
          suggestion: Option<String>, // Optional suggestion for fixing the error
      }
      ```
      
      Ensure that error messages include:
      1. What went wrong (clear description of the error)
      2. Where it went wrong (precise location in the source code)
      3. Why it matters (explanation of the rule that was violated)
      4. How to fix it (suggestion for correcting the error)
      
    - **Documentation Prompt:**
      Document the error reporting approach, explaining how errors are reported to the user. Include examples of common errors and their error messages.
      
      Provide a comprehensive list of potential error messages with examples:
      
      **Syntax Errors:**
      ```
      error: Invalid choice syntax. Expected 'choice at Role { option Label { ... } ... }'.
        --> protocol.rs:15:5
         |
      15 |     choice Client {
         |     ^^^^^^^^^^^^^ Expected 'at' keyword after 'choice'
         |
         = help: Use 'choice at Client { ... }' instead
      ```
      
      **Participant Errors:**
      ```
      error: Undefined participant 'Database'. All participants must be declared at the beginning of the protocol.
        --> protocol.rs:12:5
         |
      12 |     Client -> Database: Query;
         |              ^^^^^^^^ Participant not declared
         |
         = help: Add 'participant Database;' to the beginning of the protocol
      ```
      
      **Recursion Errors:**
      ```
      error: Undefined recursion label 'Loop'. Labels must be defined with 'rec Loop' before they can be referenced with 'continue Loop'.
        --> protocol.rs:18:9
         |
      18 |         continue Loop;
         |                 ^^^^ Label not defined
         |
         = help: Define the recursion label with 'rec Loop { ... }' before using 'continue Loop'
      ```
      
      ```
      error: Duplicate recursion label 'Loop'. Each recursion label must be unique within its scope.
        --> protocol.rs:22:9
         |
      22 |     rec Loop {
         |         ^^^^ Duplicate label
         |
         = help: Choose a different label name for this recursion block
      ```
      
      **Type Errors:**
      ```
      error: Invalid message type 'Map<String, Value>'. Message types must be valid Rust types.
        --> protocol.rs:10:24
         |
      10 |     Client -> Server: Map<String, Value>;
         |                        ^^^^^^^^^^^^^^^^^ Unknown type
         |
         = help: Use a fully qualified path or import the type with 'use'
      ```
      
      **Semantic Errors:**
      ```
      error: Role 'Client' cannot make a choice in a branch where 'Server' is the deciding role.
        --> protocol.rs:25:13
         |
      25 |             Client -> Server: Continue;
         |             ^^^^^^ Invalid role in choice branch
         |
         = help: In a choice block, the first message in each branch must be sent by the deciding role
      ```

---

### 7. Handling Complex Nested Structures
- **Short:** Implement specialized handling for complex nested structures like recursion within choice branches.
    - **Implementation Prompt:**
      Develop a robust approach for handling complex nested structures, particularly recursion within choice branches:
      
      ```rust
      // Track recursion context during parsing and validation
      struct RecursionContext {
          // Stack of active recursion labels to detect nested recursion
          active_labels: Vec<Ident>,
          // Map of label to its scope (for validation)
          label_scopes: HashMap<String, Span>,
          // Track recursion depth for potential optimizations
          depth: usize,
      }
      
      impl RecursionContext {
          fn enter_recursion(&mut self, label: &Ident) -> Result<(), syn::Error> {
              // Check if label is already active (detect invalid nested recursion with same label)
              if self.active_labels.iter().any(|l| l == label) {
                  return Err(syn::Error::new(
                      label.span(),
                      format!("Recursion label '{}' is already active in an outer scope", label)
                  ));
              }
              
              // Add label to active set
              self.active_labels.push(label.clone());
              self.label_scopes.insert(label.to_string(), label.span());
              self.depth += 1;
              
              Ok(())
          }
          
          fn exit_recursion(&mut self, label: &Ident) -> Result<(), syn::Error> {
              // Ensure we're exiting the most recently entered recursion
              if let Some(active_label) = self.active_labels.last() {
                  if active_label != label {
                      return Err(syn::Error::new(
                          label.span(),
                          format!("Mismatched recursion labels: expected '{}', found '{}'",
                                  active_label, label)
                      ));
                  }
                  
                  self.active_labels.pop();
                  self.depth -= 1;
                  Ok(())
              } else {
                  Err(syn::Error::new(
                      label.span(),
                      "Exiting recursion without matching 'rec' block".to_string()
                  ))
              }
          }
          
          fn check_continue(&self, label: &Ident) -> Result<(), syn::Error> {
              // Check if the label is in the active set
              if !self.active_labels.iter().any(|l| l == label) {
                  return Err(syn::Error::new(
                      label.span(),
                      format!("Continue to undefined or inactive recursion label '{}'", label)
                  ));
              }
              
              Ok(())
          }
      }
      ```
      
      Implement specialized handling for choice branches containing recursion:
      
      ```rust
      impl Interaction {
          fn validate_with_context(&self, context: &mut ValidationContext) -> Result<(), Vec<syn::Error>> {
              match self {
                  Interaction::Choice { participant, branches } => {
                      let mut errors = Vec::new();
                      
                      // Create a branch context that clones the current recursion context
                      // This allows each branch to have its own recursion state
                      for branch in branches {
                          // Clone the recursion context for this branch
                          let mut branch_context = context.clone_recursion_context();
                          
                          // Validate the branch with its own context
                          if let Err(branch_errors) = branch.interaction.validate_with_context(&mut branch_context) {
                              errors.extend(branch_errors);
                          }
                          
                          // Check for unreachable code after continue statements
                          if branch.has_unreachable_code_after_continue() {
                              errors.push(syn::Error::new(
                                  branch.span(),
                                  "Unreachable code after 'continue' statement"
                              ));
                          }
                      }
                      
                      if !errors.is_empty() {
                          return Err(errors);
                      }
                      
                      Ok(())
                  },
                  
                  Interaction::Recursion { label, body } => {
                      // Enter recursion scope
                      if let Err(e) = context.recursion.enter_recursion(label) {
                          return Err(vec![e]);
                      }
                      
                      // Validate the body with updated context
                      let result = body.validate_with_context(context);
                      
                      // Exit recursion scope (even if body validation failed)
                      if let Err(e) = context.recursion.exit_recursion(label) {
                          let mut errors = result.err().unwrap_or_default();
                          errors.push(e);
                          return Err(errors);
                      }
                      
                      result
                  },
                  
                  Interaction::Continue(label) => {
                      if let Err(e) = context.recursion.check_continue(label) {
                          return Err(vec![e]);
                      }
                      
                      Ok(())
                  },
                  
                  // Other variants...
                  _ => Ok(()),
              }
          }
      }
      ```
      
      Implement code generation that correctly handles these nested structures:
      
      ```rust
      impl Interaction {
          fn generate_code(&self, recursion_labels: &HashMap<String, Ident>) -> proc_macro2::TokenStream {
              match self {
                  Interaction::Choice { participant, branches } => {
                      // Generate code for each branch, passing down the recursion context
                      let branch_codes = branches.iter().map(|branch| {
                          branch.generate_code(recursion_labels)
                      }).collect::<Vec<_>>();
                      
                      quote! {
                          ::sessrums_types::session_types::global::GlobalInteraction::choice(
                              #participant,
                              vec![
                                  #(#branch_codes),*
                              ]
                          )
                      }
                  },
                  
                  Interaction::Recursion { label, body } => {
                      // Create a unique type identifier for this recursion label
                      let label_str = label.to_string();
                      let label_type = format_ident!("{}Label", label_str);
                      
                      // Add this label to the context for use by continue statements
                      let mut updated_labels = recursion_labels.clone();
                      updated_labels.insert(label_str, label_type.clone());
                      
                      // Generate the body with the updated recursion context
                      let body_code = body.generate_code(&updated_labels);
                      
                      quote! {
                          ::sessrums_types::session_types::global::GlobalInteraction::rec(
                              #label,
                              #body_code
                          )
                      }
                  },
                  
                  Interaction::Continue(label) => {
                      // Look up the label in the recursion context
                      let label_str = label.to_string();
                      let label_type = recursion_labels.get(&label_str).expect("Label should be validated");
                      
                      quote! {
                          ::sessrums_types::session_types::global::GlobalInteraction::var(#label)
                      }
                  },
                  
                  // Other variants...
                  _ => quote! { /* ... */ },
              }
          }
      }
      ```
      
      Implement specialized validation for complex cases:
      
      ```rust
      // Check for valid recursion patterns
      fn validate_recursion_patterns(interaction: &Interaction) -> Result<(), Vec<syn::Error>> {
          let mut errors = Vec::new();
          
          // Check for tail recursion optimization opportunities
          check_tail_recursion(interaction, &mut errors);
          
          // Check for mutual recursion
          check_mutual_recursion(interaction, &mut errors);
          
          // Check for recursion within choice branches
          check_recursion_in_choice(interaction, &mut errors);
          
          if errors.is_empty() {
              Ok(())
          } else {
              Err(errors)
          }
      }
      
      // Specialized check for recursion within choice branches
      fn check_recursion_in_choice(interaction: &Interaction, errors: &mut Vec<syn::Error>) {
          if let Interaction::Choice { branches, .. } = interaction {
              for branch in branches {
                  // Check if this branch contains recursion
                  let contains_recursion = branch.contains_recursion();
                  let contains_continue = branch.contains_continue();
                  
                  // Validate that continue statements are properly nested within recursion
                  if contains_continue && !contains_recursion {
                      // This is valid if the continue refers to an outer recursion
                      // We'll check this during context-based validation
                  }
                  
                  // Recursively check nested interactions
                  check_recursion_in_choice(&branch.interaction, errors);
              }
          }
          
          // Recursively check other interaction types
          match interaction {
              Interaction::Message { cont, .. } => check_recursion_in_choice(cont, errors),
              Interaction::Recursion { body, .. } => check_recursion_in_choice(body, errors),
              _ => {}
          }
      }
      ```
      
    - **Documentation Prompt:**
      Document the approach for handling complex nested structures, explaining the challenges and solutions. Include diagrams or examples illustrating how recursion within choice branches is handled. Explain the validation rules and code generation strategies for these complex cases.
      
      Provide examples of valid and invalid nested structures:
      
      **Valid Example: Recursion with Choice Branches**
      ```
      rec Loop {
          choice at Client {
              option Continue {
                  Client -> Server: ContinueMsg;
                  Server -> Client: Ack;
                  continue Loop;
              }
              option Exit {
                  Client -> Server: ExitMsg;
              }
          }
      }
      ```
      
      **Valid Example: Nested Recursion**
      ```
      rec OuterLoop {
          Client -> Server: Start;
          rec InnerLoop {
              choice at Client {
                  option Continue {
                      Client -> Server: Continue;
                      continue InnerLoop;
                  }
                  option Break {
                      Client -> Server: Break;
                  }
              }
          }
          Server -> Client: OuterContinue;
          continue OuterLoop;
      }
      ```
      
      **Invalid Example: Continue to Undefined Label**
      ```
      rec Loop1 {
          Client -> Server: Msg;
          continue Loop2; // Error: Loop2 is not defined
      }
      ```
      
      **Invalid Example: Unreachable Code After Continue**
      ```
      rec Loop {
          Client -> Server: Msg;
          continue Loop;
          Server -> Client: Response; // Error: Unreachable code
      }
      ```

### 8. Implement the Main Macro
- **Short:** Implement the main procedural macro that ties everything together.
    - **Implementation Prompt:**  
      Implement the `#[proc_macro]` function that serves as the entry point for the macro:
      ```rust
      #[proc_macro]
      pub fn mpst(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
          let input = syn::parse_macro_input!(input as ProtocolDefinition);
          
          // Validate the input
          if let Err(errors) = input.validate() {
              // Return the errors
              return errors.into_iter()
                  .map(|e| e.to_compile_error())
                  .collect::<proc_macro2::TokenStream>()
                  .into();
          }
          
          // Generate code
          let output = input.generate_code();
          
          output.into()
      }
      ```
      
      Ensure the macro handles all error cases gracefully and provides helpful error messages.
    - **Documentation Prompt:**  
      Document the main macro function, explaining its role in the overall system. Include examples of how to use the macro and how errors are reported.

---

### 8. Testing: Basic Syntax
- **Short:** Implement tests for basic DSL syntax.
    - **Implementation Prompt:**  
      Create `tests/basic_syntax.rs` with tests for basic DSL syntax:
      ```rust
      #[test]
      fn test_simple_protocol() {
          let result = trybuild::TestCases::new().pass("tests/pass/simple_protocol.rs");
      }
      ```
      
      Create test files in `tests/pass/` and `tests/fail/` directories to test successful and failing cases.
    - **Documentation Prompt:**  
      Document each test, explaining what aspect of the DSL syntax it is testing and what the expected outcome is. Include comments explaining the test protocol and its expected representation.

---

### 9. Testing: Complex Protocols
- **Short:** Implement tests for complex protocol definitions.
    - **Implementation Prompt:**  
      Create `tests/complex_protocols.rs` with tests for complex protocol definitions:
      ```rust
      #[test]
      fn test_recursive_protocol() {
          let result = trybuild::TestCases::new().pass("tests/pass/recursive_protocol.rs");
      }
      
      #[test]
      fn test_choice_protocol() {
          let result = trybuild::TestCases::new().pass("tests/pass/choice_protocol.rs");
      }
      ```
      
      Create test files for complex protocols with recursion, choice, and nested structures.
    - **Documentation Prompt:**  
      Document each test, explaining the complex protocol being tested and how the macro handles nested structures. Include diagrams or comments illustrating the expected structure.

---

### 10. Testing: Error Reporting
- **Short:** Implement comprehensive tests for error reporting across all error categories.
    - **Implementation Prompt:**
      Create `tests/error_reporting.rs` with tests for error reporting across all error categories:
      ```rust
      #[test]
      fn test_syntax_errors() {
          let test_cases = trybuild::TestCases::new();
          test_cases.compile_fail("tests/fail/syntax/invalid_choice_syntax.rs");
          test_cases.compile_fail("tests/fail/syntax/missing_semicolon.rs");
          test_cases.compile_fail("tests/fail/syntax/invalid_message_arrow.rs");
      }
      
      #[test]
      fn test_participant_errors() {
          let test_cases = trybuild::TestCases::new();
          test_cases.compile_fail("tests/fail/participant/undefined_participant.rs");
          test_cases.compile_fail("tests/fail/participant/duplicate_participant.rs");
          test_cases.compile_fail("tests/fail/participant/invalid_participant_name.rs");
      }
      
      #[test]
      fn test_recursion_errors() {
          let test_cases = trybuild::TestCases::new();
          test_cases.compile_fail("tests/fail/recursion/undefined_recursion_label.rs");
          test_cases.compile_fail("tests/fail/recursion/duplicate_recursion_label.rs");
          test_cases.compile_fail("tests/fail/recursion/continue_outside_recursion.rs");
      }
      
      #[test]
      fn test_type_errors() {
          let test_cases = trybuild::TestCases::new();
          test_cases.compile_fail("tests/fail/type/invalid_message_type.rs");
          test_cases.compile_fail("tests/fail/type/unsupported_generic_type.rs");
      }
      
      #[test]
      fn test_semantic_errors() {
          let test_cases = trybuild::TestCases::new();
          test_cases.compile_fail("tests/fail/semantic/invalid_choice_role.rs");
          test_cases.compile_fail("tests/fail/semantic/empty_choice.rs");
          test_cases.compile_fail("tests/fail/semantic/unreachable_code.rs");
      }
      ```
      
      Create test files for each error case with expected error messages in comments:
      
      Example for `tests/fail/participant/undefined_participant.rs`:
      ```rust
      use sessrums_macro::mpst;
      
      // This test should fail with:
      // error: Undefined participant 'Database'. All participants must be declared at the beginning of the protocol.
      
      mpst! {
          protocol QueryProtocol {
              participant Client;
              participant Server;
              
              // Database is not declared as a participant
              Client -> Database: Query;
              Database -> Client: Response;
          }
      }
      
      fn main() {}
      ```
      
      Example for `tests/fail/recursion/undefined_recursion_label.rs`:
      ```rust
      use sessrums_macro::mpst;
      
      // This test should fail with:
      // error: Undefined recursion label 'Loop'. Labels must be defined with 'rec Loop' before they can be referenced with 'continue Loop'.
      
      mpst! {
          protocol ChatProtocol {
              participant Client;
              participant Server;
              
              Client -> Server: Message;
              Server -> Client: Ack;
              
              // Loop is not defined before it's referenced
              continue Loop;
          }
      }
      
      fn main() {}
      ```
      
      Create similar test files for each error case, ensuring they cover all the error categories and common mistakes users might make.
      
    - **Documentation Prompt:**
      Document each test category, explaining what error cases it covers and what the expected error messages are. Include a table mapping error categories to specific test files and error messages:
      
      | Error Category | Test File | Expected Error Message |
      |----------------|-----------|------------------------|
      | Syntax | invalid_choice_syntax.rs | Invalid choice syntax. Expected 'choice at Role { option Label { ... } ... }'. |
      | Participant | undefined_participant.rs | Undefined participant 'Database'. All participants must be declared at the beginning of the protocol. |
      | Recursion | undefined_recursion_label.rs | Undefined recursion label 'Loop'. Labels must be defined with 'rec Loop' before they can be referenced with 'continue Loop'. |
      | Type | invalid_message_type.rs | Invalid message type 'Map<String, Value>'. Message types must be valid Rust types. |
      | Semantic | invalid_choice_role.rs | Role 'Client' cannot make a choice in a branch where 'Server' is the deciding role. |
      
      Explain how these tests verify that the error reporting system provides clear, actionable feedback to users. Emphasize the importance of good error messages for user experience and productivity.

---

### 11. Integration with Projection
- **Short:** Ensure the macro integrates with the existing projection mechanism.
    - **Implementation Prompt:**  
      Implement helper functions or methods to integrate the macro with the existing projection mechanism:
      ```rust
      #[proc_macro]
      pub fn project(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
          // Parse the input
          let input = syn::parse_macro_input!(input as ProjectionInput);
          
          // Generate code that calls the projection function
          let protocol = &input.protocol;
          let role = &input.role;
          
          let output = quote! {
              <#protocol as ::sessrums_types::projection::Project<#role>>::project()
          };
          
          output.into()
      }
      ```
      
      Ensure the generated code can be used with the existing projection mechanism.
    - **Documentation Prompt:**  
      Document the integration with projection, explaining how users can project global protocols defined with the macro to local protocols for specific roles. Include examples of the complete workflow from DSL to execution.

---

### 12. Documentation and Examples
- **Short:** Create comprehensive documentation and examples for the DSL macro.
    - **Implementation Prompt:**  
      Add doc comments to all public items in the macro crate. Create example files demonstrating:
      - A simple protocol (e.g., ping-pong)
      - A protocol with choice (e.g., file transfer with success/failure paths)
      - A recursive protocol (e.g., streaming data with termination)
      
      Ensure examples are well-commented and illustrate best practices.
    - **Documentation Prompt:**  
      Create a dedicated documentation file (`docs/dsl-macro.md`) explaining:
      - The DSL syntax and grammar
      - How to define protocols using the macro
      - Examples of common protocol patterns
      - Error handling and troubleshooting
      - Integration with the rest of the MPST system

---

## Summary Table

| Task Group | Actions |
|------------|---------|
| Project Preparation | Set up macro crate, add dependencies |
| DSL Syntax | Define formal syntax and grammar |
| Token Parser | Implement parsers for DSL syntax |
| Syntax Validation | Implement validation for parsed syntax |
| Code Generation | Implement code generation for validated syntax |
| Error Reporting | Implement detailed error reporting |
| Complex Structures | Handle recursion within choice branches and other nested structures |
| Main Macro | Implement the main procedural macro |
| Basic Testing | Test basic DSL syntax |
| Complex Testing | Test complex protocol definitions |
| Error Testing | Test error reporting |
| Projection Integration | Ensure integration with projection |
| Documentation | Create comprehensive documentation and examples |

---

## Pre-conditions and Post-conditions

### Pre-conditions
- Understanding of Rust's procedural macro system
- Knowledge of the existing `GlobalInteraction` structure
- Familiarity with the `syn` and `quote` crates for parsing and code generation
- Rust environment set up with necessary dependencies

### Post-conditions
- DSL macro implemented and tested
- Comprehensive test suite passes
- Documentation and examples are complete
- Integration with existing codebase is verified
- Users can define protocols using the intuitive DSL syntax
- Compile-time validation ensures protocol correctness
- Error messages are clear and helpful

Each task in this plan is self-contained with clear pre-conditions and post-conditions, making it easy to implement and verify incrementally. The tests provide a way to verify the completion of each task, ensuring the overall quality and correctness of the implementation.