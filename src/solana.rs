//! Solana RPC client and data structures for validator information.
//!
//! This module provides:
//! - Data structures for validators, gossip nodes, slot voter information, and leader schedule
//! - RPC client wrapper with async operations
//! - Parsing and conversion from Solana RPC responses

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use solana_commitment_config::CommitmentConfig;
use solana_rpc_client_api::response::{RpcContactInfo, RpcVoteAccountInfo};
use solana_sdk::{clock::Slot, pubkey::Pubkey};
use solana_transaction_status::{TransactionDetails, UiTransactionEncoding};
use std::collections::HashSet;
use std::str::FromStr;

use crate::tabs::logs;

/// Information about a Solana validator including voting and staking details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Identity public key of the validator
    pub identity: Pubkey,
    /// Vote account public key
    pub vote_account: Pubkey,
    /// Commission percentage (0-100)
    pub commission: u8,
    /// Last slot the validator voted on
    pub last_vote: Slot,
    /// Last rooted slot
    pub root_slot: Slot,
    /// Total vote credits earned
    pub vote_credits: u64,
    /// Historical epoch credits data (epoch, credits_earned, previous_credits)
    pub epoch_credits: Vec<(u64, u64, u64)>,
    /// Amount of SOL staked to this validator in lamports
    pub activated_stake: u64,
    /// Solana version string
    pub version: String,
    /// Skip rate percentage (calculated from epoch credits)
    pub skip_rate: f64,
}

/// Information about a node in the Solana gossip network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipNodeInfo {
    /// Node's identity public key
    pub pubkey: Pubkey,
    /// Gossip network address (IP:port)
    pub gossip: String,
    /// Transaction Processing Unit address
    pub tpu: Option<String>,
    /// RPC endpoint address
    pub rpc: Option<String>,
    /// QUIC protocol address for TPU
    pub tpu_quic: Option<String>,
    /// Solana version string
    pub version: Option<String>,
    /// Feature set identifier
    pub feature_set: Option<u32>,
    /// Shred version for cluster compatibility
    pub shred_version: Option<u16>,
}

/// Vote transaction information for a voter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteTransactionInfo {
    /// Vote account public key
    pub vote_account: String,
    /// Transaction signature
    pub signature: String,
}

/// Results from searching for voters in a specific slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotVoterInfo {
    /// The slot number that was searched
    pub slot: u64,
    /// Set of vote account public keys that voted in this slot
    pub voters: HashSet<String>,
    /// Vote transactions with signatures
    pub vote_transactions: Vec<VoteTransactionInfo>,
    /// Total number of voters (cached for performance)
    pub total_voters: usize,
}

/// Information about a single leader slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderSlot {
    /// The epoch this slot belongs to
    pub epoch: u64,
    /// The absolute slot number
    pub slot: u64,
    /// Local timestamp for this slot
    pub time_local: DateTime<Local>,
    /// Human-readable time difference from current time
    pub time_diff: String,
}

/// Results from fetching leader schedule for a validator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderScheduleInfo {
    /// The validator identity being tracked
    pub validator_identity: String,
    /// The target epoch
    pub target_epoch: u64,
    /// List of leader slots for this validator
    pub leader_slots: Vec<LeaderSlot>,
    /// Total number of leader slots found
    pub total_slots: usize,
    /// Next upcoming leader slot (closest to current time)
    pub next_leader_slot: Option<LeaderSlot>,
}

/// Async wrapper around Solana RPC client with caching and error handling.
#[derive(Clone)]
pub struct SolanaClient {
    rpc_url: String,
    log_store: logs::LogStore,
}

/// Vote program ID constant for efficient lookups
const VOTE_PROGRAM_ID: &str = "Vote111111111111111111111111111111111111111";

/// Approximate slots per second for Solana network
const SLOTS_PER_SECOND: f64 = 2.5;

