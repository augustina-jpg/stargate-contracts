use crate::invoice::Invoice;
use soroban_sdk::{Address, Env, Symbol};

pub fn invoice_created(env: &Env, id: u64, invoice: &Invoice) {
    env.events()
        .publish((Symbol::new(env, "invoice_created"), id), invoice.clone());
}

pub fn invoice_paid(env: &Env, id: u64, invoice: &Invoice) {
    env.events()
        .publish((Symbol::new(env, "invoice_paid"), id), invoice.clone());
}

pub fn invoice_expired(env: &Env, id: u64, invoice: &Invoice) {
    env.events()
        .publish((Symbol::new(env, "invoice_expired"), id), invoice.clone());
}

pub fn contract_paused(env: &Env, admin: &Address) {
    env.events()
        .publish((Symbol::new(env, "contract_paused"),), admin);
}

pub fn contract_unpaused(env: &Env, admin: &Address) {
    env.events()
        .publish((Symbol::new(env, "contract_unpaused"),), admin);
}
