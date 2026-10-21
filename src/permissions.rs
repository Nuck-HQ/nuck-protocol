//! Nuck permissions: a `u64` bitfield aligned with Discord's set, plus reserved
//! Nuck extension bits (ADR 0020).
//!
//! The bitfield layout and the power-level mapping in [`mirror`] are a **versioned
//! protocol commitment** (ADR 0014): never repurpose a bit, and bump the protocol
//! version on breaking changes.

use serde::{Deserialize, Serialize};

/// The lowest bit reserved for Nuck-specific extension permissions (ADR 0020).
///
/// Bits `56..=63` are reserved: `56..=59` are assigned to named extensions, `60..=63`
/// are reserved and unassigned. Discord-aligned permissions never occupy these bits.
pub const NUCK_EXTENSION_BIT_MIN: u32 = 56;

/// The highest reserved bit (inclusive).
pub const NUCK_EXTENSION_BIT_MAX: u32 = 63;


/// A Nuck permission bitfield.
///
/// Serialized as a plain `u64`. High bits (56..=63) are reserved for Nuck
/// extensions; if the set ever exceeds `u64`, add a second field rather than
/// repurposing bits (ADR 0020).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Permissions(u64);

impl Permissions {
    /// No permissions.
    pub const NONE: Self = Self(0);

    /// Every permission bit.
    pub const ALL: Self = Self(u64::MAX);

    // --- Discord-aligned bits (ADR 0020) -----------------------------------

    pub const CREATE_INSTANT_INVITE: Self = Self::bit(0);
    pub const KICK_MEMBERS: Self = Self::bit(1);
    pub const BAN_MEMBERS: Self = Self::bit(2);
    pub const ADMINISTRATOR: Self = Self::bit(3);
    pub const MANAGE_CHANNELS: Self = Self::bit(4);
    pub const MANAGE_GUILD: Self = Self::bit(5);
    pub const ADD_REACTIONS: Self = Self::bit(6);
    pub const VIEW_AUDIT_LOG: Self = Self::bit(7);
    pub const PRIORITY_SPEAKER: Self = Self::bit(8);
    pub const STREAM: Self = Self::bit(9);
    pub const VIEW_CHANNEL: Self = Self::bit(10);
    pub const SEND_MESSAGES: Self = Self::bit(11);
    pub const SEND_TTS_MESSAGES: Self = Self::bit(12);
    pub const MANAGE_MESSAGES: Self = Self::bit(13);
    pub const EMBED_LINKS: Self = Self::bit(14);
    pub const ATTACH_FILES: Self = Self::bit(15);
    pub const READ_MESSAGE_HISTORY: Self = Self::bit(16);
    pub const MENTION_EVERYONE: Self = Self::bit(17);
    pub const USE_EXTERNAL_EMOJIS: Self = Self::bit(18);
    pub const CONNECT: Self = Self::bit(20);
    pub const SPEAK: Self = Self::bit(21);
    pub const MUTE_MEMBERS: Self = Self::bit(22);
    pub const DEAFEN_MEMBERS: Self = Self::bit(23);
    pub const MOVE_MEMBERS: Self = Self::bit(24);
    pub const CHANGE_NICKNAME: Self = Self::bit(26);
    pub const MANAGE_NICKNAMES: Self = Self::bit(27);
    pub const MANAGE_ROLES: Self = Self::bit(28);
    pub const MANAGE_WEBHOOKS: Self = Self::bit(29);
    pub const MANAGE_GUILD_EXPRESSIONS: Self = Self::bit(30);
    pub const USE_APPLICATION_COMMANDS: Self = Self::bit(31);
    pub const MANAGE_EVENTS: Self = Self::bit(33);
    pub const MANAGE_THREADS: Self = Self::bit(34);
    pub const CREATE_PUBLIC_THREADS: Self = Self::bit(35);
    pub const CREATE_PRIVATE_THREADS: Self = Self::bit(36);
    pub const USE_EXTERNAL_STICKERS: Self = Self::bit(37);
    pub const SEND_MESSAGES_IN_THREADS: Self = Self::bit(38);
    pub const MODERATE_MEMBERS: Self = Self::bit(40);
    pub const SEND_VOICE_MESSAGES: Self = Self::bit(46);

