//! Tab modules for the Solana UI application.
//!
//! This module organizes all tab functionality into separate modules:
//! - validators: Validator listing and management
//! - gossip_nodes: Gossip network node information
//! - find_voters: Slot voter search functionality
//! - leader_schedule: Validator leader schedule tracking
//! - update: Application update management

pub mod find_voters;
pub mod gossip_nodes;
pub mod leader_schedule;
pub mod logs;
pub mod update;
pub mod validators;

/// Available tabs in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Validators,
    GossipNodes,
    FindVoters,
    LeaderSchedule,
    Logs,
    Update,
}

impl AppTab {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Validators => "📊 Validators",
            Self::GossipNodes => "🌐 Gossip Nodes",
            Self::FindVoters => "🔍 Find Voters",
            Self::LeaderSchedule => "⏰ Leader Schedule",
            Self::Logs => "📋 Logs",
            Self::Update => "🔄 Update",
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Validators => "Validators",
            Self::GossipNodes => "GossipNodes",
            Self::FindVoters => "FindVoters",
            Self::LeaderSchedule => "LeaderSchedule",
            Self::Logs => "Logs",
            Self::Update => "Update",
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id {
            "Validators" => Self::Validators,
            "GossipNodes" => Self::GossipNodes,
            "FindVoters" => Self::FindVoters,
            "LeaderSchedule" => Self::LeaderSchedule,
            "Logs" => Self::Logs,
            "Update" => Self::Update,
            _ => Self::Validators, // Default
        }
    }
}
