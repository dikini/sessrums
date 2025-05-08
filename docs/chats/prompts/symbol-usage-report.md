# Symbol Usage Report Generator

Generate a comprehensive usage report for a Rust symbol.

## Input Parameters
- symbol_kind: The type of symbol (struct, enum, trait, etc.)
- symbol_name: The name of the symbol to analyze

## Output Format

# Usage Report: [symbol_kind] [symbol_name]

## Definition
[path]/[filename] : (start_line-end_line)
```rust
// filepath: [full_path]
[code definition]
```

## Usage Locations

[path]/[filename] : (start_line-end_line)
- Brief description of usage context
```rust
// filepath: [full_path]
[relevant code snippet]
```

## Requirements
- Show complete symbol definition with exact line numbers
- List every usage location with exact line numbers
- Include test usage locations
- Sort by file and then by line number
- Provide brief context for each usage
- Include relevant code snippets
- Format in markdown with rust code blocks

## Example

Input: `struct PingMsg`

# Usage Report: struct PingMsg

## Definition
sessrums-types/src/messages.rs : (8-11)
```rust
// filepath: sessrums-types/src/messages.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PingMsg {
    pub seq: Option<i32>,
}
```

## Usage Locations

sessrums-types/src/messages.rs : (24-30)
- Test serialization implementation
```rust
// filepath: sessrums-types/src/messages.rs
#[test]
fn test_ping_serialization() {
    let msg = PingMsg { seq: Some(1) };
    // ...existing code...
}
```

sessrums-types/src/session_types/binary.rs : (71-98)
- Protocol state transition test
```rust
// filepath: sessrums-types/src/session_types/binary.rs
#[test]
fn test_ping_pong_protocol() {
    let msg = PingMsg { seq: Some(42) };
    // ...existing code...
}
```