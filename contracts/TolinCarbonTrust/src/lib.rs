
#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol, log};

const PRODUCER_DATA: Symbol = symbol_short!("P_DATA");

#[derive(Clone, Copy)]
pub enum ProductType {
    CoconutShell,  // 10 CCT/kg
    Activated,     // 15 CCT/kg
    Briquette,     // 8 CCT/kg
}

#[contract]
pub struct TolinCarbonTrust;

#[contractimpl]
impl TolinCarbonTrust {
    /// Returns the coefficient for a given product type.
    fn get_coefficient(product_type: u32) -> u32 {
        match product_type {
            0 => 10, // CoconutShell
            1 => 15, // Activated Carbon
            2 => 8,  // Briquette
            _ => 10, // fallback default
        }
    }

    /// Mints carbon credits based on production data.
    /// product_type: 0 = CoconutShell, 1 = Activated, 2 = Briquette
    pub fn mint_credits(
        env: Env,
        producer: Address,
        production_units: u32,
        product_type: u32,
    ) -> u32 {
        producer.require_auth();

        let coef = Self::get_coefficient(product_type);
        let credits_to_mint = production_units
            .checked_mul(coef)
            .expect("overflow in credit calculation");

        let mut total_credits: u32 = env
            .storage()
            .instance()
            .get(&producer)
            .unwrap_or(0);

        total_credits = total_credits
            .checked_add(credits_to_mint)
            .expect("overflow in total credits");

        env.storage().instance().set(&producer, &total_credits);

        log!(&env, "Credits minted for producer", producer);

        total_credits
    }

    /// Returns the current carbon credit balance for a producer.
    pub fn get_balance(env: Env, producer: Address) -> u32 {
        env.storage().instance().get(&producer).unwrap_or(0)
    }
}