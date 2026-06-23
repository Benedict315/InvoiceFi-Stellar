pub mod error;
pub mod types;

pub use error::{InvoiceError, InvoiceStatus};
pub use types::{InvoiceData, LeafHashData, StorageKey};

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol};

use crate::error::InvoiceError;
use crate::types::{InvoiceData, StorageKey};

pub trait InvoiceTrait {
    fn init(e: Env, admin: Address, escrow_pubkey: BytesN<32>);
    fn get_issuer(e: Env, invoice_id: Symbol) -> Option<Address>;
    fn get_payee(e: Env, invoice_id: Symbol) -> Option<Address>;
    fn get_buyer(e: Env, invoice_id: Symbol) -> Option<Address>;
    fn get_amount(e: Env, invoice_id: Symbol) -> Option<i128>;
    fn get_due_date(e: Env, invoice_id: Symbol) -> Option<u64>;
    fn get_status(e: Env, invoice_id: Symbol) -> Option<u32>;
    fn get_invoice_merkle_root(e: Env, invoice_id: Symbol) -> Option<Symbol>;
    fn get_user_tree_root(e: Env, user_addr: Address, index: u64) -> Option<Symbol>;
    fn get_user_tree_count(e: Env, user_addr: Address) -> Option<u64>;
    fn list_user_invoices(e: Env, user_addr: Address) -> soroban_sdk::Vec<Symbol>;

    fn mint_invoice(
        e: Env,
        caller: Address,
        invoice_id: Symbol,
        invoice_number: u64,
        issuer: Address,
        payee: Address,
        buyer: Address,
        amount: i128,
        currency_code: Symbol,
        memo: Symbol,
        due_date: u64,
        metadata_hash: Symbol,
        payment_line_hash: Symbol,
        related_invoice_hash: Symbol,
        leaf_hashes: soroban_sdk::Vec<Symbol>,
    );

    fn transfer_invoice(e: Env, caller: Address, invoice_id: Symbol, to: Address);
    fn cancel_invoice(e: Env, caller: Address, invoice_id: Symbol);
    fn approve_invoice(e: Env, caller: Address, invoice_id: Symbol);
    fn accept_invoice(e: Env, caller: Address, invoice_id: Symbol, buyer: Address);
    fn reject_invoice(e: Env, caller: Address, invoice_id: Symbol);
    fn return_invoice(e: Env, caller: Address, invoice_id: Symbol);
    fn request_settlement_auth(
        e: Env,
        caller: Address,
        invoice_id: Symbol,
        leaf_hashes: soroban_sdk::Vec<Symbol>,
    );
    fn set_invoice_merkle_root(e: Env, caller: Address, invoice_id: Symbol, root: Symbol);
    fn add_user_tree_root(e: Env, caller: Address, user_addr: Address, root: Symbol);
}

#[contract]
pub struct InvoiceContract;

#[contractimpl]
impl InvoiceTrait for InvoiceContract {
    fn init(e: Env, admin: Address, escrow_pubkey: BytesN<32>) {
        admin.require_auth();
        e.storage()
            .instance()
            .set(&StorageKey::instance("ADMIN"), &admin);
        e.storage()
            .instance()
            .set(&StorageKey::instance("ESCROW_PUBKEY"), &escrow_pubkey);
    }

    fn get_issuer(e: Env, invoice_id: Symbol) -> Option<Address> {
        e.storage().persistent().get(&StorageKey::invoice_field(
            &invoice_id, "issuer",
        ))
    }

    fn get_payee(e: Env, invoice_id: Symbol) -> Option<Address> {
        e.storage().persistent().get(&StorageKey::invoice_field(
            &invoice_id, "payee",
        ))
    }

    fn get_buyer(e: Env, invoice_id: Symbol) -> Option<Address> {
        e.storage().persistent().get(&StorageKey::invoice_field(
            &invoice_id, "buyer",
        ))
    }

    fn get_amount(e: Env, invoice_id: Symbol) -> Option<i128> {
        e.storage().persistent().get(&StorageKey::invoice_field(
            &invoice_id, "amount",
        ))
    }

    fn get_due_date(e: Env, invoice_id: Symbol) -> Option<u64> {
        e.storage().persistent().get(&StorageKey::invoice_field(
            &invoice_id, "due_date",
        ))
    }

    fn get_status(e: Env, invoice_id: Symbol) -> Option<u32> {
        e.storage()
            .persistent()
            .get(&StorageKey::status(&invoice_id))
    }

    fn get_invoice_merkle_root(e: Env, invoice_id: Symbol) -> Option<Symbol> {
        e.storage().persistent().get(&StorageKey::invoice_field(
            &invoice_id, "tree_root",
        ))
    }

    fn get_user_tree_root(e: Env, user_addr: Address, index: u64) -> Option<Symbol> {
        e.storage().persistent().get(&StorageKey::user_tree_root(
            &user_addr, index,
        ))
    }