    // --- Nuck extension bits (reserved 56..=63, ADR 0020) ------------------

    /// Manage the guild's federation settings (blocklists, hosting policy).
    pub const MANAGE_FEDERATION: Self = Self::bit(56);
    /// Register, manage, and revoke guild bots (ADR 0018).
    pub const MANAGE_BOTS: Self = Self::bit(57);
    /// Toggle the guild's visibility in the public directory (ADR 0021).
    pub const MANAGE_DIRECTORY_LISTING: Self = Self::bit(58);
    /// Manage voice channels specifically (mute/limit/move in voice).
    pub const MANAGE_VOICE_CHANNELS: Self = Self::bit(59);

    const fn bit(n: u32) -> Self {
        Self(1u64 << n)
    }

    /// Wrap raw bits (e.g. from storage or a federated event).
    #[must_use]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// The raw bits.
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Whether **all** bits in `other` are set.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether **any** bit in `other` is set.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

impl std::ops::BitOr for Permissions {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

/// A single documented permission: its name, bit, and its **ordering level**.
///
/// The table is the canonical bit assignment — a **versioned protocol commitment**
/// (ADR 0014/0020). Add entries at the end; never change an existing entry's bit or
/// meaning.
#[derive(Clone, Copy, Debug)]
pub struct PermissionDef {
    /// Stable machine name (used in audit logs, docs, and bot APIs).
    pub name: &'static str,
    /// The bit.
    pub permission: Permissions,
    /// The level a member is treated as holding this permission *at*.
    ///
    /// This is **not** a substrate mapping and **not** an enforcement threshold: since
    /// ADR 0041 all fine-grained authorization is computed by [`crate::authorization`]
    /// from role membership, and the substrate sees only the coarse
    /// [`coarse_power_levels`]. It survives as (a) the ordering fed to
    /// [`effective_level`] and (b) documented metadata for the legacy fine-grained
    /// mirror (ADR 0040), which is kept only to explain why mirroring is not the
    /// enforcement path.
    pub required_level: i64,
}

/// The canonical permission table. Add entries at the end; never change an existing
/// entry's bit or meaning.
pub const PERMISSION_TABLE: &[PermissionDef] = &[
    PermissionDef { name: "CREATE_INSTANT_INVITE", permission: Permissions::CREATE_INSTANT_INVITE, required_level: 0 },
    PermissionDef { name: "KICK_MEMBERS", permission: Permissions::KICK_MEMBERS, required_level: 50 },
    PermissionDef { name: "BAN_MEMBERS", permission: Permissions::BAN_MEMBERS, required_level: 50 },
    PermissionDef { name: "ADMINISTRATOR", permission: Permissions::ADMINISTRATOR, required_level: 100 },
    PermissionDef { name: "MANAGE_CHANNELS", permission: Permissions::MANAGE_CHANNELS, required_level: 50 },
    PermissionDef { name: "MANAGE_GUILD", permission: Permissions::MANAGE_GUILD, required_level: 50 },
    PermissionDef { name: "ADD_REACTIONS", permission: Permissions::ADD_REACTIONS, required_level: 0 },
    PermissionDef { name: "VIEW_AUDIT_LOG", permission: Permissions::VIEW_AUDIT_LOG, required_level: 50 },
    PermissionDef { name: "PRIORITY_SPEAKER", permission: Permissions::PRIORITY_SPEAKER, required_level: 0 },
    PermissionDef { name: "STREAM", permission: Permissions::STREAM, required_level: 0 },
    PermissionDef { name: "VIEW_CHANNEL", permission: Permissions::VIEW_CHANNEL, required_level: 0 },
    PermissionDef { name: "SEND_MESSAGES", permission: Permissions::SEND_MESSAGES, required_level: 0 },
    PermissionDef { name: "SEND_TTS_MESSAGES", permission: Permissions::SEND_TTS_MESSAGES, required_level: 0 },
    PermissionDef { name: "MANAGE_MESSAGES", permission: Permissions::MANAGE_MESSAGES, required_level: 50 },
    PermissionDef { name: "EMBED_LINKS", permission: Permissions::EMBED_LINKS, required_level: 0 },
    PermissionDef { name: "ATTACH_FILES", permission: Permissions::ATTACH_FILES, required_level: 0 },
    PermissionDef { name: "READ_MESSAGE_HISTORY", permission: Permissions::READ_MESSAGE_HISTORY, required_level: 0 },
    PermissionDef { name: "MENTION_EVERYONE", permission: Permissions::MENTION_EVERYONE, required_level: 50 },
    PermissionDef { name: "USE_EXTERNAL_EMOJIS", permission: Permissions::USE_EXTERNAL_EMOJIS, required_level: 0 },
    PermissionDef { name: "CONNECT", permission: Permissions::CONNECT, required_level: 0 },
    PermissionDef { name: "SPEAK", permission: Permissions::SPEAK, required_level: 0 },
    PermissionDef { name: "MUTE_MEMBERS", permission: Permissions::MUTE_MEMBERS, required_level: 50 },
    PermissionDef { name: "DEAFEN_MEMBERS", permission: Permissions::DEAFEN_MEMBERS, required_level: 50 },
    PermissionDef { name: "MOVE_MEMBERS", permission: Permissions::MOVE_MEMBERS, required_level: 50 },
    PermissionDef { name: "CHANGE_NICKNAME", permission: Permissions::CHANGE_NICKNAME, required_level: 0 },
    PermissionDef { name: "MANAGE_NICKNAMES", permission: Permissions::MANAGE_NICKNAMES, required_level: 50 },
    PermissionDef { name: "MANAGE_ROLES", permission: Permissions::MANAGE_ROLES, required_level: 50 },
    PermissionDef { name: "MANAGE_WEBHOOKS", permission: Permissions::MANAGE_WEBHOOKS, required_level: 50 },
    PermissionDef { name: "MANAGE_GUILD_EXPRESSIONS", permission: Permissions::MANAGE_GUILD_EXPRESSIONS, required_level: 50 },
    PermissionDef { name: "USE_APPLICATION_COMMANDS", permission: Permissions::USE_APPLICATION_COMMANDS, required_level: 0 },
    PermissionDef { name: "MANAGE_EVENTS", permission: Permissions::MANAGE_EVENTS, required_level: 50 },
    PermissionDef { name: "MANAGE_THREADS", permission: Permissions::MANAGE_THREADS, required_level: 50 },
    PermissionDef { name: "CREATE_PUBLIC_THREADS", permission: Permissions::CREATE_PUBLIC_THREADS, required_level: 0 },
    PermissionDef { name: "CREATE_PRIVATE_THREADS", permission: Permissions::CREATE_PRIVATE_THREADS, required_level: 0 },
    PermissionDef { name: "USE_EXTERNAL_STICKERS", permission: Permissions::USE_EXTERNAL_STICKERS, required_level: 0 },
    PermissionDef { name: "SEND_MESSAGES_IN_THREADS", permission: Permissions::SEND_MESSAGES_IN_THREADS, required_level: 0 },
    PermissionDef { name: "MODERATE_MEMBERS", permission: Permissions::MODERATE_MEMBERS, required_level: 50 },
    PermissionDef { name: "SEND_VOICE_MESSAGES", permission: Permissions::SEND_VOICE_MESSAGES, required_level: 0 },
    // Nuck extensions.
    PermissionDef { name: "MANAGE_FEDERATION", permission: Permissions::MANAGE_FEDERATION, required_level: 50 },
    PermissionDef { name: "MANAGE_BOTS", permission: Permissions::MANAGE_BOTS, required_level: 50 },
    PermissionDef { name: "MANAGE_DIRECTORY_LISTING", permission: Permissions::MANAGE_DIRECTORY_LISTING, required_level: 50 },
    PermissionDef { name: "MANAGE_VOICE_CHANNELS", permission: Permissions::MANAGE_VOICE_CHANNELS, required_level: 50 },
];

/// The standard `m.room.power_levels` thresholds a room carries.
///
/// **This is not a fine-grained permission mirror.** ADR 0040 showed that a scalar
/// power level cannot represent an arbitrary permission set (it over-grants for
/// non-level-closed sets and only covers six actions), so ADR 0041 moved all
/// fine-grained authorization into nuck-server. Rooms are written with the *coarse*
/// [`coarse_power_levels`] — members vs. the action bot — and [`mirror`] / [`Action`]
/// below are retained only to document that finding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowerLevelsMirror {
    pub users_default: i64,
    pub events_default: i64,
    pub state_default: i64,
    pub ban: i64,
    pub kick: i64,
    pub invite: i64,
    pub redact: i64,
}

/// Compute fine-grained power levels for a member holding `permissions`.
///
/// **Legacy — not an enforcement path (ADR 0040/0041).** A scalar power level cannot
/// mirror an arbitrary permission set; this is kept as the evidence for that finding
/// and to drive the `mirror_invariant` tests. Rooms are written with
/// [`coarse_power_levels`] instead.
///
/// The *inverse* direction — a member's effective level — is the maximum
/// `required_level` over the permissions they hold (see [`effective_level`]).
#[must_use]
pub fn mirror(permissions: Permissions) -> PowerLevelsMirror {
    let level = effective_level(permissions);
    PowerLevelsMirror {
        users_default: 0,
        events_default: if permissions.contains(Permissions::SEND_MESSAGES) { 0 } else { 50 },
        state_default: if permissions.contains(Permissions::MANAGE_CHANNELS) { 0 } else { 50 },
        ban: if permissions.contains(Permissions::BAN_MEMBERS) { 50 } else { 100 },
        kick: if permissions.contains(Permissions::KICK_MEMBERS) { 50 } else { 100 },
        invite: if permissions.contains(Permissions::CREATE_INSTANT_INVITE) { 0 } else { 50 },
        redact: if permissions.contains(Permissions::MANAGE_MESSAGES) { 50 } else { 100 },
    }
    .with_floor(level)
}

impl PowerLevelsMirror {
    fn with_floor(self, _level: i64) -> Self {
        self
    }
}

/// The power level implied by a permission set: the highest `required_level` among
/// the permissions held. `ADMINISTRATOR` implies the owner level (100).
#[must_use]
pub fn effective_level(permissions: Permissions) -> i64 {
    if permissions.contains(Permissions::ADMINISTRATOR) {
        return 100;
    }
    PERMISSION_TABLE
        .iter()
        .filter(|def| permissions.contains(def.permission))
        .map(|def| def.required_level)
        .max()
        .unwrap_or(0)
}

/// A substrate-enforceable action: one expressible as a Matrix power-level threshold.
///
/// **Legacy — not an enforcement path (ADR 0040/0041).** Only these six actions can be
/// expressed as room power levels at all, and even they over-grant for non-level-closed
/// sets. This enum exists to document that finding and to drive the `mirror_invariant`
/// tests. Everything else — reactions, attachments, viewing, voice connect/speak, and
/// every fine-grained permission — is enforced by nuck-server (ADR 0041).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Ban,
    Kick,
    Redact,
    Invite,
    SendMessage,
    SetState,
}

