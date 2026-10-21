# Changelog

Notable changes to `nuck-protocol`. The **crate version** is packaging and Rust API;
the **protocol version** is the wire version negotiated via `/_nuck/v1/server-info`
(ADR 0026). Only breaking wire changes move the protocol version.

## Unreleased

- Finalized the permission table (ADR 0042): added the four declared-but-undocumented
  permissions (`PRIORITY_SPEAKER`, `STREAM`, `SEND_TTS_MESSAGES`, `USE_EXTERNAL_EMOJIS`)
  and registry tests that keep the table and the declared bits in sync. Documented the
  coarse power-level mapping; demoted `required_level` to ordering metadata.
- Added the missing event content types: `space.nuck.emoji.v1`,
  `space.nuck.sticker.v1`, `space.nuck.interaction.v1`.
- Added `tests/registry.rs`: constant↔content-type agreement, well-formed unique
  versioned names, and tolerant parsing (ADR 0014).
- **Protocol version unchanged (1)** — every change here is additive.

## v0.1.0 — 2026-10-21

- Initial extraction of `nuck-protocol` into its own repository (previously a member
  of the nuck-server Cargo workspace).
- **Protocol version: 1.**
- Contents: `space.nuck.*` event registry (ADR 0014), `/_nuck/v1` DTOs (ADR 0013),
  permission bitfield and power-level mapping (ADR 0020), authorization engine
  (ADR 0020/0041).