impl GossipNodeInfo {
    /// Convert from Solana RPC ContactInfo to our internal representation.
    /// Uses safe defaults for unparseable data.
    pub fn from_rpc_contact_info(contact_info: RpcContactInfo) -> Self {
        let pubkey = contact_info.pubkey.parse().unwrap_or_default();

        Self {
            pubkey,
            gossip: contact_info
                .gossip
                .map(|addr| addr.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            tpu: contact_info.tpu.map(|addr| addr.to_string()),
            rpc: contact_info.rpc.map(|addr| addr.to_string()),
            tpu_quic: contact_info.tpu_quic.map(|addr| addr.to_string()),
            version: contact_info.version,
            feature_set: contact_info.feature_set,
            shred_version: contact_info.shred_version,
        }
    }
}

impl ValidatorInfo {
    /// Convert from Solana RPC VoteAccountInfo to our internal representation.
    /// Calculates skip rate based on latest epoch credits.
    pub fn from_rpc_vote_account(vote_account: RpcVoteAccountInfo) -> Self {
        let identity = vote_account.node_pubkey.parse().unwrap_or_default();
        let vote_account_pubkey = vote_account.vote_pubkey.parse().unwrap_or_default();

        let (vote_credits, skip_rate) = Self::calculate_skip_rate(&vote_account.epoch_credits);

        Self {
            identity,
            vote_account: vote_account_pubkey,
            commission: vote_account.commission,
            last_vote: vote_account.last_vote,
            root_slot: vote_account.root_slot,
            vote_credits,
            epoch_credits: vote_account.epoch_credits,
            activated_stake: vote_account.activated_stake,
            version: "Unknown".to_string(),
            skip_rate,
        }
    }

    /// Calculate skip rate from epoch credits data.
    /// Returns (total_credits, skip_rate_percentage).
    fn calculate_skip_rate(epoch_credits: &[(u64, u64, u64)]) -> (u64, f64) {
        match epoch_credits.last() {
            Some(latest_epoch) => {
                let credits = latest_epoch.1;
                let prev_credits = latest_epoch.2;

                if credits > prev_credits {
                    const SLOTS_PER_EPOCH: f64 = 432_000.0; // Approximate
                    let slots_voted = credits - prev_credits;
                    let vote_rate = slots_voted as f64 / SLOTS_PER_EPOCH;
                    let skip_rate = (1.0 - vote_rate.min(1.0)) * 100.0;
                    (credits, skip_rate.max(0.0))
                } else {
                    (credits, 0.0)
                }
            }
            None => (0, 0.0),
        }
    }
}

impl SolanaClient {
    /// Create a new Solana RPC client wrapper.
    pub fn new(rpc_url: String, log_store: logs::LogStore) -> Self {
        Self { rpc_url, log_store }
    }

    /// Fetch current slot information and epoch data.
    /// Returns (current_slot, latest_slot, current_epoch).
    pub async fn fetch_slot_info(&self) -> Result<(Slot, Slot, u64)> {
        let rpc_url = self.rpc_url.clone();
        let log_store = self.log_store.clone();

        logs::log_request(
            &log_store,
            "get_slot + get_epoch_info",
            &rpc_url,
            &format!("endpoint: {}", rpc_url),
        );

        let result: Result<(Slot, Slot, u64)> = tokio::task::spawn_blocking(move || {
            let client = RpcClient::new(rpc_url);
            let current_slot = client.get_slot()?;
            let epoch_info = client.get_epoch_info()?;

            Ok((current_slot, epoch_info.absolute_slot, epoch_info.epoch))
        })
        .await?;

        match &result {
            Ok((current, latest, epoch)) => {
                logs::log_response(
                    &log_store,
                    "get_slot + get_epoch_info",
                    &self.rpc_url,
                    &format!("current: {}, latest: {}, epoch: {}", current, latest, epoch),
                    "200 OK",
                );
            }
            Err(e) => {
                logs::log_error(
                    &log_store,
                    "get_slot + get_epoch_info",
                    &self.rpc_url,
                    &e.to_string(),
                );
            }
        }

        result
    }

