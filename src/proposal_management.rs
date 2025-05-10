// filepath: /home/nathfavour/Documents/code/web3lancer/web3lancer/contracts/polkadot-contract/src/proposal_management.rs
#![cfg_attr(not(feature = "std"), no_std)]

use crate::types::{Proposal, ProposalId, ProposalStatus, JobId, UserId, Balance, ContractError};
use crate::job_management; // To check job status
use uapi::{HostFn, HostFnImpl as api};

const MAX_PROPOSALS: usize = 200; // Example limit
static mut PROPOSALS: [Option<Proposal>; MAX_PROPOSALS] = [None; MAX_PROPOSALS];
static mut NEXT_PROPOSAL_ID: ProposalId = 0;

fn find_proposal_index(proposal_id: ProposalId) -> Option<usize> {
    unsafe {
        for i in 0..MAX_PROPOSALS {
            if let Some(proposal) = &PROPOSALS[i] {
                if proposal.id == proposal_id {
                    return Some(i);
                }
            }
        }
        None
    }
}


/// Submits a new proposal for a job.
/// Input: job_id (JobId), freelancer_id (UserId), bid_amount (Balance)
/// Output: proposal_id (ProposalId) or error code
pub fn submit_proposal(job_id: JobId, freelancer_id: UserId, bid_amount: Balance) -> Result<ProposalId, ContractError> {
    unsafe {
        // Check if job exists and is open (simplified check)
        match job_management::get_job(job_id) {
            Ok((_, _, status_u8)) => {
                if status_u8 != crate::types::JobStatus::Open as u8 {
                    return Err(ContractError::InvalidOperation); // Job not open
                }
            },
            Err(_) => return Err(ContractError::NotFound), // Job not found
        }

        if NEXT_PROPOSAL_ID >= MAX_PROPOSALS as ProposalId {
            return Err(ContractError::StorageFull);
        }

        let proposal_id = NEXT_PROPOSAL_ID;
        let new_proposal = Proposal {
            id: proposal_id,
            job_id,
            freelancer_id,
            bid_amount,
            status: ProposalStatus::Submitted,
        };

        let mut stored = false;
        for i in 0..MAX_PROPOSALS {
            if PROPOSALS[i].is_none() {
                PROPOSALS[i] = Some(new_proposal);
                stored = true;
                break;
            }
        }
        if !stored {
            return Err(ContractError::StorageFull);
        }

        NEXT_PROPOSAL_ID += 1;
        Ok(proposal_id)
    }
}

/// Gets proposal details.
/// Input: proposal_id (ProposalId)
/// Output: (job_id, freelancer_id, bid_amount, status_u8) or error code
pub fn get_proposal(proposal_id: ProposalId) -> Result<(JobId, UserId, Balance, u8), ContractError> {
    unsafe {
        if let Some(index) = find_proposal_index(proposal_id) {
            if let Some(proposal) = &PROPOSALS[index] {
                return Ok((
                    proposal.job_id,
                    proposal.freelancer_id,
                    proposal.bid_amount,
                    proposal.status as u8,
                ));
            }
        }
        Err(ContractError::NotFound)
    }
}

/// Updates a proposal's status (e.g., accept/reject).
/// Input: proposal_id (ProposalId), new_status_u8 (u8)
/// Output: 0 on success or error code
pub fn update_proposal_status(proposal_id: ProposalId, new_status_u8: u8) -> Result<(), ContractError> {
    unsafe {
        let new_status = ProposalStatus::from_u8(new_status_u8).ok_or(ContractError::InvalidInput)?;

        if let Some(index) = find_proposal_index(proposal_id) {
            if let Some(proposal) = &mut PROPOSALS[index] {
                // Basic state transition validation
                match (proposal.status, new_status) {
                    (ProposalStatus::Submitted, ProposalStatus::Accepted) => {},
                    (ProposalStatus::Submitted, ProposalStatus::Rejected) => {},
                    _ => return Err(ContractError::InvalidOperation),
                }
                proposal.status = new_status;
                // Potentially trigger agreement creation if accepted (handled in agreement_management)
                return Ok(());
            }
        }
        Err(ContractError::NotFound)
    }
}

// --- ABI Helper Functions (Simplified) ---

/// Decodes submit_proposal arguments: job_id (u32), freelancer_id (u32), bid_amount (u128)
pub fn decode_submit_proposal_args(data: &[u8]) -> Result<(JobId, UserId, Balance), ContractError> {
    if data.len() < 4 + 4 + 16 { // selector + job_id + freelancer_id + bid_amount
        return Err(ContractError::InvalidInput);
    }
    let job_id = u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?);
    let freelancer_id = u32::from_be_bytes(data[4..8].try_into().map_err(|_| ContractError::InvalidInput)?);
    let bid_amount = u128::from_be_bytes(data[8..24].try_into().map_err(|_| ContractError::InvalidInput)?);
    Ok((job_id, freelancer_id, bid_amount))
}

/// Decodes get_proposal arguments: proposal_id (u32)
pub fn decode_get_proposal_args(data: &[u8]) -> Result<ProposalId, ContractError> {
    if data.len() < 4 { // selector + proposal_id
        return Err(ContractError::InvalidInput);
    }
    Ok(u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?))
}

/// Decodes update_proposal_status arguments: proposal_id (u32), new_status (u8)
pub fn decode_update_proposal_status_args(data: &[u8]) -> Result<(ProposalId, u8), ContractError> {
    if data.len() < 4 + 1 { // selector + proposal_id + status
        return Err(ContractError::InvalidInput);
    }
    let proposal_id = u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?);
    let status = data[4];
    Ok((proposal_id, status))
}

/// Encodes a ProposalId result
pub fn encode_proposal_id_result(result: Result<ProposalId, ContractError>, output_buffer: &mut [u8; 32]) {
    match result {
        Ok(id) => {
            output_buffer[28..].copy_from_slice(&id.to_be_bytes());
        }
        Err(err) => {
            output_buffer[31] = err as u8;
        }
    }
}

/// Encodes a GetProposal result
pub fn encode_get_proposal_result(result: Result<(JobId, UserId, Balance, u8), ContractError>, output_buffer: &mut [u8; 32]) {
    match result {
        Ok((job_id, freelancer_id, bid_amount, status_u8)) => {
            let mut temp_buf = [0u8; 4 + 4 + 16 + 1];
            temp_buf[0..4].copy_from_slice(&job_id.to_be_bytes());
            temp_buf[4..8].copy_from_slice(&freelancer_id.to_be_bytes());
            temp_buf[8..24].copy_from_slice(&bid_amount.to_be_bytes());
            temp_buf[24] = status_u8;
            let offset = 32 - temp_buf.len();
            output_buffer[offset..].copy_from_slice(&temp_buf);
        }
        Err(err) => {
            output_buffer[31] = err as u8;
        }
    }
}
