//! Structural checks for the `/_nuck/v1` endpoint inventory and round-trips for the
//! v1 DTOs (docs/protocol/api.md). Unlike the event registry (which has `TYPE`
//! constants), the DTOs have no runtime name, so this file round-trips representative
//! shapes and validates the `ENDPOINTS` table.

use nuck_protocol::api::*;
use nuck_protocol::events::{ChannelKind, DirectoryVisibility, JoinMode, JoinRequestStatus};
use nuck_protocol::permissions::Permissions;
use serde::de::DeserializeOwned;
use serde::Serialize;

fn roundtrip<T: Serialize + DeserializeOwned>(value: T) {
    let json = serde_json::to_string(&value).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("reserialize");
    assert_eq!(json, json2, "round-trip mismatch");
}

#[test]
fn endpoints_are_unique_and_well_formed() {
    use std::collections::HashSet;

    let mut seen = HashSet::new();
    for e in ENDPOINTS {
        assert!(
            e.path.starts_with("/_nuck/v1/"),
            "path outside the v1 namespace: {}",
            e.path
        );
        assert!(
            matches!(e.auth, "public" | "as-user" | "authorized+bot" | "admin"),
            "unknown auth class {} on {} {}",
            e.auth,
            e.method,
            e.path
        );
        assert!(
            matches!(e.method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE"),
            "unexpected method {} on {}",
            e.method,
            e.path
        );
        assert!(
            seen.insert((e.method, e.path)),
            "duplicate endpoint: {} {}",
            e.method,
            e.path
        );
    }
    assert!(ENDPOINTS.len() >= 45, "inventory looks truncated");
}

#[test]
fn server_and_auth_shapes_round_trip() {
    roundtrip(ServerInfo {
        server_name: "localhost".into(),
        product_version: "0.3.0".into(),
        protocol_version: 1,
        protocol_min_version: 1,
        capabilities: vec!["guilds".into()],
    });
    roundtrip(ApiError::new("M_FORBIDDEN", "nope"));
    roundtrip(ApiError {
        errcode: "M_LIMIT_EXCEEDED".into(),
        error: "slow down".into(),
        retry_after_ms: Some(5000),
    });
    roundtrip(Me {
        user_id: "@a:localhost".into(),
        display_name: Some("A".into()),
        avatar_url: None,
        settings: serde_json::json!({}),
    });
}

#[test]
fn registration_shapes_round_trip() {
    roundtrip(RegisterPolicy {
        mode: RegistrationMode::Approval,
        token_required: false,
        email_required: true,
        captcha: false,
    });
    roundtrip(RegisterRequest {
        username: "alice".into(),
        password: "secret".into(),
        token: Some("t".into()),
        email: None,
        display_name: Some("Alice".into()),
    });
    roundtrip(RegisterResponse {
        user_id: "@alice:localhost".into(),
        access_token: "tok".into(),
        device_id: "DEV".into(),
        home_server: "localhost".into(),
    });
    roundtrip(RegisterPendingResponse {
        request_id: "r1".into(),
    });
    roundtrip(RegisterRequestsResponse {
        items: vec![RegisterRequestInfo {
            id: "r1".into(),
            username: "alice".into(),
            created_at: 1,
        }],
        next_cursor: None,
    });
    roundtrip(RegisterDecisionResponse {
        user_id: Some("@a:localhost".into()),
    });
}

#[test]
fn guild_shapes_round_trip() {
    roundtrip(GuildsResponse {
        guilds: vec![GuildSummary {
            guild_id: "!g:localhost".into(),
            name: Some("G".into()),
            home_server: String::new(),
            owner: Some("@a:localhost".into()),
        }],
    });
    roundtrip(UpdateGuildRequest {
        name: Some("G".into()),
        topic: None,
        icon_url: None,
        join_mode: Some(JoinMode::Approval),
        directory_visibility: Some(DirectoryVisibility::Hidden),
    });
    roundtrip(MembersResponse {
        items: vec![MemberInfo {
            user_id: "@a:localhost".into(),
            membership: "join".into(),
            roles: vec!["everyone".into()],
            display_name: None,
        }],
        next_cursor: Some("c".into()),
    });
    roundtrip(UpdateRoleRequest {
        name: Some("mod".into()),
        color: Some(0xff0000),
        position: Some(3),
        permissions: Some(Permissions::default()),
        hoist: Some(true),
        mentionable: None,
    });
    roundtrip(KickRequest {
        reason: Some("spam".into()),
    });
    roundtrip(CreateRoleRequest {
        name: "mod".into(),
        color: Some(0x00ff00),
        position: 2,
        permissions: Permissions::default(),
        hoist: false,
        mentionable: true,
    });
}

#[test]
fn channel_shapes_round_trip() {
    roundtrip(ChannelsResponse {
        channels: vec![ChannelInfo {
            channel_id: "!c:localhost".into(),
            kind: "text".into(),
            position: 0,
            category: None,
        }],
    });
    roundtrip(UpdateChannelRequest {
        name: Some("general".into()),
        topic: None,
        kind: Some(ChannelKind::Text),
        position: Some(1),
        category: None,
    });
}

#[test]
fn invite_shapes_round_trip() {
    roundtrip(CreateInviteRequest {
        max_uses: Some(5),
        expires_in_ms: Some(3_600_000),
    });
    roundtrip(CreateInviteResponse {
        code: "nuck_abc".into(),
        url: "https://localhost/invite/nuck_abc".into(),
        expires_at: None,
    });
    roundtrip(InvitePreview {
        guild_id: "!g:localhost".into(),
        guild_name: Some("G".into()),
        guild_icon_url: None,
        member_count: 3,
        join_mode: JoinMode::Open,
    });
    roundtrip(RedeemResponse {
        guild_id: "!g:localhost".into(),
        channel_id: Some("!c:localhost".into()),
    });
    roundtrip(JoinRequestResponse {
        user_id: "@a:localhost".into(),
        status: JoinRequestStatus::Pending,
    });
    roundtrip(JoinRequestsResponse {
        items: vec![JoinRequestInfo {
            user_id: "@a:localhost".into(),
            message: Some("hi".into()),
            status: JoinRequestStatus::Approved,
            requested_at: 7,
        }],
        next_cursor: None,
    });
}

#[test]
fn friends_and_dm_shapes_round_trip() {
    roundtrip(FriendsResponse {
        friends: vec![FriendInfo {
            user_id: "@b:localhost".into(),
            display_name: Some("B".into()),
            since: 1,
        }],
    });
    roundtrip(FriendRequestsResponse {
        requests: vec![FriendRequestInfo {
            user_id: "@b:localhost".into(),
            message: None,
            direction: FriendRequestDirection::Incoming,
            created_at: 2,
        }],
    });
    roundtrip(FriendRequestCreate {
        user_id: "@b:localhost".into(),
        message: Some("hey".into()),
    });
    roundtrip(BlocksResponse {
        blocked: vec!["@c:localhost".into()],
    });
    roundtrip(CreateDmRequest {
        user_id: "@b:localhost".into(),
    });
    roundtrip(CreateDmResponse {
        room_id: "!dm:localhost".into(),
    });
}

#[test]
fn moderation_shapes_round_trip() {
    roundtrip(BansResponse {
        items: vec![BanInfo {
            user_id: "@b:localhost".into(),
            reason: Some("spam".into()),
            banned_by: "@a:localhost".into(),
            created_at: 9,
        }],
        next_cursor: None,
    });
    roundtrip(AuditLogResponse {
        items: vec![AuditEntry {
            id: 1,
            actor: "@a:localhost".into(),
            action: "ban".into(),
            target: Some("@b:localhost".into()),
            reason: None,
            event_id: "$e".into(),
            created_at: 9,
        }],
        next_cursor: Some("c".into()),
    });
}
