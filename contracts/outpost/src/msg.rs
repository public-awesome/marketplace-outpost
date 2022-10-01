use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, IbcTimeout, Timestamp};
use sg_marketplace::state::SaleType;

#[cw_serde]
pub struct InstantiateMsg {
    pub cw721_base_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Receives an NFT to be IBC transfered away. The `msg` field must
    /// be a binary encoded `IbcSetAskAwayMsg`.
    ReceiveNft(cw721::Cw721ReceiveMsg),
}

#[cw_serde]
pub struct IbcStargazeMarketplaceSetAskMsg {
    /// The *local* channel ID this ought to be sent away on. This
    /// contract must have a connection on this channel.
    pub channel_id: String,
    /// Timeout for the IBC message. TODO: make this optional and set
    /// default?
    pub timeout: IbcTimeout,
    /// Marketplace fields
    pub sale_type: SaleType,
    pub price: Coin,
    pub payment_address: String,
    pub expires_at: Timestamp,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetCount {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}