impl Action {
    /// Every substrate-enforceable action.
    pub const ALL: [Self; 6] = [
        Self::Ban,
        Self::Kick,
        Self::Redact,
        Self::Invite,
        Self::SendMessage,
        Self::SetState,
    ];

    /// The permission that gates this action.
    #[must_use]
    pub const fn permission(self) -> Permissions {
        match self {
            Self::Ban => Permissions::BAN_MEMBERS,
            Self::Kick => Permissions::KICK_MEMBERS,
            Self::Redact => Permissions::MANAGE_MESSAGES,
            Self::Invite => Permissions::CREATE_INSTANT_INVITE,
            Self::SendMessage => Permissions::SEND_MESSAGES,
            Self::SetState => Permissions::MANAGE_CHANNELS,
        }
    }

    /// The mirrored power-level threshold for this action.
    #[must_use]
    pub const fn threshold(self, mirror: &PowerLevelsMirror) -> i64 {
        match self {
            Self::Ban => mirror.ban,
            Self::Kick => mirror.kick,
            Self::Redact => mirror.redact,
            Self::Invite => mirror.invite,
            Self::SendMessage => mirror.events_default,
            Self::SetState => mirror.state_default,
        }
    }
}

/// Whether the **engine** grants `action` to a member holding `permissions`.
///
/// `ADMINISTRATOR` grants everything (ADR 0020).
#[must_use]
pub fn engine_grants(permissions: Permissions, action: Action) -> bool {
    permissions.contains(Permissions::ADMINISTRATOR) || permissions.contains(action.permission())
}

