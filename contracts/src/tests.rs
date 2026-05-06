#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};
    use crate::ContributorShield;

    #[test]
    fn test_register_and_claim() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ContributorShield);
        let client = crate::ContributorShieldClient::new(&env, &contract_id);

        let github_id: u64 = 12345678;
        let pub_key = Bytes::from_slice(&env, &[0u8; 32]); // placeholder
        let stealth = Address::generate(&env);

        client.register_bounty(&github_id, &pub_key, &1000);

        // Full claim test requires a real proof; this asserts registration succeeds.
        let record: crate::BountyRecord = env
            .storage()
            .persistent()
            .get(&github_id)
            .unwrap();

        assert_eq!(record.amount, 1000);
        assert!(!record.claimed);
    }
}
