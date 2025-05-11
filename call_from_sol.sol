// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.0;

/**
 * @title Web3LancerPolkadotInterface
 * @dev Interface for interacting with the Polkadot Rust contract via precompile or FFI.
 *      Function selectors and argument types must match the Rust contract ABI.
 */
interface IWeb3LancerPolkadot {
    // Job Management
    function createJob(uint32 clientId, uint128 budget) external returns (uint32 jobId);
    function getJob(uint32 jobId) external view returns (uint32 clientId, uint128 budget, uint8 status);
    function updateJobStatus(uint32 jobId, uint8 newStatus) external returns (bool success);

    // Proposal Management
    function submitProposal(uint32 jobId, uint32 freelancerId, uint128 bidAmount) external returns (uint32 proposalId);
    function getProposal(uint32 proposalId) external view returns (uint32 jobId, uint32 freelancerId, uint128 bidAmount, uint8 status);
    function updateProposalStatus(uint32 proposalId, uint8 newStatus) external returns (bool success);

    // Agreement Management
    function createAgreementFromProposal(uint32 proposalId) external returns (uint32 agreementId);
    function getAgreement(uint32 agreementId) external view returns (uint32 jobId, uint32 clientId, uint32 freelancerId, uint128 totalAmount, uint8 status);
    function updateAgreementStatus(uint32 agreementId, uint8 newStatus) external returns (bool success);
}

/**
 * @title Web3LancerCaller
 * @dev Example contract that calls into the Polkadot Rust contract for Web3Lancer logic.
 */
contract Web3LancerCaller {
    IWeb3LancerPolkadot public polkadotContract;

    constructor(address polkadotContractAddress) {
        polkadotContract = IWeb3LancerPolkadot(polkadotContractAddress);
    }

    // --- Job Management ---
    function createJob(uint32 clientId, uint128 budget) external returns (uint32) {
        return polkadotContract.createJob(clientId, budget);
    }

    function getJob(uint32 jobId) external view returns (uint32, uint128, uint8) {
        return polkadotContract.getJob(jobId);
    }

    function updateJobStatus(uint32 jobId, uint8 newStatus) external returns (bool) {
        return polkadotContract.updateJobStatus(jobId, newStatus);
    }

    // --- Proposal Management ---
    function submitProposal(uint32 jobId, uint32 freelancerId, uint128 bidAmount) external returns (uint32) {
        return polkadotContract.submitProposal(jobId, freelancerId, bidAmount);
    }

    function getProposal(uint32 proposalId) external view returns (uint32, uint32, uint128, uint8) {
        return polkadotContract.getProposal(proposalId);
    }

    function updateProposalStatus(uint32 proposalId, uint8 newStatus) external returns (bool) {
        return polkadotContract.updateProposalStatus(proposalId, newStatus);
    }

    // --- Agreement Management ---
    function createAgreementFromProposal(uint32 proposalId) external returns (uint32) {
        return polkadotContract.createAgreementFromProposal(proposalId);
    }

    function getAgreement(uint32 agreementId) external view returns (uint32, uint32, uint32, uint128, uint8) {
        return polkadotContract.getAgreement(agreementId);
    }

    function updateAgreementStatus(uint32 agreementId, uint8 newStatus) external returns (bool) {
        return polkadotContract.updateAgreementStatus(agreementId, newStatus);
    }
}
