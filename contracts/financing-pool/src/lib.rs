pub mod error;
pub mod types;

pub use error::{FinancingPoolError, DepositStatus, DepositType, InvestmentStatus};
pub use types::{DepositData, CertificateData, InvestmentRequestData, StorageKey, PoolBalance};

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

use crate::error::FinancingPoolError;
use crate::types::StorageKey;

pub trait FinancingPoolTrait {
    fn init(e: Env, admin: Address);
    fn get_admin(e: Env) -> Option<Address>;
    fn is_approved_investor(e: Env, addr: Address) -> bool;
    fn approve_investor(e: Env, caller: Address, addr: Address);
    fn reject_investor(e: Env, caller: Address, addr: Address);
    fn get_investor_status(e: Env, addr: Address) -> Option<u32>;
    fn get_pool_balance(e: Env) -> Option<i128>;
    fn get_deposit_balance(e: Env, dep_key: Symbol) -> Option<i128>;
    fn get_deposit_status(e: Env, dep_key: Symbol) -> Option<u32>;
    fn get_certificate_status(e: Env, cert_key: Symbol) -> Option<u32>;
    fn get_investment_amount(e: Env, inv_key: Symbol) -> Option<i128>;
    fn get_investment_status(e: Env, inv_key: Symbol) -> Option<u32>;
    fn list_approved_investors(e: Env) -> soroban_sdk::Vec<Address>;
    fn list_user_deposits(e: Env, addr: Address) -> soroban_sdk::Vec<Symbol>;
    fn list_user_cad(e: Env, addr: Address) -> soroban_sdk::Vec<Symbol>;
    fn list_open_funding_requests(e: Env) -> soroban_sdk::Vec<Symbol>;

    fn set_approved_investors(e: Env, caller: Address, investors: soroban_sdk::Vec<Address>);

    fn issue_deposit(
        e: Env,
        caller: Address,
        dep_key: Symbol,
        amount: i128,
        deposit_type: u32,
        memo: Symbol,
        InvestNow: bool,
    );

    fn close_deposit(e: Env, caller: Address, dep_key: Symbol);
    fn issue_certificate_against_deposit(
        e: Env,
        caller: Address,
        cert_key: Symbol,
        dep_key: Symbol,
        amount: i128,
        certificate_type: u32,
        payable_amount: i128,
        payment_due_date: u64,
        pool_invest_nonce: u64,
        interest_rate: u32,
        approval_status: u32,
    );

    fn request_investment_withdrawal(e: Env, caller: Address, cert_key: Symbol);
    fn approve_investment_withdrawal(e: Env, caller: Address, cad_key: Symbol);
    fn reject_investment_withdrawal(e: Env, caller: Address, cad_key: Symbol);
    fn release_investment(e: Env, caller: Address, inv_key: Symbol);

    fn fund_invoice_request(
        e: Env,
        caller: Address,
        inv_key: Symbol,
        invoice_id: Symbol,
        amount: i128,
    );

    fn accept_funding(e: Env, caller: Address, inv_key: Symbol);
    fn reject_funding(e: Env, caller: Address, inv_key: Symbol);

    fn release_from_reserve(e: Env, caller: Address, cert_key: Symbol);
    fn transfer_from_reserve(e: Env, caller: Address, dest: Address, amount: i128);
    fn increment_wallet(e: Env, caller: Address, addr: Address, amount: i128);
    fn transfer_deposit(e: Env, caller: Address, from: Address, to: Address, amount: i128);
    fn approve_fund_request(e: Env, caller: Address, req_key: Symbol);
    fn set_role(e: Env, caller: Address, addr: Address, role: u32);
    fn check_status(e: Env, caller: Address, key: Symbol);
    fn transfer_admin(e: Env, caller: Address, new_admin: Address);
    fn update_settings(
        e: Env,
        caller: Address,
        min_deposit_amount: i128,
        max_deposit_amount: i128,
        fee_rate_bips: u32,
    );
}

