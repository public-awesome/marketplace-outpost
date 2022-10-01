#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::CW721_ICS_CODE_ID;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg-marketplace-outpost";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CW721_ICS_CODE_ID.save(deps.storage, &msg.cw721_base_code_id)?;

    Ok(Response::default()
        .add_attribute("action", "instantiate")
        .add_attribute("cw721_code_id", msg.cw721_base_code_id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ReceiveNft(cw721::Cw721ReceiveMsg {
            sender,
            token_id,
            msg,
        }) => execute::receive_nft(deps, info, sender, token_id, msg),
    }
}

pub mod execute {
    use cosmwasm_std::{from_binary, IbcMsg};

    use crate::{
        ibc::ListNftPacketData,
        msg::IbcStargazeMarketplaceSetAskMsg,
        state::{
            UniversalNftInfoResponse, CLASS_ID_TO_CLASS_URI, CLASS_ID_TO_NFT_CONTRACT,
            NFT_CONTRACT_TO_CLASS_ID, OUTGOING_CLASS_TOKEN_TO_CHANNEL,
        },
    };

    use super::*;

    pub fn receive_nft(
        deps: DepsMut,
        info: MessageInfo,
        token_id: String,
        sender: String,
        msg: Binary,
    ) -> Result<Response, ContractError> {
        let sender = deps.api.addr_validate(&sender)?;
        let msg: IbcStargazeMarketplaceSetAskMsg = from_binary(&msg)?;

        let class_id = if NFT_CONTRACT_TO_CLASS_ID.has(deps.storage, info.sender.clone()) {
            NFT_CONTRACT_TO_CLASS_ID.load(deps.storage, info.sender.clone())?
        } else {
            let class_id = info.sender.to_string();
            // If we do not yet have a class ID for this contract, it is a
            // local NFT and its class ID is its conract address.
            NFT_CONTRACT_TO_CLASS_ID.save(deps.storage, info.sender.clone(), &class_id)?;
            CLASS_ID_TO_NFT_CONTRACT.save(deps.storage, info.sender.to_string(), &info.sender)?;
            // We set class level metadata to None for local NFTs.
            //
            // Merging and usage of this PR may change that:
            // <https://github.com/CosmWasm/cw-nfts/pull/75>
            CLASS_ID_TO_CLASS_URI.save(deps.storage, info.sender.to_string(), &None)?;
            class_id
        };

        let class_uri = CLASS_ID_TO_CLASS_URI
            .may_load(deps.storage, class_id.clone())?
            .flatten();

        let UniversalNftInfoResponse { token_uri, .. } = deps.querier.query_wasm_smart(
            info.sender,
            &cw721::Cw721QueryMsg::NftInfo {
                token_id: token_id.clone(),
            },
        )?;

        let ibc_message = ListNftPacketData {
            class_id: class_id.clone(),
            class_uri,
            token_id: token_id.clone(),
            token_uri: token_uri.unwrap_or_default(), /* Currently token_uri is optional in
                                                       * cw721 - we set to empty string as
                                                       * default. */
            sender: sender.into_string(),
            sale_type: msg.sale_type,
            price: msg.price,
            payment_address: msg.payment_address,
            expires_at: msg.expires_at,
        };
        let ibc_message = IbcMsg::SendPacket {
            channel_id: msg.channel_id.clone(),
            data: to_binary(&ibc_message)?,
            timeout: msg.timeout,
        };

        OUTGOING_CLASS_TOKEN_TO_CHANNEL.save(
            deps.storage,
            (class_id.clone(), token_id.clone()),
            &msg.channel_id,
        )?;

        Ok(Response::default()
            .add_attribute("action", "receive_nft")
            .add_attribute("token_id", token_id)
            .add_attribute("class_id", class_id)
            .add_attribute("channel_id", msg.channel_id)
            .add_message(ibc_message))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query::count(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        // let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: 5 })
    }
}
