use super::{order::OrderRequest, ClientOrderRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Existing OID-based structures
#[derive(Debug)]
pub struct ClientModifyRequest {
    pub oid: u64,
    pub order: ClientOrderRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifyRequest {
    pub oid: u64,
    pub order: OrderRequest,
}

// New CLOID-based structures
#[derive(Debug)]
pub struct ClientModifyRequestCloid {
    pub cloid: Uuid,
    pub order: ClientOrderRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModifyRequestCloid {
    pub cloid: String,
    pub order: OrderRequest,
}
