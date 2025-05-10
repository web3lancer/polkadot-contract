#![no_main]
#![no_std]

// Declare modules
mod types;
mod job_management;
mod proposal_management;
mod agreement_management;

use uapi::{HostFn, HostFnImpl as api, ReturnFlags};
use types::ContractError;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("unimp");
        core::hint::unreachable_unchecked();
    }
}

/// Constructor - called once per contract.
#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn deploy() {
    // Initialization logic for the Web3Lancer contract can go here.
    // For now, static arrays in modules are initialized at compile time.
}

/// Main entry point when the contract is called.
/// Handles dispatching to different contract functionalities.
#[no_mangle]
#[polkavm_derive::polkavm_export]
pub extern "C" fn call() {
    let mut input_buffer = [0u8; 256]; // Max expected input size
    let call_data_len = api::read_call_data(&mut input_buffer, 0);
    let call_data = &input_buffer[..call_data_len as usize];

    if call_data.len() < 4 {
        // Not enough data for a selector
        api::return_value(ReturnFlags::REVERT, &[]);
        return;
    }

    let selector = u32::from_be_bytes(call_data[0..4].try_into().unwrap());
    let args = &call_data[4..];
    let mut output_buffer = [0u8; 32]; // Standard 32-byte output for single values or errors

    // Dispatch based on selector
    // Selectors are the first 4 bytes of the keccak256 hash of the function signature.
    // Example: "createJob(uint32,uint128)" -> keccak256 -> first 4 bytes
    // For simplicity, we'll use hardcoded placeholder selectors.
    // In a real scenario, generate these properly.

    // Job Management Selectors (Placeholder values)
    const CREATE_JOB_SELECTOR: u32 = 0x00000001;
    const GET_JOB_SELECTOR: u32 = 0x00000002;
    const UPDATE_JOB_STATUS_SELECTOR: u32 = 0x00000003;

    // Proposal Management Selectors (Placeholder values)
    const SUBMIT_PROPOSAL_SELECTOR: u32 = 0x00000010;
    const GET_PROPOSAL_SELECTOR: u32 = 0x00000011;
    const UPDATE_PROPOSAL_STATUS_SELECTOR: u32 = 0x00000012;

    // Agreement Management Selectors (Placeholder values)
    const CREATE_AGREEMENT_SELECTOR: u32 = 0x00000020; // From proposal
    const GET_AGREEMENT_SELECTOR: u32 = 0x00000021;
    const UPDATE_AGREEMENT_STATUS_SELECTOR: u32 = 0x00000022;

    match selector {
        // --- Job Management ---
        CREATE_JOB_SELECTOR => {
            match job_management::decode_create_job_args(args) {
                Ok((client_id, budget)) => {
                    let result = job_management::create_job(client_id, budget);
                    job_management::encode_job_id_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }
        GET_JOB_SELECTOR => {
            match job_management::decode_get_job_args(args) {
                Ok(job_id) => {
                    let result = job_management::get_job(job_id);
                    job_management::encode_get_job_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }
        UPDATE_JOB_STATUS_SELECTOR => {
            match job_management::decode_update_job_status_args(args) {
                Ok((job_id, new_status)) => {
                    let result = job_management::update_job_status(job_id, new_status);
                    job_management::encode_simple_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }

        // --- Proposal Management ---
        SUBMIT_PROPOSAL_SELECTOR => {
            match proposal_management::decode_submit_proposal_args(args) {
                Ok((job_id, freelancer_id, bid_amount)) => {
                    let result = proposal_management::submit_proposal(job_id, freelancer_id, bid_amount);
                    proposal_management::encode_proposal_id_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }
        GET_PROPOSAL_SELECTOR => {
            match proposal_management::decode_get_proposal_args(args) {
                Ok(proposal_id) => {
                    let result = proposal_management::get_proposal(proposal_id);
                    proposal_management::encode_get_proposal_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }
        UPDATE_PROPOSAL_STATUS_SELECTOR => {
            match proposal_management::decode_update_proposal_status_args(args) {
                Ok((proposal_id, new_status)) => {
                    let result = proposal_management::update_proposal_status(proposal_id, new_status);
                    // If proposal is accepted, an agreement should be created.
                    // This might be a separate call or an internal trigger.
                    // For now, let's assume a separate call to create_agreement_from_proposal.
                    job_management::encode_simple_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }

        // --- Agreement Management ---
        CREATE_AGREEMENT_SELECTOR => { // create_agreement_from_proposal
            match agreement_management::decode_create_agreement_args(args) {
                Ok(proposal_id) => {
                    let result = agreement_management::create_agreement_from_proposal(proposal_id);
                    agreement_management::encode_agreement_id_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }
        GET_AGREEMENT_SELECTOR => {
            match agreement_management::decode_get_agreement_args(args) {
                Ok(agreement_id) => {
                    let result = agreement_management::get_agreement(agreement_id);
                    agreement_management::encode_get_agreement_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }
        UPDATE_AGREEMENT_STATUS_SELECTOR => {
            match agreement_management::decode_update_agreement_status_args(args) {
                Ok((agreement_id, new_status)) => {
                    let result = agreement_management::update_agreement_status(agreement_id, new_status);
                    job_management::encode_simple_result(result, &mut output_buffer);
                }
                Err(e) => output_buffer[31] = e as u8,
            }
        }

        _ => {
            // Unknown selector
            api::return_value(ReturnFlags::REVERT, &[]);
            return;
        }
    }

    // Check if the last byte of output_buffer (error indicator) is non-zero
    if output_buffer[31] != 0 {
         // If it's a known error from ContractError enum
        if output_buffer[31] >= ContractError::InvalidOperation as u8 && output_buffer[31] <= ContractError::Unauthorized as u8 {
             api::return_value(ReturnFlags::REVERT, &[output_buffer[31]]); // Return the error code
        } else {
            api::return_value(ReturnFlags::REVERT, &[]); // Generic revert for unknown errors
        }
    } else {
        api::return_value(ReturnFlags::empty(), &output_buffer);
    }
}
