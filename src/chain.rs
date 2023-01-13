use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct ChainChannelInfo {
    pub channel_src: String,
    pub port_src: String,
//    pub chain_dest:String,
    pub channel_dest: String,
    pub port_dest: String,
    pub ordering: String,
    pub version: String,
}

#[derive(Serialize, Deserialize,Debug)]
pub struct Chain {
    pub chain_name: String,
    pub transfers: HashMap<String,ChainChannelInfo>
}
