use cw_storage_plus::Map;

/// Same as above, but for NFTs arriving at this contract.
pub const INCOMING_CLASS_TOKEN_TO_CHANNEL: Map<(String, String), String> =
    Map::new("incoming_class_token_to_channel");

/// Maps (class ID, token ID) -> local channel ID. Used to determine
/// the local channel that NFTs have been sent out on.
pub const OUTGOING_CLASS_TOKEN_TO_CHANNEL: Map<(String, String), String> =
    Map::new("outgoing_class_token_to_channel");
