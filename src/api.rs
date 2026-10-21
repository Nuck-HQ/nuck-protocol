//! `/_nuck/v1` request/response DTOs (ADR 0013).
//!
//! Draft surface — see `docs/protocol/api.md`. These are the shared shapes between
//! nuck-server and the Nuck client; the full endpoint inventory and error model are
//! still being specified.

use serde::{Deserialize, Serialize};

use crate::events::{ChannelKind, DirectoryVisibility, JoinMode, JoinRequestStatus};
use crate::permissions::Permissions;

/// `GET /_nuck/v1/server-info` — capability and version advertisement (ADR 0010/0026).
///
/// Public (unauthenticated). Peers and clients use it to negotiate feature gates.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerInfo {
    /// The server's Matrix `server_name`.
    pub server_name: String,
    /// Product version of nuck-server (moves freely).
    pub product_version: String,
    /// The Nuck protocol version this server speaks.
    pub protocol_version: u32,
    /// The oldest protocol version this server still speaks.
    pub protocol_min_version: u32,
    /// Advertised capabilities, e.g. `guilds`, `voice`, `bots`, `search`.
    pub capabilities: Vec<String>,
}

/// The uniform API error body (ADR 0013/0026: never silently misbehave).
#[derive(Clone, Debug, Deserialize, Serialize, thiserror::Error)]
#[error("{errcode}: {error}")]
pub struct ApiError {
    /// A stable, machine-readable error code, e.g. `M_FORBIDDEN`.
    pub errcode: String,
    /// A human-readable message.
    pub error: String,
    /// Set on `M_LIMIT_EXCEEDED` (ADR 0036). Optional and additive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
}

impl ApiError {
    #[must_use]
    pub fn new(errcode: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            errcode: errcode.into(),
            error: error.into(),
            retry_after_ms: None,
        }
    }
}

// --- Privileged guild actions (ADR 0006/0041) ------------------------------
//
// These are the requests nuck-server authorizes and then executes through the
// privileged **action bot** (ADR 0041). The client never writes privileged guild
// state directly; the resulting substrate event (and the audit entry) carry the
// acting user for attribution.

/// `POST /_nuck/v1/guilds/{guildId}/roles` — create a role.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateRoleRequest {
    pub name: String,
    /// RGB color, `0xRRGGBB`.
    #[serde(default)]
    pub color: Option<u32>,
    /// Higher position wins on conflict. Must be below the caller's highest role.
    pub position: i64,
    pub permissions: Permissions,
    #[serde(default)]
    pub hoist: bool,
    #[serde(default)]
    pub mentionable: bool,
}

/// The created role.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateRoleResponse {
    /// The generated role id (the state key of `space.nuck.role.v1`).
    pub role_id: String,
    /// The substrate event the action bot wrote.
    pub event_id: String,
}

/// `PUT /_nuck/v1/guilds/{guildId}/members/{userId}/roles` — assign a member's roles.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SetMemberRolesRequest {
    /// The full role set to assign.
    pub roles: Vec<String>,
}

/// `PUT /_nuck/v1/guilds/{guildId}/bans/{userId}` — ban a member (ADR 0016).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BanRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

/// The outcome of a privileged action: the substrate event the bot wrote (ADR 0041).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionResponse {
    pub event_id: String,
}

/// `PUT /_nuck/v1/channels/{channelId}/overrides/{target}` — set a channel override
/// (ADR 0020).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SetChannelOverrideRequest {
    /// Bits this override grants.
    #[serde(default)]
    pub allow: Permissions,
    /// Bits this override removes. Deny wins over allow within one override.
    #[serde(default)]
    pub deny: Permissions,
}

/// One channel permission override.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelOverride {
    /// A role id (including `@everyone`) or a Matrix user id.
    pub target: String,
    pub allow: Permissions,
    pub deny: Permissions,
}

/// `GET /_nuck/v1/channels/{channelId}/overrides` — a channel's overrides (ADR 0020).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelOverridesResponse {
    pub overrides: Vec<ChannelOverride>,
}

// --- Guild & channel provisioning (MVP) ------------------------------------