    fn get_user_tree_count(e: Env, user_addr: Address) -> Option<u64> {
        e.storage()
            .persistent()
            .get(&StorageKey::user_count(&user_addr))
    }

    fn list_user_invoices(e: Env, user_addr: Address) -> soroban_sdk::Vec<Symbol> {
        let count: u64 = e
            .storage()
            .persistent()
            .get(&StorageKey::user_count(&user_addr))
            .unwrap_or(0);
        let mut out = soroban_sdk::Vec::new(&e);
        for i in 0..count {
            if let Some(root) = e
                .storage()
                .persistent()
                .get(&StorageKey::user_tree_root(&user_addr, i))
            {
                out.push_back(root);
            }
        }
        out
    }

    fn mint_invoice(
        e: Env,
        caller: Address,
        invoice_id: Symbol,
        invoice_number: u64,
        issuer: Address,
        payee: Address,
        buyer: Address,
        amount: i128,
        currency_code: Symbol,
        memo: Symbol,
        due_date: u64,
        metadata_hash: Symbol,
        payment_line_hash: Symbol,
        related_invoice_hash: Symbol,
        _leaf_hashes: soroban_sdk::Vec<Symbol>,
    ) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("Invoice: not initialized");

        if caller != admin {
            panic!("Err: UNAUTHORIZED");
        }

        if amount <= 0 {
            panic!("Err: INVALID_AMOUNT");
        }

        let invoice = InvoiceData {
            id: invoice_id.clone(),
            invoice_number,
            issuer: issuer.clone(),
            payee: payee.clone(),
            buyer: buyer.clone(),
            amount,
            currency_code,
            memo,
            due_date,
            metadata_hash,
            payment_line_hash,
            related_invoice_hash,
            status: InvoiceStatus::Draft,
        };

        let inv_key = StorageKey::invoice_data(&invoice_id);
        e.storage().persistent().set(&inv_key, &invoice);
        e.storage()
            .persistent()
            .set(&StorageKey::status(&invoice_id), &(InvoiceStatus::Draft as u32));

