// filepath: /home/nathfavour/Documents/code/web3lancer/web3lancer/contracts/polkadot-contract/src/types.rs
#![cfg_attr(not(feature = "std"), no_std)]

pub type JobId = u32;
pub type ProposalId = u32;
pub type AgreementId = u32;
pub type UserId = u32;
pub type Balance = u128;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum JobStatus {
    Open = 0,
    InProgress = 1,
    Completed = 2,
    Cancelled = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum ProposalStatus {
    Submitted = 0,
    Accepted = 1,
    Rejected = 2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum AgreementStatus {
    Active = 0,
    Completed = 1,
    Disputed = 2,
}

// Simplified Job structure
#[derive(Debug, Clone, Copy)]
pub struct Job {
    pub id: JobId,
    pub client_id: UserId,
    pub budget: Balance,
    pub status: JobStatus,
    // For simplicity in ABI encoding, detailed fields like title/description are omitted.
    // In a real scenario, these would be handled, possibly via IPFS hashes or byte arrays.
}

// Simplified Proposal structure
#[derive(Debug, Clone, Copy)]
pub struct Proposal {
    pub id: ProposalId,
    pub job_id: JobId,
    pub freelancer_id: UserId,
    pub bid_amount: Balance,
    pub status: ProposalStatus,
}

// Simplified Agreement structure
#[derive(Debug, Clone, Copy)]
pub struct Agreement {
    pub id: AgreementId,
    pub job_id: JobId,
    pub client_id: UserId,
    pub freelancer_id: UserId,
    pub total_amount: Balance,
    pub status: AgreementStatus,
}

// Basic error type
#[repr(u32)]
pub enum ContractError {
    InvalidOperation = 1,
    NotFound = 2,
    AlreadyExists = 3,
    StorageFull = 4,
    InvalidInput = 5,
    Unauthorized = 6,
}

impl JobStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(JobStatus::Open),
            1 => Some(JobStatus::InProgress),
            2 => Some(JobStatus::Completed),
            3 => Some(JobStatus::Cancelled),
            _ => None,
        }
    }
}

impl ProposalStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ProposalStatus::Submitted),
            1 => Some(ProposalStatus::Accepted),
            2 => Some(ProposalStatus::Rejected),
            _ => None,
        }
    }
}

impl AgreementStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(AgreementStatus::Active),
            1 => Some(AgreementStatus::Completed),
            2 => Some(AgreementStatus::Disputed),
            _ => None,
        }
    }
}
