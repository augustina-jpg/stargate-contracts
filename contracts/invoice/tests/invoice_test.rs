use invoice::{InvoiceContract, InvoiceContractClient, InvoiceError, InvoiceStatus};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

fn setup() -> (Env, Address, InvoiceContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let id = env.register_contract(None, InvoiceContract);
    let client = InvoiceContractClient::new(&env, &id);
    client.initialize(&admin);
    (env, admin, client)
}

#[test]
fn test_create_invoice_succeeds() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.id, 1);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(invoice.amount_usdc, 10_000_000);
    assert_eq!(invoice.gross_usdc, 10_250_000);
    // Issue #6: payer is None before payment
    assert!(invoice.payer.is_none());
}

#[test]
fn test_mark_paid_requires_admin() {
    let (env, _admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let rogue_admin = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    assert!(client.try_mark_paid(&rogue_admin, &id, &payer).is_err());
}

#[test]
fn test_expired_invoice_cannot_be_paid() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &1);
    env.ledger().with_mut(|ledger| ledger.timestamp += 2);
    assert!(client.try_mark_paid(&admin, &id, &payer).is_err());
}

#[test]
fn test_pause_blocks_create_invoice() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    client.pause(&admin);
    assert!(client
        .try_create_invoice(&merchant, &10_000_000, &10_250_000, &3600)
        .is_err());
}

#[test]
fn test_double_payment_rejected() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    client.mark_paid(&admin, &id, &payer);
    assert!(client.try_mark_paid(&admin, &id, &payer).is_err());
}

// Issue #5: get_invoice returns NotFound for unknown ID
#[test]
fn test_get_invoice_unknown_id_returns_not_found() {
    let (_env, _admin, client) = setup();
    let err = client.try_get_invoice(&999).unwrap_err().unwrap();
    assert_eq!(err, InvoiceError::NotFound);
}

// Issue #5: mark_paid returns NotFound for unknown ID
#[test]
fn test_mark_paid_unknown_id_returns_not_found() {
    let (env, admin, client) = setup();
    let payer = Address::generate(&env);
    let err = client.try_mark_paid(&admin, &999, &payer).unwrap_err().unwrap();
    assert_eq!(err, InvoiceError::NotFound);
}

// Issue #6: payer is set to Some(payer) after payment
#[test]
fn test_payer_set_after_payment() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &3600);
    client.mark_paid(&admin, &id, &payer);
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.payer, Some(payer));
}

// Issue #7: expired event emitted when mark_paid finds stale invoice
#[test]
fn test_expired_event_emitted_on_stale_mark_paid() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &1);
    env.ledger().with_mut(|ledger| ledger.timestamp += 2);
    let _ = client.try_mark_paid(&admin, &id, &payer);
    // Invoice should now be Expired in storage
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.status, InvoiceStatus::Expired);
}

// Issue #8: payment at exactly expires_at is rejected (boundary is exclusive)
#[test]
fn test_payment_at_exact_expiry_is_rejected() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    // expires_in_seconds=10, ledger starts at 0, so expires_at=10
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &10);
    env.ledger().with_mut(|ledger| ledger.timestamp = 10);
    let err = client.try_mark_paid(&admin, &id, &payer).unwrap_err().unwrap();
    assert_eq!(err, InvoiceError::Expired);
}

// Issue #8: payment one second before expiry succeeds
#[test]
fn test_payment_before_expiry_succeeds() {
    let (env, admin, client) = setup();
    let merchant = Address::generate(&env);
    let payer = Address::generate(&env);
    let id = client.create_invoice(&merchant, &10_000_000, &10_250_000, &10);
    env.ledger().with_mut(|ledger| ledger.timestamp = 9);
    client.mark_paid(&admin, &id, &payer);
    let invoice = client.get_invoice(&id).unwrap();
    assert_eq!(invoice.status, InvoiceStatus::Paid);
}