        Self::bump_user_tree_nonce(&e, &issuer);

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "minted")),
            (
                invoice_id,
                issuer,
                payee,
                buyer,
                amount,
                currency_code,
                due_date,
            ),
        );
    }

    fn transfer_invoice(e: Env, caller: Address, invoice_id: Symbol, to: Address) {
        caller.require_auth();

        let invoice = Self::load_invoice(&e, &invoice_id)
            .ok_or(InvoiceError::NotFound)
            .unwrap();
        if invoice.issuer != caller && invoice.buyer != caller {
            panic!("Err: NOT_AUTHORIZED");
        }
        if invoice.status != InvoiceStatus::PendingAcceptance
            && invoice.status != InvoiceStatus::Accepted
        {
            panic!("Err: STATUS_NOT_TRANSFERABLE");
        }
        Self::set_status(&e, &invoice_id, InvoiceStatus::PendingAcceptance);
        let inv_key = StorageKey::invoice_data(&invoice_id);
        let mut updated = invoice;
        let old_issuer = updated.issuer.clone();
        updated.payee = to.clone();
        updated.issuer = caller.clone();
        e.storage().persistent().set(&inv_key, &updated);

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "transferred")),
            (invoice_id, old_issuer, to),
        );
    }

    fn cancel_invoice(e: Env, caller: Address, invoice_id: Symbol) {
        caller.require_auth();

        let invoice = Self::load_invoice(&e, &invoice_id)
            .ok_or(InvoiceError::NotFound)
            .unwrap();
        if invoice.issuer != caller {
            panic!("Err: NOT_AUTHORIZED");
        }
        let ok = [
            InvoiceStatus::Draft,
            InvoiceStatus::PendingAcceptance,
            InvoiceStatus::Accepted,
        ];
        if !ok.contains(&invoice.status) {
            panic!("Err: CANNOT_CANCEL");
        }
        Self::set_status(&e, &invoice_id, InvoiceStatus::Cancelled);
        e.storage()
            .persistent()
            .remove(&StorageKey::invoice_data(&invoice_id));

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "cancelled")),
            (invoice_id, caller),
        );
    }

    fn approve_invoice(e: Env, caller: Address, invoice_id: Symbol) {
        caller.require_auth();

        let invoice = Self::load_invoice(&e, &invoice_id)
            .ok_or(InvoiceError::NotFound)
            .unwrap();
        if invoice.buyer != caller {
            panic!("Err: NOT_AUTHORIZED");
        }
        if invoice.status != InvoiceStatus::PendingAcceptance {
            panic!("Err: ALREADY_PROCESSED");
        }
        Self::set_status(&e, &invoice_id, InvoiceStatus::Accepted);

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "approved")),
            (invoice_id, caller),
        );
    }

    fn accept_invoice(e: Env, caller: Address, invoice_id: Symbol, buyer: Address) {
        caller.require_auth();

        let invoice = Self::load_invoice(&e, &invoice_id)
            .ok_or(InvoiceError::NotFound)
            .unwrap();
        if invoice.issuer != caller {
            panic!("Err: NOT_AUTHORIZED");
        }
        if invoice.status != InvoiceStatus::Draft {
            panic!("Err: INVALID_STATUS");
        }
        Self::set_status(&e, &invoice_id, InvoiceStatus::PendingAcceptance);
        let inv_key = StorageKey::invoice_data(&invoice_id);
        let mut updated = invoice;
        updated.buyer = buyer.clone();
        updated.payee = invoice.payee.clone();
        e.storage().persistent().set(&inv_key, &updated);

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "accepted")),
            (invoice_id, buyer),
        );
    }

    fn reject_invoice(e: Env, caller: Address, invoice_id: Symbol) {
        caller.require_auth();

        let invoice = Self::load_invoice(&e, &invoice_id)
            .ok_or(InvoiceError::NotFound)
            .unwrap();
        if invoice.buyer != caller {
            panic!("Err: NOT_AUTHORIZED");
        }
        if invoice.status != InvoiceStatus::PendingAcceptance {
            panic!("Err: INVALID_STATUS");
        }
        Self::set_status(&e, &invoice_id, InvoiceStatus::Rejected);
        e.storage()
            .persistent()
            .remove(&StorageKey::invoice_data(&invoice_id));

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "rejected")),
            (invoice_id, caller),
        );
    }

    fn return_invoice(e: Env, caller: Address, invoice_id: Symbol) {
        caller.require_auth();

        let invoice = Self::load_invoice(&e, &invoice_id)
            .ok_or(InvoiceError::NotFound)
            .unwrap();
        if invoice.buyer != caller {
            panic!("Err: NOT_AUTHORIZED");
        }
        if invoice.status != InvoiceStatus::Accepted {
            panic!("Err: INVALID_STATUS");
        }
        Self::set_status(&e, &invoice_id, InvoiceStatus::PendingAcceptance);

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "returned")),
            (invoice_id, caller),
        );
    }

    fn request_settlement_auth(
        e: Env,
        caller: Address,
        invoice_id: Symbol,
        _leaf_hashes: soroban_sdk::Vec<Symbol>,
    ) {
        caller.require_auth();

        let invoice = Self::load_invoice(&e, &invoice_id)
            .ok_or(InvoiceError::NotFound)
            .unwrap();
        if invoice.issuer != caller && invoice.payee != caller {
            panic!("Err: NOT_AUTHORIZED");
        }
        if invoice.status != InvoiceStatus::Accepted {
            panic!("Err: INVALID_STATUS");
        }
        Self::set_status(
            &e,
            &invoice_id,
            InvoiceStatus::PendingApprovalForSettlement,
        );

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "settlement_auth_req")),
            (invoice_id, caller),
        );
    }

    fn set_invoice_merkle_root(e: Env, caller: Address, invoice_id: Symbol, root: Symbol) {
        caller.require_auth();
        Self::load_invoice(&e, &invoice_id)
            .ok_or(InvoiceError::NotFound)
            .unwrap();
        e.storage().persistent().set(
            &StorageKey::invoice_field(&invoice_id, "tree_root"),
            &root,
        );

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "root_set")),
            (invoice_id, root),
        );
    }

    fn add_user_tree_root(e: Env, caller: Address, user_addr: Address, root: Symbol) {
        caller.require_auth();

        let admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::instance("ADMIN"))
            .expect("Invoice: not initialized");
        if caller != admin {
            panic!("Err: UNAUTHORIZED");
        }
        Self::bump_user_tree_nonce(&e, &user_addr);

        e.events().publish(
            (Symbol::new(&e, "invoice"), Symbol::new(&e, "user_root_added")),
            (user_addr, root),
        );
    }
}

impl InvoiceContract {
    fn load_invoice(e: &Env, invoice_id: &Symbol) -> Option<InvoiceData> {
        e.storage()
            .persistent()
            .get(&StorageKey::invoice_data(invoice_id))
    }

    fn set_status(e: &Env, invoice_id: &Symbol, s: InvoiceStatus) {
        e.storage()
            .persistent()
            .set(&StorageKey::status(invoice_id), &(s as u32));
    }

    fn bump_user_tree_nonce(e: &Env, user: &Address) {
        let ck = StorageKey::user_count(user);
        let count: u64 = e.storage().persistent().get(&ck).unwrap_or(0);
        let nk = StorageKey::user_tree_root(user, count);
        e.storage()
            .persistent()
            .set(&nk, &Symbol::new(e, "INITIAL_ROOT"));
        e.storage().persistent().set(&ck, &(count + 1));
    }
}

#[cfg(test)]
pub mod tests;
