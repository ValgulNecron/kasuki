use serenity::all::audit_log::Action;
use serenity::all::{MemberAction, MembershipState, TeamMemberRole};
use std::fmt::Display;

pub enum InternalTeamMemberRole {
    Admin,
    Developer,
    ReadOnly,
    Other(String),
}

impl From<TeamMemberRole> for InternalTeamMemberRole {
    fn from(role: TeamMemberRole) -> Self {
        match role {
            TeamMemberRole::Admin => InternalTeamMemberRole::Admin,
            TeamMemberRole::Developer => InternalTeamMemberRole::Developer,
            TeamMemberRole::ReadOnly => InternalTeamMemberRole::ReadOnly,
            TeamMemberRole::Other(a) => InternalTeamMemberRole::Other(a),
            _ => InternalTeamMemberRole::Other("Other".to_string()),
        }
    }
}

impl Display for InternalTeamMemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalTeamMemberRole::Admin => write!(f, "Admin"),
            InternalTeamMemberRole::Developer => write!(f, "Developer"),
            InternalTeamMemberRole::ReadOnly => write!(f, "ReadOnly"),
            InternalTeamMemberRole::Other(a) => write!(f, "Other ({})", a),
        }
    }
}

pub enum InternalMembershipState {
    Invited,
    Accepted,
    Unknown(u8),
}

impl From<MembershipState> for InternalMembershipState {
    fn from(state: MembershipState) -> Self {
        match state {
            MembershipState::Invited => InternalMembershipState::Invited,
            MembershipState::Accepted => InternalMembershipState::Accepted,
            MembershipState::Unknown(a) => InternalMembershipState::Unknown(a),
            _ => InternalMembershipState::Unknown(u8::MAX),
        }
    }
}

impl Display for InternalMembershipState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalMembershipState::Invited => write!(f, "Invited"),
            InternalMembershipState::Accepted => write!(f, "Accepted"),
            InternalMembershipState::Unknown(a) => write!(f, "Unknown ({})", a),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InternalAction {
    Member(InternalMemberAction),
    Other,
}
#[derive(Debug, Clone)]

pub enum InternalMemberAction {
    BanAdd,
    Kick,
    Other,
}

impl PartialEq for InternalAction {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (InternalAction::Member(a), InternalAction::Member(b)) => a == b,
            (InternalAction::Other, InternalAction::Other) => true,
            _ => false,
        }
    }
}

impl PartialEq for InternalMemberAction {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (InternalMemberAction::BanAdd, InternalMemberAction::BanAdd)
                | (InternalMemberAction::Kick, InternalMemberAction::Kick)
                | (InternalMemberAction::Other, InternalMemberAction::Other)
        )
    }
}

impl From<Action> for InternalAction {
    fn from(action: Action) -> Self {
        match action {
            Action::Member(a) => InternalAction::Member(a.into()),
            _ => InternalAction::Other,
        }
    }
}

impl From<MemberAction> for InternalMemberAction {
    fn from(action: MemberAction) -> Self {
        match action {
            MemberAction::BanAdd => InternalMemberAction::BanAdd,
            MemberAction::Kick => InternalMemberAction::Kick,
            _ => InternalMemberAction::Other,
        }
    }
}