    /// Fetch all current validators from the network.
    pub async fn fetch_validators(&self) -> Result<Vec<ValidatorInfo>> {
        let rpc_url = self.rpc_url.clone();
        let log_store = self.log_store.clone();

        logs::log_request(
            &log_store,
            "get_vote_accounts",
            &rpc_url,
            &format!("endpoint: {}", rpc_url),
        );

        let result: Result<Vec<ValidatorInfo>> = tokio::task::spawn_blocking(move || {
            let client = RpcClient::new(rpc_url);
            let vote_accounts = client.get_vote_accounts()?;

            Ok(vote_accounts
                .current
                .into_iter()
                .map(ValidatorInfo::from_rpc_vote_account)
                .collect::<Vec<_>>())
        })
        .await?;

        match &result {
            Ok(validators) => {
                logs::log_response(
                    &log_store,
                    "get_vote_accounts",
                    &self.rpc_url,
                    &format!("Found {} validators", validators.len()),
                    "200 OK",
                );
            }
            Err(e) => {
                logs::log_error(
                    &log_store,
                    "get_vote_accounts",
                    &self.rpc_url,
                    &e.to_string(),
                );
            }
        }

        result
    }

    /// Fetch all nodes in the gossip network.
    pub async fn fetch_cluster_nodes(&self) -> Result<Vec<GossipNodeInfo>> {
        let rpc_url = self.rpc_url.clone();
        let log_store = self.log_store.clone();

        logs::log_request(
            &log_store,
            "get_cluster_nodes",
            &rpc_url,
            &format!("endpoint: {}", rpc_url),
        );

        let result: Result<Vec<GossipNodeInfo>> = tokio::task::spawn_blocking(move || {
            let client = RpcClient::new(rpc_url);
            let cluster_nodes = client.get_cluster_nodes()?;

            Ok(cluster_nodes
                .into_iter()
                .map(GossipNodeInfo::from_rpc_contact_info)
                .collect::<Vec<_>>())
        })
        .await?;

        match &result {
            Ok(nodes) => {
                logs::log_response(
                    &log_store,
                    "get_cluster_nodes",
                    &self.rpc_url,
                    &format!("Found {} gossip nodes", nodes.len()),
                    "200 OK",
                );
            }
            Err(e) => {
                logs::log_error(
                    &log_store,
                    "get_cluster_nodes",
                    &self.rpc_url,
                    &e.to_string(),
                );
            }
        }

        result
    }

    /// Find all vote accounts that voted in a specific slot.
    /// Analyzes all transactions in the block to identify voting activity.
    pub async fn find_voters_in_slot(&self, slot: u64) -> Result<SlotVoterInfo> {
        let rpc_url = self.rpc_url.clone();
        let log_store = self.log_store.clone();

        logs::log_request(
            &log_store,
            "get_block",
            &rpc_url,
            &format!("slot: {}", slot),
        );

        let result: Result<SlotVoterInfo> = tokio::task::spawn_blocking(move || {
            let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

            let config = RpcBlockConfig {
                encoding: Some(UiTransactionEncoding::Base64),
                transaction_details: Some(TransactionDetails::Full),
                rewards: Some(false),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            };

            let block = client.get_block_with_config(slot, config)?;
            let vote_program_id = Pubkey::from_str(VOTE_PROGRAM_ID)?;
            let mut voters = HashSet::new();
            let mut vote_transactions = Vec::new();

            if let Some(transactions) = block.transactions {
                for encoded_transaction in transactions {
                    if let Some(tx_with_meta) = encoded_transaction.transaction.decode() {
                        // Get the first signature from the transaction's signatures
                        let signature = tx_with_meta
                            .signatures
                            .first()
                            .map(|sig| sig.to_string())
                            .unwrap_or_else(|| "unknown".to_string());

                        Self::extract_voters_from_versioned_transaction_with_signature(
                            &tx_with_meta,
                            &vote_program_id,
                            &mut voters,
                            &mut vote_transactions,
                            &signature,
                        );
                    }
                }
            }

            let total_voters = voters.len();
            Ok(SlotVoterInfo {
                slot,
                voters,
                vote_transactions,
                total_voters,
            })
        })
        .await?;

        match &result {
            Ok(voter_info) => {
                logs::log_response(
                    &log_store,
                    "get_block",
                    &self.rpc_url,
                    &format!(
                        "Found {} voters in slot {}",
                        voter_info.total_voters, voter_info.slot
                    ),
                    "200 OK",
                );
            }
            Err(e) => {
                logs::log_error(&log_store, "get_block", &self.rpc_url, &e.to_string());
            }
        }

        result
    }

