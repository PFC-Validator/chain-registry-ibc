use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize,Debug)]
pub struct IbcChainInfo {
    pub chain_name: String,
    pub client_id: String,
    pub connection_id: String,
}

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct IbcChannelInfo {
    pub channel_id: String,
    pub port_id: String,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct IbcChannelTags {
    pub status: String,
    pub preferred: bool,
    pub dex: String,
}

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct IbcChannel {
    pub chain_1: IbcChannelInfo,
    pub chain_2: IbcChannelInfo,
    pub ordering: String,
    pub version: String,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct IbcData {
    pub chain_1: IbcChainInfo,
    pub chain_2: IbcChainInfo,
    pub channels: Vec<IbcChannel>,
}