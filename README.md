# nuck-protocol

Shared protocol types for [Nuck](https://github.com/nuck-hq/nuck-server) — the single
source of truth for the client/server contract (ADR 0027), carrying its own version
axis (ADR 0026).

Both **nuck-server** and the **Nuck client** depend on this crate, so the wire
contract is compile-time verified: drift is impossible for anything expressed here.

## Contents

| Module | What it holds |
|---|---|
| `events` | The custom `space.nuck.*` event registry (ADR 0014) |
| `permissions` | The permission bitfield and power-level mapping (ADR 0020) |
| `authorization` | The role/override resolution engine (ADR 0020/0041) |
| `api` | `/_nuck/v1` request/response DTOs (ADR 0013) |

Every type here is exchanged over the wire between Nuck servers, or between
nuck-server and the Nuck client.

## Versioning

There are two independent version axes:

- **Crate version** (`Cargo.toml`, released as `vX.Y.Z` git tags) — packaging and
  Rust API. Moves freely.
- **`PROTOCOL_VERSION`** (a `u32` constant, negotiated via `/_nuck/v1/server-info`) —
  the wire version. Bumped only on a **breaking** protocol change (ADR 0014/0026).

Additive changes keep the type name and rely on tolerant parsing; breaking changes
introduce a new type name (`…v2`) and bump `PROTOCOL_VERSION`. See `CHANGELOG.md` for
which releases moved the protocol version.

## Where the spec lives

The concrete protocol specification — the `space.nuck.*` event registry and the
`/_nuck/v1` endpoint inventory — currently lives with the design docs in
[nuck-server](https://github.com/nuck-hq/nuck-server/tree/main/docs/protocol).

## Using it

```toml
[dependencies]
nuck-protocol = { git = "https://github.com/nuck-hq/nuck-protocol.git", tag = "v0.1.0" }
```
