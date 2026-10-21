# Changelog

Notable changes to `nuck-protocol`. The **crate version** is packaging and Rust API;
the **protocol version** is the wire version negotiated via `/_nuck/v1/server-info`
(ADR 0026). Only breaking wire changes move the protocol version.

## v0.3.0 — 2026-10-22

- **Finalized the `/_nuck/v1` surface** (v1). Added the full DTO set for the in-scope
  client surface: `GET /me`, registration policy + approval queue (ADR 0037), guild
  reads and `PATCH`/`DELETE` (incl. the new `GET /guilds` list), channel
  `PATCH`/`DELETE`, member list + kick, invites/joining (ADR 0011/0025), friends,
  blocking and DMs (ADR 0030), and bans + the audit-log view (ADR 0016).
- Added `api::ENDPOINTS` — the endpoint inventory as data, checked by `tests/api_registry.rs`.
- `ApiError` gained an optional `retry_after_ms` (additive).
- **Protocol version unchanged (1)** — all additions are additive; the previous
  `server-info` capability list was aspirational and is corrected server-side.

## v0.2.0 — 2026-10-22

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
