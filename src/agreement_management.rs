// filepath: /home/nathfavour/Documents/code/web3lancer/web3lancer/contracts/polkadot-contract/src/agreement_management.rs
#![cfg_attr(not(feature = "std"), no_std)]

use crate::types::{Agreement, AgreementId, AgreementStatus, JobId, UserId, Balance, ContractError};
use crate::job_management; // To update job status
use crate::proposal_management; // To get proposal details
use uapi::{HostFn, HostFnImpl as api};

const MAX_AGREEMENTS: usize = 100; // Example limit
static mut AGREEMENTS: [Option<Agreement>; MAX_AGREEMENTS] = [None; MAX_AGREEMENTS];
static mut NEXT_AGREEMENT_ID: AgreementId = 0;

fn find_agreement_index(agreement_id: AgreementId) -> Option<usize> {
    unsafe {
        for i in 0..MAX_AGREEMENTS {
            if let Some(agreement) = &AGREEMENTS[i] {
                if agreement.id == agreement_id {
                    return Some(i);
                }
            }
        }
        None
    }
}

/// Creates an agreement when a proposal is accepted.
/// Input: proposal_id (ProposalId)
/// Output: agreement_id (AgreementId) or error code
pub fn create_agreement_from_proposal(proposal_id: ProposalId) -> Result<AgreementId, ContractError> {
    unsafe {
        // 1. Fetch proposal details
        let (job_id, freelancer_id, bid_amount, proposal_status_u8) = 
            proposal_management::get_proposal(proposal_id).map_err(|_| ContractError::NotFound)?;

        if proposal_status_u8 != crate::types::ProposalStatus::Accepted as u8 {
            return Err(ContractError::InvalidOperation); // Proposal not accepted
        }

        // 2. Fetch job details to get client_id
        let (client_id, _, job_status_u8) = 
            job_management::get_job(job_id).map_err(|_| ContractError::NotFound)?;
        
        if job_status_u8 != crate::types::JobStatus::Open as u8 {
             // Or if it was already in progress with another freelancer, depends on logic
            return Err(ContractError::InvalidOperation); 
        }

        if NEXT_AGREEMENT_ID >= MAX_AGREEMENTS as AgreementId {
            return Err(ContractError::StorageFull);
        }

        let agreement_id = NEXT_AGREEMENT_ID;
        let new_agreement = Agreement {
            id: agreement_id,
            job_id,
            client_id,
            freelancer_id,
            total_amount: bid_amount, // Or could be job_budget if fixed price
            status: AgreementStatus::Active,
        };

        let mut stored = false;
        for i in 0..MAX_AGREEMENTS {
            if AGREEMENTS[i].is_none() {
                AGREEMENTS[i] = Some(new_agreement);
                stored = true;
                break;
            }
        }
        if !stored {
            return Err(ContractError::StorageFull);
        }
        
        NEXT_AGREEMENT_ID += 1;

        // 3. Update job status to InProgress
        job_management::update_job_status(job_id, crate::types::JobStatus::InProgress as u8)?;

        Ok(agreement_id)
    }
}

/// Gets agreement details.
/// Input: agreement_id (AgreementId)
/// Output: (job_id, client_id, freelancer_id, total_amount, status_u8) or error code
pub fn get_agreement(agreement_id: AgreementId) -> Result<(JobId, UserId, UserId, Balance, u8), ContractError> {
    unsafe {
        if let Some(index) = find_agreement_index(agreement_id) {
            if let Some(agreement) = &AGREEMENTS[index] {
                return Ok((
                    agreement.job_id,
                    agreement.client_id,
                    agreement.freelancer_id,
                    agreement.total_amount,
                    agreement.status as u8,
                ));
            }
        }
        Err(ContractError::NotFound)
    }
}

/// Updates an agreement's status (e.g., complete, dispute).
/// Input: agreement_id (AgreementId), new_status_u8 (u8)
/// Output: 0 on success or error code
pub fn update_agreement_status(agreement_id: AgreementId, new_status_u8: u8) -> Result<(), ContractError> {
    unsafe {
        let new_status = AgreementStatus::from_u8(new_status_u8).ok_or(ContractError::InvalidInput)?;

        if let Some(index) = find_agreement_index(agreement_id) {
            if let Some(agreement) = &mut AGREEMENTS[index] {
                // Basic state transition validation
                match (agreement.status, new_status) {
                    (AgreementStatus::Active, AgreementStatus::Completed) => {
                        // When agreement completes, update the job status as well
                        job_management::update_job_status(agreement.job_id, crate::types::JobStatus::Completed as u8)?;
                    },
                    (AgreementStatus::Active, AgreementStatus::Disputed) => {},
                    // Add more transitions as needed
                    _ => return Err(ContractError::InvalidOperation),
                }
                agreement.status = new_status;
                return Ok(());
            }
        }
        Err(ContractError::NotFound)
    }
}

// --- ABI Helper Functions (Simplified) ---

/// Decodes create_agreement_from_proposal arguments: proposal_id (u32)
pub fn decode_create_agreement_args(data: &[u8]) -> Result<ProposalId, ContractError> {
    if data.len() < 4 { // selector + proposal_id
        return Err(ContractError::InvalidInput);
    }
    Ok(u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?))
}

/// Decodes get_agreement arguments: agreement_id (u32)
pub fn decode_get_agreement_args(data: &[u8]) -> Result<AgreementId, ContractError> {
    if data.len() < 4 { // selector + agreement_id
        return Err(ContractError::InvalidInput);
    }
    Ok(u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?))
}

/// Decodes update_agreement_status arguments: agreement_id (u32), new_status (u8)
pub fn decode_update_agreement_status_args(data: &[u8]) -> Result<(AgreementId, u8), ContractError> {
    if data.len() < 4 + 1 { // selector + agreement_id + status
        return Err(ContractError::InvalidInput);
    }
    let agreement_id = u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?);
    let status = data[4];
    Ok((agreement_id, status))
}

/// Encodes an AgreementId result
pub fn encode_agreement_id_result(result: Result<AgreementId, ContractError>, output_buffer: &mut [u8; 32]) {
    match result {
        Ok(id) => {
            output_buffer[28..].copy_from_slice(&id.to_be_bytes());
        }
        Err(err) => {
            output_buffer[31] = err as u8;
        }
    }
}

/// Encodes a GetAgreement result
pub fn encode_get_agreement_result(result: Result<(JobId, UserId, UserId, Balance, u8), ContractError>, output_buffer: &mut [u8; 32]) {
    match result {
        Ok((job_id, client_id, freelancer_id, total_amount, status_u8)) => {
            let mut temp_buf = [0u8; 4 + 4 + 4 + 16 + 1];
            temp_buf[0..4].copy_from_slice(&job_id.to_be_bytes());
            temp_buf[4..8].copy_from_slice(&client_id.to_be_bytes());
            temp_buf[8..12].copy_from_slice(&freelancer_id.to_be_bytes());
            temp_buf[12..28].copy_from_slice(&total_amount.to_be_bytes());
            temp_buf[28] = status_u8;
            let offset = 32 - temp_buf.len();
            output_buffer[offset..].copy_from_slice(&temp_buf);
        }
        Err(err) => {
            output_buffer[31] = err as u8;
        }
    }
}
