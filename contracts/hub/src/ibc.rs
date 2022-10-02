use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, DepsMut, Empty, Env, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacket, IbcPacketAckMsg,
    IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, Reply, Response, StdResult,
    SubMsg, SubMsgResult, WasmMsg,
};
use cw_utils::parse_reply_instantiate_data;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sg_marketplace_outpost::ibc::ListNftPacketData;

use crate::{
    error::Never,
    ibc_helpers::{
        ack_fail, ack_success, get_endpoint_prefix, try_get_ack_error, try_pop_source_prefix,
        validate_order_and_version,
    },
    msg::ExecuteMsg,
    state::{INCOMING_CLASS_TOKEN_TO_CHANNEL, OUTGOING_CLASS_TOKEN_TO_CHANNEL},
    // state::{
    //     CLASS_ID_TO_NFT_CONTRACT, INCOMING_CLASS_TOKEN_TO_CHANNEL, NFT_CONTRACT_TO_CLASS_ID,
    //     OUTGOING_CLASS_TOKEN_TO_CHANNEL,
    // },
    ContractError,
};

/// Submessage reply ID used for instantiating cw721 contracts.
pub(crate) const INSTANTIATE_CW721_REPLY_ID: u64 = 0;
/// Submessages dispatched with this reply ID will set the ack on the
/// response depending on if the submessage execution succeded or
/// failed.
pub(crate) const ACK_AND_DO_NOTHING: u64 = 1;
/// The IBC version this contract expects to communicate with.
pub(crate) const IBC_VERSION: &str = "ics721-1";
/// ACK error text fallback to use if the ACK error message has the
/// same encoding as the ACK success message.
const ACK_ERROR_FALLBACK: &str =
    "an unexpected error occurred - error text is hidden because it would serialize as ACK success";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    // Regardless of if our processing of this packet works we need to
    // commit an ACK to the chain. As such, we wrap all handling logic
    // in a seprate function and on error write out an error ack.
    match do_ibc_packet_receive(deps, env, msg.packet) {
        Ok(response) => Ok(response),
        Err(error) => Ok(IbcReceiveResponse::new()
            .add_attribute("method", "ibc_packet_receive")
            .add_attribute("error", error.to_string())
            .set_ack(ack_fail(&error.to_string()).unwrap())),
    }
}

fn do_ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    packet: IbcPacket,
) -> Result<IbcReceiveResponse, ContractError> {
    // parse the packet data
    let data: ListNftPacketData = from_binary(&packet.data)?;
    // data.validate()?;
    let token_id = data.token_id;

    let local_class_id = try_pop_source_prefix(&packet.src, &data.class_id);

    if let Some(local_class_id) = local_class_id {
        let key = (local_class_id.to_string(), token_id);
        let outgoing_channel =
            OUTGOING_CLASS_TOKEN_TO_CHANNEL.may_load(deps.storage, key.clone())?;
        let returning_to_source = outgoing_channel.map_or(false, |outgoing_channel| {
            outgoing_channel == packet.dest.channel_id
        });
        if returning_to_source {
            // We previously sent this NFT out on this
            // channel. Unlock the local version for the
            // receiver.
            OUTGOING_CLASS_TOKEN_TO_CHANNEL.remove(deps.storage, key);
            // messages.push(Action::Transfer {
            //     class_id: local_class_id.to_string(),
            //     token_id: token,
            // });
            // return Ok(messages);
        }
    }

    // It's not something we've sent out before => mint a new NFT.
    let local_prefix = get_endpoint_prefix(&packet.dest);
    let local_class_id = format!("{}{}", local_prefix, data.class_id);
    INCOMING_CLASS_TOKEN_TO_CHANNEL.save(
        deps.storage,
        (local_class_id.clone(), token_id),
        &packet.dest.channel_id,
    )?;
    // messages.push(Action::InstantiateAndMint {
    //     class_id: local_class_id,
    //     token_id: token,
    //     token_uri,
    // });

    // if the collection does not exist, create it
    // mint the token
    // call set ask on marketplace with token
    Ok(IbcReceiveResponse::default())
}