/// `POST /_nuck/v1/guilds` — create a guild.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CreateGuildRequest {
    #[serde(default)]
    pub name: Option<String>,
}

/// A newly created guild and its default channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildResponse {
    pub guild_id: String,
    pub channel_id: String,
}

/// `POST /_nuck/v1/guilds/{guildId}/channels` — create a channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(default = "default_channel_kind")]
    pub kind: ChannelKind,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub position: Option<i64>,
}

fn default_channel_kind() -> ChannelKind {
    ChannelKind::Text
}

/// A channel as the guild view reports it.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelInfo {
    pub channel_id: String,
    pub kind: String,
    pub position: i64,
    pub category: Option<String>,
}

/// A role as the guild view reports it.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RoleInfo {
    pub role_id: String,
    pub name: Option<String>,
    pub position: i64,
    pub permissions: Permissions,
    pub color: Option<u32>,
    pub hoist: bool,
    pub mentionable: bool,
}

/// `GET /_nuck/v1/guilds/{guildId}` — the aggregated guild view.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildView {
    pub guild_id: String,
    pub name: Option<String>,
    pub home_server: String,
    pub owner: Option<String>,
    pub channels: Vec<ChannelInfo>,
    pub roles: Vec<RoleInfo>,
}

// ===========================================================================
// v1 surface (docs/protocol/api.md). Additive over the MVP types above.
// ===========================================================================

/// The `/_nuck/v1` endpoint inventory, as data (kept in step with
/// `docs/protocol/api.md`; `tests/api_registry.rs` checks its structure).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoint {
    pub method: &'static str,
    pub path: &'static str,
    /// One of `public`, `as-user`, `authorized+bot`, `admin`.
    pub auth: &'static str,
}

/// Every endpoint in the v1 inventory.
pub const ENDPOINTS: &[Endpoint] = &[
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/server-info",
        auth: "public",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/me",
        auth: "as-user",
    },
    // Registration (ADR 0037)
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/register/policy",
        auth: "public",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/register",
        auth: "public",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/register/requests",
        auth: "admin",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/register/requests/{id}/approve",
        auth: "admin",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/register/requests/{id}/deny",
        auth: "admin",
    },
    // Guilds (ADR 0005/0006)
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/guilds",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds",
        auth: "as-user",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds/{guildId}",
        auth: "as-user",
    },
    Endpoint {
        method: "PATCH",
        path: "/_nuck/v1/guilds/{guildId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/guilds/{guildId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds/{guildId}/members",
        auth: "as-user",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/guilds/{guildId}/members/{userId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds/{guildId}/roles",
        auth: "as-user",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/guilds/{guildId}/roles",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "PATCH",
        path: "/_nuck/v1/guilds/{guildId}/roles/{roleId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/guilds/{guildId}/roles/{roleId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "PUT",
        path: "/_nuck/v1/guilds/{guildId}/members/{userId}/roles",
        auth: "authorized+bot",
    },
    // Channels (ADR 0005/0020)
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds/{guildId}/channels",
        auth: "as-user",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/guilds/{guildId}/channels",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "PATCH",
        path: "/_nuck/v1/channels/{channelId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/channels/{channelId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/channels/{channelId}/overrides",
        auth: "as-user",
    },
    Endpoint {
        method: "PUT",
        path: "/_nuck/v1/channels/{channelId}/overrides/{target}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/channels/{channelId}/overrides/{target}",
        auth: "authorized+bot",
    },
    // Invites & joining (ADR 0011/0025)
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/guilds/{guildId}/invites",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds/{guildId}/invites",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/guilds/{guildId}/invites/{code}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/invites/{code}",
        auth: "public",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/invites/{code}/redeem",
        auth: "as-user",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/guilds/{guildId}/join-requests",
        auth: "as-user",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds/{guildId}/join-requests",
        auth: "as-user",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/guilds/{guildId}/join-requests/{userId}/approve",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/guilds/{guildId}/join-requests/{userId}/deny",
        auth: "authorized+bot",
    },
    // Friends, blocking & DMs (ADR 0030)
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/friends",
        auth: "as-user",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/friends/requests",
        auth: "as-user",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/friends/requests",
        auth: "as-user",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/friends/requests/{userId}/accept",
        auth: "as-user",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/friends/requests/{userId}/decline",
        auth: "as-user",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/friends/{userId}",
        auth: "as-user",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/blocks",
        auth: "as-user",
    },
    Endpoint {
        method: "PUT",
        path: "/_nuck/v1/blocks/{userId}",
        auth: "as-user",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/blocks/{userId}",
        auth: "as-user",
    },
    Endpoint {
        method: "POST",
        path: "/_nuck/v1/dm",
        auth: "as-user",
    },
    // Moderation & audit log (ADR 0016)
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds/{guildId}/bans",
        auth: "as-user",
    },
    Endpoint {
        method: "PUT",
        path: "/_nuck/v1/guilds/{guildId}/bans/{userId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "DELETE",
        path: "/_nuck/v1/guilds/{guildId}/bans/{userId}",
        auth: "authorized+bot",
    },
    Endpoint {
        method: "GET",
        path: "/_nuck/v1/guilds/{guildId}/audit-log",
        auth: "as-user",
    },
];

