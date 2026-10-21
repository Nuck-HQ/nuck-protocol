# Changelog

Notable changes to `nuck-protocol`. The **crate version** is packaging and Rust API;
the **protocol version** is the wire version negotiated via `/_nuck/v1/server-info`
(ADR 0026). Only breaking wire changes move the protocol version.

## v0.1.0 — 2026-10-21

- Initial extraction of `nuck-protocol` into its own repository (previously a member
  of the nuck-server Cargo workspace).
- **Protocol version: 1.**
- Contents: `space.nuck.*` event registry (ADR 0014), `/_nuck/v1` DTOs (ADR 0013),
  permission bitfield and power-level mapping (ADR 0020), authorization engine
  (ADR 0020/0041).
