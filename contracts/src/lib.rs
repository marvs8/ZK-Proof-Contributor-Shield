#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Bytes, Env, Address};
use soroban_zk_verifier::{Proof, PublicInputs, verify_proof};

#[contracttype]
pub struct BountyRecord {
    pub github_id: u64,
    pub pub_key: Bytes,
    pub amount: i128,
    pub claimed: bool,
}

#[contract]
pub struct ContributorShield;

#[contractimpl]
impl ContributorShield {
    /// Admin registers a bounty for a GitHub contributor.
    pub fn register_bounty(env: Env, github_id: u64, pub_key: Bytes, amount: i128) {
        let record = BountyRecord { github_id, pub_key, amount, claimed: false };
        env.storage().persistent().set(&github_id, &record);
    }

    /// Contributor submits a ZK proof and a stealth address to claim the bounty.
    /// The proof attests: "I know priv_key s.t. scalar_mul(priv_key, G) == pub_key
    ///                     AND github_id != 0"
    pub fn claim_bounty(
        env: Env,
        github_id: u64,
        stealth_address: Address,
        proof: Proof,
        public_inputs: PublicInputs,
    ) {
        let mut record: BountyRecord = env
            .storage()
            .persistent()
            .get(&github_id)
            .expect("bounty not found");

        assert!(!record.claimed, "already claimed");

        // Verify the ZK proof on-chain
        verify_proof(&env, &proof, &public_inputs).expect("invalid proof");

        record.claimed = true;
        env.storage().persistent().set(&github_id, &record);

        // Transfer funds to the stealth address — no link to contributor's main wallet
        let token = soroban_sdk::token::Client::new(&env, &env.current_contract_address());
        token.transfer(&env.current_contract_address(), &stealth_address, &record.amount);
    }
}