// --- Server ----------------------------------------------------------------

/// `GET /_nuck/v1/me` — current profile plus Nuck settings (account data
/// `space.nuck.settings.v1`).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Me {
    pub user_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub settings: serde_json::Value,
}

// --- Registration (ADR 0037) -----------------------------------------------

/// The operator's registration policy (ADR 0037).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationMode {
    Open,
    Token,
    Approval,
    Closed,
}

/// `GET /_nuck/v1/register/policy` — public, so a client can render the form.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterPolicy {
    pub mode: RegistrationMode,
    pub token_required: bool,
    pub email_required: bool,
    /// Always `false` in v1 (Tuwunel has no CAPTCHA; ADR 0037 verification).
    pub captcha: bool,
}

/// `POST /_nuck/v1/register` — public.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    /// Required when the policy is `token`.
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// The immediate result (modes `open` / `token`).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub access_token: String,
    pub device_id: String,
    pub home_server: String,
}

/// The deferred result (mode `approval`, HTTP 202).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterPendingResponse {
    pub request_id: String,
}

/// One pending registration request (admin queue).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterRequestInfo {
    pub id: String,
    pub username: String,
    pub created_at: u64,
}

/// `GET /_nuck/v1/register/requests` — admin.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterRequestsResponse {
    pub items: Vec<RegisterRequestInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// The outcome of approving or denying a registration request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegisterDecisionResponse {
    /// The created user id (approve only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

// --- Guilds (ADR 0005/0006) -------------------------------------------------

/// `GET /_nuck/v1/guilds` — the caller's guilds.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildsResponse {
    pub guilds: Vec<GuildSummary>,
}

/// A guild as the list reports it.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildSummary {
    pub guild_id: String,
    #[serde(default)]
    pub name: Option<String>,
    pub home_server: String,
    #[serde(default)]
    pub owner: Option<String>,
}

/// `PATCH /_nuck/v1/guilds/{guildId}` — update guild settings (all optional).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateGuildRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub join_mode: Option<JoinMode>,
    #[serde(default)]
    pub directory_visibility: Option<DirectoryVisibility>,
}

/// One guild member.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemberInfo {
    pub user_id: String,
    /// The projected `m.room.member` membership, e.g. `join` / `invite` / `leave`.
    pub membership: String,
    pub roles: Vec<String>,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// `GET /_nuck/v1/guilds/{guildId}/members` — paginated.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MembersResponse {
    pub items: Vec<MemberInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// `GET /_nuck/v1/guilds/{guildId}/roles`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RolesResponse {
    pub roles: Vec<RoleInfo>,
}

/// `PATCH /_nuck/v1/guilds/{guildId}/roles/{roleId}` — all optional.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateRoleRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub color: Option<u32>,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default)]
    pub permissions: Option<Permissions>,
    #[serde(default)]
    pub hoist: Option<bool>,
    #[serde(default)]
    pub mentionable: Option<bool>,
}

/// `DELETE /_nuck/v1/guilds/{guildId}/members/{userId}` — kick (ADR 0016).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct KickRequest {
    #[serde(default)]
    pub reason: Option<String>,
}