    /// Extract vote account addresses and transaction signatures from a single versioned transaction.
    fn extract_voters_from_versioned_transaction_with_signature(
        versioned_tx: &solana_sdk::transaction::VersionedTransaction,
        vote_program_id: &Pubkey,
        voters: &mut HashSet<String>,
        vote_transactions: &mut Vec<VoteTransactionInfo>,
        signature: &str,
    ) {
        let account_keys = versioned_tx.message.static_account_keys();

        for instruction in versioned_tx.message.instructions() {
            let program_id_index = instruction.program_id_index as usize;

            if program_id_index < account_keys.len()
                && account_keys[program_id_index] == *vote_program_id
                && !instruction.accounts.is_empty()
            {
                let vote_account_index = instruction.accounts[0] as usize;
                if vote_account_index < account_keys.len() {
                    let vote_account = account_keys[vote_account_index].to_string();
                    voters.insert(vote_account.clone());
                    vote_transactions.push(VoteTransactionInfo {
                        vote_account,
                        signature: signature.to_string(),
                    });
                }
            }
        }
    }

    /// Fetch leader schedule for a specific validator identity.
    /// Returns leader slots with accurate timestamps.
    pub async fn fetch_leader_schedule(
        &self,
        identity: &str,
        target_epoch: Option<u64>,
    ) -> Result<LeaderScheduleInfo> {
        let rpc_url = self.rpc_url.clone();
        let identity_clone = identity.to_string();
        let log_store = self.log_store.clone();

        logs::log_request(
            &log_store,
            "get_leader_schedule",
            &rpc_url,
            &format!("identity: {}, epoch: {:?}", identity, target_epoch),
        );

        let result: Result<LeaderScheduleInfo> = tokio::task::spawn_blocking(move || {
            let client = RpcClient::new(rpc_url);

            // Parse validator identity
            let validator_pubkey = Pubkey::from_str(&identity_clone)?;

            // Get current network state
            let current_slot = client.get_slot()?;
            let current_timestamp = Utc::now().timestamp();
            let epoch_info = client.get_epoch_info()?;

            let epoch_to_fetch = target_epoch.unwrap_or(epoch_info.epoch);

            // Get epoch schedule for slot calculations
            let epoch_schedule = client.get_epoch_schedule()?;

            // Calculate epoch start slot
            let epoch_start_slot = Self::calculate_epoch_start_slot(
                epoch_to_fetch,
                epoch_info.epoch,
                epoch_info.absolute_slot,
                epoch_info.slot_index,
                epoch_schedule.slots_per_epoch,
                epoch_schedule.first_normal_slot,
            );

            // Get leader schedule
            let leader_schedule = if target_epoch.is_some() {
                client.get_leader_schedule(Some(epoch_to_fetch))?
            } else {
                client.get_leader_schedule(None)?
            };

            match leader_schedule {
                Some(schedule) => {
                    if let Some(slots) = schedule.get(&validator_pubkey.to_string()) {
                        let mut leader_slots = Vec::new();
                        let mut next_leader_slot = None;

                        for &relative_slot in slots {
                            let absolute_slot = epoch_start_slot + relative_slot as u64;
                            let time_local = Self::slot_to_timestamp_local(
                                absolute_slot,
                                SLOTS_PER_SECOND,
                                current_slot,
                                current_timestamp,
                            );
                            let slot_timestamp = time_local.timestamp();
                            let time_diff =
                                Self::format_time_difference(current_timestamp, slot_timestamp);

                            let leader_slot = LeaderSlot {
                                epoch: epoch_to_fetch,
                                slot: absolute_slot,
                                time_local,
                                time_diff,
                            };

                            // Track next upcoming slot
                            if slot_timestamp > current_timestamp && next_leader_slot.is_none() {
                                next_leader_slot = Some(leader_slot.clone());
                            }

                            leader_slots.push(leader_slot);
                        }

                        // Sort by slot number
                        leader_slots.sort_by_key(|slot| slot.slot);

                        let total_slots = leader_slots.len();

                        Ok(LeaderScheduleInfo {
                            validator_identity: identity_clone,
                            target_epoch: epoch_to_fetch,
                            leader_slots,
                            total_slots,
                            next_leader_slot,
                        })
                    } else {
                        Ok(LeaderScheduleInfo {
                            validator_identity: identity_clone,
                            target_epoch: epoch_to_fetch,
                            leader_slots: Vec::new(),
                            total_slots: 0,
                            next_leader_slot: None,
                        })
                    }
                }
                None => Ok(LeaderScheduleInfo {
                    validator_identity: identity_clone,
                    target_epoch: epoch_to_fetch,
                    leader_slots: Vec::new(),
                    total_slots: 0,
                    next_leader_slot: None,
                }),
            }
        })
        .await?;

        match &result {
            Ok(schedule) => {
                logs::log_response(
                    &log_store,
                    "get_leader_schedule",
                    &self.rpc_url,
                    &format!(
                        "Found {} leader slots for {} in epoch {}",
                        schedule.total_slots, schedule.validator_identity, schedule.target_epoch
                    ),
                    "200 OK",
                );
            }
            Err(e) => {
                logs::log_error(
                    &log_store,
                    "get_leader_schedule",
                    &self.rpc_url,
                    &e.to_string(),
                );
            }
        }

        result
    }