/// Whether the **substrate** would grant `action`, given the member's effective
/// level and the mirrored power levels.
#[must_use]
pub fn substrate_grants(permissions: Permissions, action: Action) -> bool {
    level_allows(effective_level(permissions), &mirror(permissions), action)
}

/// Whether a member at `level` passes the threshold for `action` under `mirror`.
#[must_use]
pub fn level_allows(level: i64, mirror: &PowerLevelsMirror, action: Action) -> bool {
    level >= action.threshold(mirror)
}

/// The power level of an ordinary guild member (ADR 0041).
pub const MEMBER_LEVEL: i64 = 0;

/// The power level of the privileged **action bot** (ADR 0041).
pub const BOT_LEVEL: i64 = 100;

/// The **coarse** power levels actually written to a room (ADR 0041).
///
/// Power levels are used only to separate members from the action bot; every
/// privileged action is bot-only and all fine-grained policy is enforced by
/// nuck-server. Unlike the fine-grained [`mirror`], this tiering is **exact**:
/// members can send messages, and nothing else in [`Action::ALL`].
#[must_use]
pub const fn coarse_power_levels() -> PowerLevelsMirror {
    PowerLevelsMirror {
        users_default: MEMBER_LEVEL,
        events_default: 0,
        state_default: BOT_LEVEL,
        ban: BOT_LEVEL,
        kick: BOT_LEVEL,
        invite: BOT_LEVEL,
        redact: BOT_LEVEL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitfield_operations() {
        let p = Permissions::SEND_MESSAGES | Permissions::MANAGE_ROLES;
        assert!(p.contains(Permissions::SEND_MESSAGES));
        assert!(p.contains(Permissions::MANAGE_ROLES));
        assert!(!p.contains(Permissions::BAN_MEMBERS));
        assert_eq!(p.difference(Permissions::MANAGE_ROLES), Permissions::SEND_MESSAGES);
    }

    #[test]
    fn administrator_is_owner_level() {
        assert_eq!(effective_level(Permissions::ADMINISTRATOR), 100);
    }

    #[test]
    fn every_table_bit_is_unique() {
        let mut seen = Permissions::NONE.bits();
        for def in PERMISSION_TABLE {
            assert_eq!(seen & def.permission.bits(), 0, "duplicate bit for {}", def.name);
            seen |= def.permission.bits();
        }
    }

    /// Every named permission bit declared on [`Permissions`].
    ///
    /// Deliberately written out rather than derived: adding a permission bit without a
    /// [`PERMISSION_TABLE`] entry — or an entry without a declared bit — must fail the
    /// registry tests below. This is what keeps the table the single documented
    /// commitment (ADR 0020).
    const NAMED_PERMISSIONS: &[Permissions] = &[
        Permissions::CREATE_INSTANT_INVITE,
        Permissions::KICK_MEMBERS,
        Permissions::BAN_MEMBERS,
        Permissions::ADMINISTRATOR,
        Permissions::MANAGE_CHANNELS,
        Permissions::MANAGE_GUILD,
        Permissions::ADD_REACTIONS,
        Permissions::VIEW_AUDIT_LOG,
        Permissions::PRIORITY_SPEAKER,
        Permissions::STREAM,
        Permissions::VIEW_CHANNEL,
        Permissions::SEND_MESSAGES,
        Permissions::SEND_TTS_MESSAGES,
        Permissions::MANAGE_MESSAGES,
        Permissions::EMBED_LINKS,
        Permissions::ATTACH_FILES,
        Permissions::READ_MESSAGE_HISTORY,
        Permissions::MENTION_EVERYONE,
        Permissions::USE_EXTERNAL_EMOJIS,
        Permissions::CONNECT,
        Permissions::SPEAK,
        Permissions::MUTE_MEMBERS,
        Permissions::DEAFEN_MEMBERS,
        Permissions::MOVE_MEMBERS,
        Permissions::CHANGE_NICKNAME,
        Permissions::MANAGE_NICKNAMES,
        Permissions::MANAGE_ROLES,
        Permissions::MANAGE_WEBHOOKS,
        Permissions::MANAGE_GUILD_EXPRESSIONS,
        Permissions::USE_APPLICATION_COMMANDS,
        Permissions::MANAGE_EVENTS,
        Permissions::MANAGE_THREADS,
        Permissions::CREATE_PUBLIC_THREADS,
        Permissions::CREATE_PRIVATE_THREADS,
        Permissions::USE_EXTERNAL_STICKERS,
        Permissions::SEND_MESSAGES_IN_THREADS,
        Permissions::MODERATE_MEMBERS,
        Permissions::SEND_VOICE_MESSAGES,
        Permissions::MANAGE_FEDERATION,
        Permissions::MANAGE_BOTS,
        Permissions::MANAGE_DIRECTORY_LISTING,
        Permissions::MANAGE_VOICE_CHANNELS,
    ];

    #[test]
    fn every_named_permission_is_in_the_table() {
        for bit in NAMED_PERMISSIONS {
            assert!(
                PERMISSION_TABLE.iter().any(|d| d.permission == *bit),
                "bit {:#x} is declared on Permissions but missing from PERMISSION_TABLE",
                bit.bits()
            );
        }
    }

    #[test]
    fn the_table_has_no_undeclared_bits() {
        for def in PERMISSION_TABLE {
            assert!(
                NAMED_PERMISSIONS.contains(&def.permission),
                "{} is in PERMISSION_TABLE but not declared on Permissions",
                def.name
            );
        }
    }

    #[test]
    fn table_names_are_unique() {
        let mut names: Vec<&str> = PERMISSION_TABLE.iter().map(|d| d.name).collect();
        let count = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), count, "duplicate name in PERMISSION_TABLE");
    }

    #[test]
    fn every_table_entry_is_a_single_bit_in_range() {
        for def in PERMISSION_TABLE {
            let bits = def.permission.bits();
            assert_eq!(bits.count_ones(), 1, "{} is not a single bit", def.name);
            assert!(bits.trailing_zeros() < 64, "{} has an out-of-range bit", def.name);
        }
    }

    #[test]
    fn nuck_extensions_live_in_the_reserved_range() {
        for bit in [
            Permissions::MANAGE_FEDERATION,
            Permissions::MANAGE_BOTS,
            Permissions::MANAGE_DIRECTORY_LISTING,
            Permissions::MANAGE_VOICE_CHANNELS,
        ] {
            let index = bit.bits().trailing_zeros();
            assert!(
                (NUCK_EXTENSION_BIT_MIN..=NUCK_EXTENSION_BIT_MAX).contains(&index),
                "extension bit {index} is outside the reserved range 56..=63"
            );
        }
    }
}
