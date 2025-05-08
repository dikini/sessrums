Generate a comprehensive map of the Rust codebase following this format:

# sessrums-types Codebase Map

## Symbol Index

### src/[filename]
Brief one-line description of the file's purpose.

- symbol_kind name : filename (start_line-end_line)
- impl Type for Target : filename (start_line-end_line)
- #[cfg(test)] mod tests : filename (start_line-end_line)
  - #[test] fn test_name : filename (start_line-end_line)

Requirements:
- List every struct, enum, trait, impl block, and function
- Include test modules and functions with their attributes
- Sort by file and then by line number
- Use precise line numbers
- Include full generic constraints in impl blocks
- Format in markdown

Example entry:
### src/messages.rs
Defines serializable message types using serde.

- struct PingMsg : messages.rs (8-11)
- #[cfg(test)] mod tests : messages.rs (19-43)
  - fn test_ping_serialization : messages.rs (24-30)