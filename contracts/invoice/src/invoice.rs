use soroban_sdk::{contracterror, contracttype, Address};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum InvoiceError {
    Unauthorized = 1,
    ContractPaused = 2,
    InvalidAmount = 3,
    NotPending = 4,
    Expired = 5,
    NotFound = 6,
    AlreadyInitialized = 7,
    ZeroDuration = 8,
    ExpiryOverflow = 9,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvoiceStatus {
    Pending,
    Paid,
    Expired,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaybePayer {
    None,
    Some(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    pub id: u64,
    pub merchant: Address,
    pub amount_usdc: i128,
    pub gross_usdc: i128,
    pub status: InvoiceStatus,
    pub expires_at: u64,
    pub paid_at: Option<u64>,
    pub payer: MaybePayer,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Invoice(u64),
    InvoiceCount,
    Admin,
    Paused,
}
