// filepath: /home/nathfavour/Documents/code/web3lancer/web3lancer/contracts/polkadot-contract/src/job_management.rs
#![cfg_attr(not(feature = "std"), no_std)]

use crate::types::{Job, JobId, JobStatus, UserId, Balance, ContractError};
use uapi::{HostFn, HostFnImpl as api};

// For simplicity, we'll use a fixed-size array for storage.
// A real contract would use pallet_contracts::storage or similar.
const MAX_JOBS: usize = 100;
static mut JOBS: [Option<Job>; MAX_JOBS] = [None; MAX_JOBS];
static mut NEXT_JOB_ID: JobId = 0;

fn find_job_index(job_id: JobId) -> Option<usize> {
    unsafe {
        for i in 0..MAX_JOBS {
            if let Some(job) = &JOBS[i] {
                if job.id == job_id {
                    return Some(i);
                }
            }
        }
        None
    }
}

// --- Public Functions (callable via contract ABI) ---

/// Creates a new job.
/// Input: client_id (UserId), budget (Balance)
/// Output: job_id (JobId) or error code
pub fn create_job(client_id: UserId, budget: Balance) -> Result<JobId, ContractError> {
    unsafe {
        if NEXT_JOB_ID >= MAX_JOBS as JobId {
            return Err(ContractError::StorageFull);
        }

        let job_id = NEXT_JOB_ID;
        let new_job = Job {
            id: job_id,
            client_id,
            budget,
            status: JobStatus::Open,
        };

        // Find an empty slot (should be NEXT_JOB_ID if no deletions)
        let mut stored = false;
        for i in 0..MAX_JOBS {
            if JOBS[i].is_none() { // Check if slot is empty
                 JOBS[i] = Some(new_job);
                 stored = true;
                 break;
            }
        }
        if !stored { // Should not happen if MAX_JOBS check is correct
             return Err(ContractError::StorageFull);
        }

        NEXT_JOB_ID += 1;
        Ok(job_id)
    }
}

/// Gets job details.
/// Input: job_id (JobId)
/// Output: (client_id, budget, status_u8) or error code
pub fn get_job(job_id: JobId) -> Result<(UserId, Balance, u8), ContractError> {
    unsafe {
        if let Some(index) = find_job_index(job_id) {
            if let Some(job) = &JOBS[index] {
                return Ok((job.client_id, job.budget, job.status as u8));
            }
        }
        Err(ContractError::NotFound)
    }
}

/// Updates a job's status.
/// Input: job_id (JobId), new_status_u8 (u8)
/// Output: 0 on success or error code
pub fn update_job_status(job_id: JobId, new_status_u8: u8) -> Result<(), ContractError> {
    unsafe {
        let new_status = JobStatus::from_u8(new_status_u8).ok_or(ContractError::InvalidInput)?;

        if let Some(index) = find_job_index(job_id) {
            if let Some(job) = &mut JOBS[index] {
                // Basic state transition validation (can be expanded)
                match (job.status, new_status) {
                    (JobStatus::Open, JobStatus::InProgress) => {},
                    (JobStatus::Open, JobStatus::Cancelled) => {},
                    (JobStatus::InProgress, JobStatus::Completed) => {},
                    (JobStatus::InProgress, JobStatus::Cancelled) => {},
                    _ => return Err(ContractError::InvalidOperation),
                }
                job.status = new_status;
                return Ok(());
            }
        }
        Err(ContractError::NotFound)
    }
}

// --- Helper for ABI encoding/decoding (simplified) ---

/// Decodes create_job arguments: client_id (u32), budget (u128)
pub fn decode_create_job_args(data: &[u8]) -> Result<(UserId, Balance), ContractError> {
    if data.len() < 4 + 16 { // selector (4) + client_id (4) + budget (16)
        return Err(ContractError::InvalidInput);
    }
    let client_id = u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?);
    let budget = u128::from_be_bytes(data[4..20].try_into().map_err(|_| ContractError::InvalidInput)?);
    Ok((client_id, budget))
}

/// Decodes get_job arguments: job_id (u32)
pub fn decode_get_job_args(data: &[u8]) -> Result<JobId, ContractError> {
    if data.len() < 4 { // selector (4) + job_id (4)
        return Err(ContractError::InvalidInput);
    }
    Ok(u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?))
}

/// Decodes update_job_status arguments: job_id (u32), new_status (u8)
pub fn decode_update_job_status_args(data: &[u8]) -> Result<(JobId, u8), ContractError> {
    if data.len() < 4 + 1 { // selector (4) + job_id (4) + status (1)
        return Err(ContractError::InvalidInput);
    }
    let job_id = u32::from_be_bytes(data[0..4].try_into().map_err(|_| ContractError::InvalidInput)?);
    let status = data[4];
    Ok((job_id, status))
}

/// Encodes a JobId result
pub fn encode_job_id_result(result: Result<JobId, ContractError>, output_buffer: &mut [u8; 32]) {
    match result {
        Ok(job_id) => {
            output_buffer[28..].copy_from_slice(&job_id.to_be_bytes());
        }
        Err(err) => {
            output_buffer[31] = err as u8; // Simplified error reporting
        }
    }
}

/// Encodes a GetJob result
pub fn encode_get_job_result(result: Result<(UserId, Balance, u8), ContractError>, output_buffer: &mut [u8; 32]) {
    match result {
        Ok((client_id, budget, status_u8)) => {
            let mut temp_buf = [0u8; 4 + 16 + 1];
            temp_buf[0..4].copy_from_slice(&client_id.to_be_bytes());
            temp_buf[4..20].copy_from_slice(&budget.to_be_bytes());
            temp_buf[20] = status_u8;
            // Right-align in the 32-byte buffer
            let offset = 32 - temp_buf.len();
            output_buffer[offset..].copy_from_slice(&temp_buf);
        }
        Err(err) => {
            output_buffer[31] = err as u8;
        }
    }
}

/// Encodes a simple success/failure result (e.g., for updates)
pub fn encode_simple_result(result: Result<(), ContractError>, output_buffer: &mut [u8; 32]) {
    match result {
        Ok(_) => {
            // output_buffer remains zero for success, or set a specific success code if desired
        }
        Err(err) => {
            output_buffer[31] = err as u8;
        }
    }
}