// --- Channels (ADR 0005/0020) ----------------------------------------------

/// `GET /_nuck/v1/guilds/{guildId}/channels` — ordered by position, then id.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChannelsResponse {
    pub channels: Vec<ChannelInfo>,
}

/// `PATCH /_nuck/v1/channels/{channelId}` — all optional.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateChannelRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub kind: Option<ChannelKind>,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default)]
    pub category: Option<String>,
}

// --- Invites & joining (ADR 0011/0025) -------------------------------------

/// `POST /_nuck/v1/guilds/{guildId}/invites`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CreateInviteRequest {
    /// Use cap; absent = unlimited.
    #[serde(default)]
    pub max_uses: Option<u32>,
    /// Lifetime; absent = never expires.
    #[serde(default)]
    pub expires_in_ms: Option<u64>,
}

/// A newly created invite.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateInviteResponse {
    pub code: String,
    /// A shareable `https://…/invite/{code}` URL.
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
}

/// One invite as listed.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InviteInfo {
    pub code: String,
    pub uses: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    pub created_by: String,
}

/// `GET /_nuck/v1/guilds/{guildId}/invites` — paginated.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvitesResponse {
    pub items: Vec<InviteInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// `GET /_nuck/v1/invites/{code}` — public pre-redeem view.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvitePreview {
    pub guild_id: String,
    #[serde(default)]
    pub guild_name: Option<String>,
    #[serde(default)]
    pub guild_icon_url: Option<String>,
    pub member_count: u64,
    pub join_mode: JoinMode,
}

/// `POST /_nuck/v1/invites/{code}/redeem` — the joined guild + default channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RedeemResponse {
    pub guild_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
}

/// `POST /_nuck/v1/guilds/{guildId}/join-requests`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JoinRequestCreate {
    #[serde(default)]
    pub message: Option<String>,
}

/// One join request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinRequestInfo {
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub status: JoinRequestStatus,
    pub requested_at: u64,
}

/// The result of submitting a join request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinRequestResponse {
    pub user_id: String,
    pub status: JoinRequestStatus,
}

/// `GET /_nuck/v1/guilds/{guildId}/join-requests` — paginated, oldest first.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JoinRequestsResponse {
    pub items: Vec<JoinRequestInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

// --- Friends, blocking & DMs (ADR 0030) ------------------------------------

/// One friend.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FriendInfo {
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// When the friendship was established (ms since the epoch).
    pub since: u64,
}

/// `GET /_nuck/v1/friends`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FriendsResponse {
    pub friends: Vec<FriendInfo>,
}

/// Whether a pending request is incoming or outgoing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FriendRequestDirection {
    Incoming,
    Outgoing,
}

/// One pending friend request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FriendRequestInfo {
    /// The other party (requester when incoming, target when outgoing).
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub direction: FriendRequestDirection,
    pub created_at: u64,
}

/// `GET /_nuck/v1/friends/requests` — oldest first.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FriendRequestsResponse {
    pub requests: Vec<FriendRequestInfo>,
}

/// `POST /_nuck/v1/friends/requests`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FriendRequestCreate {
    pub user_id: String,
    #[serde(default)]
    pub message: Option<String>,
}

/// A user's block list.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlocksResponse {
    pub blocked: Vec<String>,
}

/// `POST /_nuck/v1/dm`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateDmRequest {
    pub user_id: String,
}

/// The DM room, created or existing.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateDmResponse {
    pub room_id: String,
}

// --- Moderation & audit log (ADR 0016) -------------------------------------

/// One guild ban.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BanInfo {
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub banned_by: String,
    pub created_at: u64,
}

/// `GET /_nuck/v1/guilds/{guildId}/bans` — paginated.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BansResponse {
    pub items: Vec<BanInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// One audit-log entry (ADR 0016/0019).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub actor: String,
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub event_id: String,
    pub created_at: u64,
}

/// `GET /_nuck/v1/guilds/{guildId}/audit-log` — paginated, newest first.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AuditLogResponse {
    pub items: Vec<AuditEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}
