# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Alien Gateway, please **do not open a public GitHub issue**.

Report it privately via one of the following channels:

- **GitHub Security Advisories**: [Report a vulnerability](https://github.com/Alien-Protocol/Alien-Gateway/security/advisories/new)
- **Email**: security@alien-protocol.io *(replace with actual contact before publishing)*

Please include:
- A description of the vulnerability and its potential impact
- Steps to reproduce or a proof-of-concept
- Affected component (circuit, contract, or off-chain tooling)

We aim to acknowledge reports within **48 hours** and provide a resolution timeline within **7 days**.

## Scope

In scope:
- ZK circuits (`zk/circuits/`)
- Soroban smart contracts (`gateway-contract/contracts/`)
- Trusted setup artifacts and verification keys

Out of scope:
- Third-party dependencies (circomlib, soroban-sdk) — report upstream
- Issues requiring physical access or social engineering

## Disclosure Policy

We follow coordinated disclosure. Please allow us reasonable time to patch before public disclosure. We will credit researchers in release notes unless anonymity is requested.

## Known Limitations (Pre-Audit)

See [`docs/threat-model.md`](docs/threat-model.md) for the current threat model and known findings being addressed before the audit handoff.