#[contract]
pub struct FinancingPoolContract;

#[contractimpl]
impl FinancingPoolTrait for FinancingPoolContract {
    fn init(e: Env, admin: Address) {
        admin.require_auth();
        let k = StorageKey::instance("ADMIN");
        e.storage().instance().set(&k, &admin);
        let bk = StorageKey::instance("POOL_BALANCE");
        e.storage().instance().set(&bk, &0i128);
    }

    fn get_admin(e: Env) -> Option<Address> {
        e.storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
    }

    fn is_approved_investor(e: Env, addr: Address) -> bool {
        let k = StorageKey::investor_status(&addr);
        e.storage().persistent().get(&k) == Some(2)
    }

    fn approve_investor(e: Env, caller: Address, addr: Address) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        let k = StorageKey::investor_status(&addr);
        e.storage().persistent().set(&k, &2u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "investor_approved")),
            (addr,),
        );
    }

    fn reject_investor(e: Env, caller: Address, addr: Address) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        let k = StorageKey::investor_status(&addr);
        e.storage().persistent().set(&k, &0u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "investor_rejected")),
            (addr,),
        );
    }

    fn get_investor_status(e: Env, addr: Address) -> Option<u32> {
        e.storage()
            .persistent()
            .get(&StorageKey::investor_status(&addr))
    }

    fn get_pool_balance(e: Env) -> Option<i128> {
        e.storage()
            .instance()
            .get(&StorageKey::instance("POOL_BALANCE"))
    }

    fn get_deposit_balance(e: Env, dep_key: Symbol) -> Option<i128> {
        e.storage()
            .persistent()
            .get(&StorageKey::deposit_balance(&dep_key))
    }

    fn get_deposit_status(e: Env, dep_key: Symbol) -> Option<u32> {
        e.storage()
            .persistent()
            .get(&StorageKey::deposit_status(&dep_key))
    }

    fn get_certificate_status(e: Env, cert_key: Symbol) -> Option<u32> {
        e.storage()
            .persistent()
            .get(&StorageKey::cert_status(&cert_key))
    }

    fn get_investment_amount(e: Env, inv_key: Symbol) -> Option<i128> {
        e.storage()
            .persistent()
            .get(&StorageKey::investment_amount(&inv_key))
    }

    fn get_investment_status(e: Env, inv_key: Symbol) -> Option<u32> {
        e.storage()
            .persistent()
            .get(&StorageKey::investment_status(&inv_key))
    }

    fn list_approved_investors(e: Env) -> soroban_sdk::Vec<Address> {
        soroban_sdk::Vec::new(&e)
    }

    fn list_user_deposits(e: Env, addr: Address) -> soroban_sdk::Vec<Symbol> {
        soroban_sdk::Vec::new(&e)
    }

    fn list_user_cad(e: Env, addr: Address) -> soroban_sdk::Vec<Symbol> {
        soroban_sdk::Vec::new(&e)
    }

    fn list_open_funding_requests(e: Env) -> soroban_sdk::Vec<Symbol> {
        soroban_sdk::Vec::new(&e)
    }

    fn set_approved_investors(
        e: Env,
        caller: Address,
        _investors: soroban_sdk::Vec<Address>,
    ) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "investors_set")),
            (),
        );
    }

    fn issue_deposit(
        e: Env,
        caller: Address,
        dep_key: Symbol,
        amount: i128,
        deposit_type: u32,
        _memo: Symbol,
        _InvestNow: bool,
    ) {
        caller.require_auth();

        let status: u32 = e
            .storage()
            .persistent()
            .get(&StorageKey::investor_status(&caller))
            .unwrap_or(0);

        if status != 2 {
            panic!("Err: NOT_APPROVED_INVESTOR");
        }

        if amount <= 0 {
            panic!("Err: INVALID_AMOUNT");
        }

        let pb: i128 = e
            .storage()
            .instance()
            .get(&StorageKey::instance("POOL_BALANCE"))
            .unwrap_or(0);
        let new_bal = pb + amount;
        e.storage()
            .instance()
            .set(&StorageKey::instance("POOL_BALANCE"), &new_bal);

        e.storage()
            .persistent()
            .set(&StorageKey::deposit_balance(&dep_key), &amount);
        e.storage()
            .persistent()
            .set(&StorageKey::deposit_status(&dep_key), &2u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "deposit_issued")),
            (dep_key, caller, amount, deposit_type),
        );
    }

    fn close_deposit(e: Env, caller: Address, dep_key: Symbol) {
        caller.require_auth();

        let status: u32 = e
            .storage()
            .persistent()
            .get(&StorageKey::deposit_status(&dep_key))
            .unwrap_or(0);

        if status != 2 {
            panic!("Err: INVALID_STATUS");
        }

        let amount: i128 = e
            .storage()
            .persistent()
            .get(&StorageKey::deposit_balance(&dep_key))
            .unwrap_or(0);

        let pb: i128 = e
            .storage()
            .instance()
            .get(&StorageKey::instance("POOL_BALANCE"))
            .unwrap_or(0);
        let new_bal = pb.saturating_sub(amount);
        e.storage()
            .instance()
            .set(&StorageKey::instance("POOL_BALANCE"), &new_bal);

        e.storage()
            .persistent()
            .set(&StorageKey::deposit_status(&dep_key), &3u32);

        let cert_k = StorageKey::cert_status(&dep_key);
        e.storage().persistent().set(&cert_k, &3u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "deposit_closed")),
            (dep_key, caller, amount),
        );
    }

    fn issue_certificate_against_deposit(
        e: Env,
        caller: Address,
        cert_key: Symbol,
        dep_key: Symbol,
        amount: i128,
        _certificate_type: u32,
        _payable_amount: i128,
        _payment_due_date: u64,
        _pool_invest_nonce: u64,
        _interest_rate: u32,
        _approval_status: u32,
    ) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        let dep_status: u32 = e
            .storage()
            .persistent()
            .get(&StorageKey::deposit_status(&dep_key))
            .unwrap_or(0);

        if dep_status != 2 {
            panic!("Err: INVALID_DEPOSIT");
        }

        let dep_balance: i128 = e
            .storage()
            .persistent()
            .get(&StorageKey::deposit_balance(&dep_key))
            .unwrap_or(0);

        if amount > dep_balance {
            panic!("Err: INSUFFICIENT");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::cert_status(&cert_key), &2u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "cert_issued")),
            (cert_key, dep_key, amount),
        );
    }

    fn request_investment_withdrawal(e: Env, caller: Address, cert_key: Symbol) {
        caller.require_auth();

        let status: u32 = e
            .storage()
            .persistent()
            .get(&StorageKey::cert_status(&cert_key))
            .unwrap_or(0);

        if status != 2 {
            panic!("Err: INVALID_STATUS");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::cert_status(&cert_key), &4u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "withdrawal_req")),
            (cert_key, caller),
        );
    }

    fn approve_investment_withdrawal(e: Env, caller: Address, cert_key: Symbol) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::cert_status(&cert_key), &5u32);

        let pb: i128 = e
            .storage()
            .instance()
            .get(&StorageKey::instance("POOL_BALANCE"))
            .unwrap_or(0);
        e.storage()
            .instance()
            .set(&StorageKey::instance("POOL_BALANCE"), &pb);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "withdrawal_approved")),
            (cert_key,),
        );
    }

    fn reject_investment_withdrawal(e: Env, caller: Address, cert_key: Symbol) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::cert_status(&cert_key), &6u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "withdrawal_rejected")),
            (cert_key,),
        );
    }

    fn release_investment(e: Env, caller: Address, inv_key: Symbol) {
        caller.require_auth();

        let status: u32 = e
            .storage()
            .persistent()
            .get(&StorageKey::investment_status(&inv_key))
            .unwrap_or(0);

        if status != 2 {
            panic!("Err: INVALID_STATUS");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::investment_status(&inv_key), &3u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "investment_released")),
            (inv_key, caller),
        );
    }

    fn fund_invoice_request(
        e: Env,
        caller: Address,
        inv_key: Symbol,
        invoice_id: Symbol,
        amount: i128,
    ) {
        caller.require_auth();

        let inv_status: u32 = e
            .storage()
            .persistent()
            .get(&StorageKey::investment_status(&inv_key))
            .unwrap_or(0);

        if inv_status != 1 {
            panic!("Err: INVALID_STATUS");
        }

        if amount <= 0 {
            panic!("Err: INVALID_AMOUNT");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::investment_amount(&inv_key), &amount);
        e.storage()
            .persistent()
            .set(&StorageKey::investment_status(&inv_key), &2u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "invoice_funded")),
            (inv_key, invoice_id, caller, amount),
        );
    }

    fn accept_funding(e: Env, caller: Address, inv_key: Symbol) {
        caller.require_auth();
        e.storage()
            .persistent()
            .set(&StorageKey::investment_status(&inv_key), &7u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "funding_accepted")),
            (inv_key, caller),
        );
    }

    fn reject_funding(e: Env, caller: Address, inv_key: Symbol) {
        caller.require_auth();
        e.storage()
            .persistent()
            .set(&StorageKey::investment_status(&inv_key), &8u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "funding_rejected")),
            (inv_key, caller),
        );
    }

    fn release_from_reserve(e: Env, caller: Address, cert_key: Symbol) {
        caller.require_auth();

        let status: u32 = e
            .storage()
            .persistent()
            .get(&StorageKey::cert_status(&cert_key))
            .unwrap_or(0);
        if status != 3 {
            panic!("Err: INVALID_STATUS");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::cert_status(&cert_key), &9u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "reserve_released")),
            (cert_key, caller),
        );
    }

    fn transfer_from_reserve(e: Env, _caller: Address, _dest: Address, _amount: i128) {
        panic!("Err: USE_STANDARD_TRANSFER");
    }

    fn increment_wallet(e: Env, caller: Address, _addr: Address, _amount: i128) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "wallet_incremented")),
            (),
        );
    }

    fn transfer_deposit(
        e: Env,
        _caller: Address,
        _from: Address,
        _to: Address,
        _amount: i128,
    ) {
        panic!("Err: USE_STANDARD_TRANSFER");
    }

    fn approve_fund_request(e: Env, caller: Address, req_key: Symbol) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::fund_req_status(&req_key), &2u32);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "fund_approved")),
            (req_key,),
        );
    }

    fn set_role(e: Env, caller: Address, addr: Address, role: u32) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        e.storage()
            .persistent()
            .set(&StorageKey::investor_status(&addr), &role);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "role_set")),
            (addr, role),
        );
    }

    fn check_status(e: Env, _caller: Address, key: Symbol) {
        let _s: u32 = e
            .storage()
            .persistent()
            .get(&StorageKey::deposit_status(&key))
            .unwrap_or(0);
    }

    fn transfer_admin(e: Env, caller: Address, new_admin: Address) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        e.storage()
            .instance()
            .set(&StorageKey::instance("ADMIN"), &new_admin);

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "admin_transferred")),
            (new_admin,),
        );
    }

    fn update_settings(
        e: Env,
        caller: Address,
        _min: i128,
        _max: i128,
        _fee_bips: u32,
    ) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("not initialized");
        if caller != admin {
            panic!("Err: NOT_ADMIN");
        }

        e.events().publish(
            (Symbol::new(&e, "pool"), Symbol::new(&e, "settings_updated")),
            (),
        );
    }
}

#[cfg(test)]
pub mod tests;
