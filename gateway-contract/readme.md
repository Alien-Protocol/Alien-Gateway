```
gateway-contract/
├── contracts/
│   └── core/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── core.rs
│       │   ├── address_manager.rs
│       │   ├── chain_registry.rs
│       │   ├── escrow.rs
│       │   ├── auth.rs
│       │   ├── events.rs
│       │   └── types.rs
│       ├── Cargo.toml
│       └── Cargo.lock
├── tests/
│   ├── integration/
│   │   ├── test_core.rs
│   │   ├── test_address_manager.rs
│   │   ├── test_chain_registry.rs
│   │   ├── test_escrow.rs
│   │   └── test_auth.rs
│   └── unit/
│       └── test_types.rs
└── README.md
```