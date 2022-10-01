use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, Timestamp};
use sg_marketplace::state::SaleType;

#[cw_serde]
// Everything needed to identify and list an NFT on Marketplace
pub struct ListNftPacketData {
    /// Uniquely identifies the collection which the tokens being
    /// transfered belong to on the sending chain.
    pub class_id: String,
    /// URL that points to metadata about the collection. This is not
    /// validated.
    pub class_uri: Option<String>,
    /// Uniquely identifies the token in the NFT collection being transfered.
    pub token_id: String,
    /// URL that points to metadata for the token bein transfered.
    pub token_uri: String,
    /// The address sending the tokens on the sending chain.
    pub sender: String,
    /// The address that should receive the tokens on the receiving
    /// chain.
    // pub receiver: String,
    pub sale_type: SaleType,
    pub price: Coin,
    pub payment_address: String,
    pub expires_at: Timestamp,
}
