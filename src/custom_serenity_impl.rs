use serenity::all::{MembershipState, TeamMemberRole};
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
            _ => {
                write!(f, "Other")
            }
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
            _ => {
                write!(f, "Unknown")
            }
        }
    }
}