    /// Calculate the starting slot for a given epoch.
    fn calculate_epoch_start_slot(
        target_epoch: u64,
        current_epoch: u64,
        current_absolute_slot: u64,
        current_slot_index: u64,
        slots_per_epoch: u64,
        first_normal_slot: u64,
    ) -> u64 {
        if target_epoch == current_epoch {
            // For current epoch, calculate from current position
            current_absolute_slot - current_slot_index
        } else if target_epoch == 0 {
            0
        } else if target_epoch * slots_per_epoch < first_normal_slot {
            // Very early epochs during warmup period
            target_epoch * slots_per_epoch / 2
        } else {
            // Normal epochs after warmup
            first_normal_slot + (target_epoch.saturating_sub(1)) * slots_per_epoch
        }
    }

    /// Convert a slot number to a local timestamp using current network time as reference.
    fn slot_to_timestamp_local(
        slot: u64,
        slots_per_second: f64,
        current_slot: u64,
        current_timestamp: i64,
    ) -> DateTime<Local> {
        let slot_duration_secs = 1.0 / slots_per_second;
        let slot_diff = slot as i64 - current_slot as i64;
        let timestamp = current_timestamp + (slot_diff as f64 * slot_duration_secs) as i64;

        let utc_time = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);
        utc_time.with_timezone(&Local)
    }

    /// Format a time difference in a human-readable way.
    pub fn format_time_difference(current_timestamp: i64, target_timestamp: i64) -> String {
        let diff = target_timestamp - current_timestamp;
        let abs_diff = diff.abs();

        let days = abs_diff / 86400;
        let hours = (abs_diff % 86400) / 3600;
        let minutes = (abs_diff % 3600) / 60;
        let seconds = abs_diff % 60;

        let mut parts = Vec::new();

        if days > 0 {
            parts.push(format!("{}d", days));
        }
        if hours > 0 {
            parts.push(format!("{}h", hours));
        }
        if minutes > 0 {
            parts.push(format!("{}m", minutes));
        }
        if seconds > 0 && days == 0 && hours == 0 {
            parts.push(format!("{}s", seconds));
        }

        if parts.is_empty() {
            "now".to_string()
        } else {
            let formatted = parts.join(" ");
            if diff < 0 {
                format!("{} ago", formatted)
            } else {
                formatted
            }
        }
    }
}
