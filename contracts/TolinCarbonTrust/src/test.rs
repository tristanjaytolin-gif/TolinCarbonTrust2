#![cfg(test)]

mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    fn setup() -> (Env, Address, TolinCarbonTrustClient<'static>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, TolinCarbonTrust);
        let client = TolinCarbonTrustClient::new(&env, &contract_id);
        let producer = Address::generate(&env);
        env.mock_all_auths();
        (env, producer, client)
    }

    // ── Happy path ──────────────────────────────────────────────

    #[test]
    fn test_mint_coconut_shell() {
        let (_, producer, client) = setup();
        // 5 kg × 10 CCT/kg = 50
        let balance = client.mint_credits(&producer, &5, &0);
        assert_eq!(balance, 50);
    }

    #[test]
    fn test_mint_activated_carbon() {
        let (_, producer, client) = setup();
        // 4 kg × 15 CCT/kg = 60
        let balance = client.mint_credits(&producer, &4, &1);
        assert_eq!(balance, 60);
    }

    #[test]
    fn test_mint_briquette() {
        let (_, producer, client) = setup();
        // 5 kg × 8 CCT/kg = 40
        let balance = client.mint_credits(&producer, &5, &2);
        assert_eq!(balance, 40);
    }

    #[test]
    fn test_unknown_product_type_uses_default() {
        let (_, producer, client) = setup();
        // product_type 99 → fallback coefficient 10
        let balance = client.mint_credits(&producer, &3, &99);
        assert_eq!(balance, 30);
    }

    // ── Cumulative minting ──────────────────────────────────────

    #[test]
    fn test_cumulative_same_type() {
        let (_, producer, client) = setup();
        client.mint_credits(&producer, &5, &0); // 50
        let balance = client.mint_credits(&producer, &5, &0); // +50
        assert_eq!(balance, 100);
    }

    #[test]
    fn test_cumulative_mixed_types() {
        let (_, producer, client) = setup();
        client.mint_credits(&producer, &10, &0); // 100 (coconut)
        client.mint_credits(&producer, &4, &1);  // +60 (activated)
        let balance = client.mint_credits(&producer, &5, &2); // +40 (briquette)
        assert_eq!(balance, 200);
    }

    // ── State verification ──────────────────────────────────────

    #[test]
    fn test_get_balance_reflects_minted() {
        let (env, producer, client) = setup();
        client.mint_credits(&producer, &10, &0); // 100
        assert_eq!(client.get_balance(&producer), 100);
    }

    #[test]
    fn test_get_balance_fresh_address_returns_zero() {
        let (env, _, client) = setup();
        let stranger = Address::generate(&env);
        assert_eq!(client.get_balance(&stranger), 0);
    }

    #[test]
    fn test_producers_are_isolated() {
        let (env, producer_a, client) = setup();
        let producer_b = Address::generate(&env);

        client.mint_credits(&producer_a, &10, &0); // A: 100
        client.mint_credits(&producer_b, &5, &1);  // B: 75

        assert_eq!(client.get_balance(&producer_a), 100);
        assert_eq!(client.get_balance(&producer_b), 75);
    }

    // ── Edge cases ──────────────────────────────────────────────

    #[test]
    fn test_zero_production_yields_zero_credits() {
        let (_, producer, client) = setup();
        let balance = client.mint_credits(&producer, &0, &0);
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_single_unit_all_types() {
        let (env, _, client) = setup();
        let p0 = Address::generate(&env);
        let p1 = Address::generate(&env);
        let p2 = Address::generate(&env);

        assert_eq!(client.mint_credits(&p0, &1, &0), 10);
        assert_eq!(client.mint_credits(&p1, &1, &1), 15);
        assert_eq!(client.mint_credits(&p2, &1, &2), 8);
    }

    // ── Auth guard ──────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn test_unauthorized_mint_panics() {
        let env = Env::default();
        let contract_id = env.register_contract(None, TolinCarbonTrust);
        let client = TolinCarbonTrustClient::new(&env, &contract_id);
        let producer = Address::generate(&env);
        // No mock_all_auths() → require_auth() panics
        client.mint_credits(&producer, &5, &0);
    }
}
