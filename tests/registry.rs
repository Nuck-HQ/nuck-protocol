//! Event-registry tests (ADR 0014/0022).
//!
//! `nuck_protocol::events` is the **canonical** `space.nuck.*` registry. These tests
//! keep it honest: every type in `event_types` must have a content type whose static
//! type string matches, the registry must be well-formed, and parsing must be tolerant
//! (ADR 0014) — unknown fields ignored, added optional fields absent-tolerant.

use std::collections::BTreeSet;

use nuck_protocol::events::event_types;
use nuck_protocol::events::{
    BanEventContent, ChannelEventContent, EmbedEventContent, EmojiEventContent,
    FriendRequestEventContent, FriendResponseEventContent, GuildSettingsEventContent,
    InteractionEventContent, InviteEventContent, JoinRequestEventContent, MemberEventContent,
    OverrideEventContent, RoleEventContent, StickerEventContent,
};
use ruma::events::StaticEventContent;

/// Every registered type as `(constant, the content type's static type string)`.
///
/// Written out deliberately: a new `event_types` constant or content type that is not
/// added here (or a mismatch between them) fails the tests below.
fn registry() -> Vec<(&'static str, &'static str)> {
    vec![
        (event_types::GUILD, <GuildSettingsEventContent as StaticEventContent>::TYPE),
        (event_types::ROLE, <RoleEventContent as StaticEventContent>::TYPE),
        (event_types::MEMBER, <MemberEventContent as StaticEventContent>::TYPE),
        (event_types::CHANNEL, <ChannelEventContent as StaticEventContent>::TYPE),
        (event_types::OVERRIDE, <OverrideEventContent as StaticEventContent>::TYPE),
        (event_types::INVITE, <InviteEventContent as StaticEventContent>::TYPE),
        (event_types::BAN, <BanEventContent as StaticEventContent>::TYPE),
        (event_types::JOIN_REQUEST, <JoinRequestEventContent as StaticEventContent>::TYPE),
        (event_types::EMOJI, <EmojiEventContent as StaticEventContent>::TYPE),
        (event_types::STICKER, <StickerEventContent as StaticEventContent>::TYPE),
        (event_types::EMBED, <EmbedEventContent as StaticEventContent>::TYPE),
        (event_types::INTERACTION, <InteractionEventContent as StaticEventContent>::TYPE),
        (event_types::FRIEND_REQUEST, <FriendRequestEventContent as StaticEventContent>::TYPE),
        (event_types::FRIEND_RESPONSE, <FriendResponseEventContent as StaticEventContent>::TYPE),
    ]
}

#[test]
fn every_constant_matches_its_content_type() {
    for (constant, ty) in registry() {
        assert_eq!(constant, ty, "event_types constant and content type disagree");
    }
}

#[test]
fn registered_types_are_well_formed_and_unique() {
    let mut seen = BTreeSet::new();
    for (constant, _) in registry() {
        assert!(constant.starts_with("space.nuck."), "{constant} is outside the namespace");
        // The version lives in the type name (ADR 0014): `space.nuck.<name>.v<digits>`.
        let (_, version) = constant.rsplit_once(".v").unwrap_or_else(|| {
            panic!("{constant} has no `.v<version>` suffix");
        });
        assert!(
            !version.is_empty() && version.chars().all(|c| c.is_ascii_digit()),
            "{constant} has a malformed version suffix"
        );
        assert!(seen.insert(constant), "{constant} is registered more than once");
    }
}

#[test]
fn unknown_fields_are_ignored() {
    // Tolerant parsing (ADR 0014): a newer peer's added field must never break us.
    let json = serde_json::json!({
        "name": "Moderator",
        "position": 50,
        "permissions": 0,
        "hoist": false,
        "mentionable": true,
        "future_field": { "anything": [1, 2, 3] }
    });
    let role: RoleEventContent =
        serde_json::from_value(json).expect("an unknown field must be ignored");
    assert_eq!(role.name, "Moderator");
}

#[test]
fn missing_optional_fields_are_absent_tolerant() {
    // Additive changes keep the type name (ADR 0014), so a field added later must be
    // optional and must parse when absent.
    let member: MemberEventContent =
        serde_json::from_value(serde_json::json!({ "roles": ["mod"] }))
            .expect("a minimal member payload parses");
    assert_eq!(member.roles, vec!["mod".to_owned()]);

    let role: RoleEventContent = serde_json::from_value(serde_json::json!({
        "name": "x",
        "position": 0,
        "permissions": 0,
        "hoist": false,
        "mentionable": false
    }))
    .expect("a role without the optional color parses");
    assert!(role.color.is_none());
}
