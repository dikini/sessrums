
## Structure for session types library

```bash
[project-name]/
├── src/                    
│   ├── proto/             # op type module definitions
│   │   ├── proto.rs
│   │   ├── send.rs
│   │   ├── recv.rs
│   │   ├── offer.rs
│   │   ├── choose.rs
│   │   ├── end.rs
│   │   └── test
│   ├── chan.rs
|   ├── lib.rs# [core library, includes and exports everything public]
│   │
│   └── Readme.md       
├── test
├── Cargo.toml         
└── Readme.md          
```

## Key Structure Decisions

This structure is a typical rust library structure. The protocols (proto) are split into individual modules iliving in the proto directory in order to keep individual file size low, as source files include unit tests. chan.rs contains the channel structure. Other decisions will be made at later dev stages.


## Build & Deployment Structure

The customary cargo commands are used to build test and package.

Readme files contain library and dev information in markdown format.