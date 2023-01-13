use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use error_chain::error_chain;

use crate::chain::{Chain, ChainChannelInfo};
use crate::ibc::{ IbcData};

mod ibc;
mod chain;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        SystemTimeError(std::time::SystemTimeError);
        SerdeError(serde_json::Error);
    }
}


fn main() -> Result<()> {
    let current_dir = PathBuf::from("./chain-registry/_IBC");
    println!(
        "Entries modified in the last 24 hours in {:?}:",
        current_dir
    );
    let mut chains: HashMap<String, Chain> = HashMap::new();

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::metadata(&path)?;
        if metadata.is_file() {
            let reader = BufReader::new(fs::File::open(path)?);
            let channel_def: IbcData = serde_json::from_reader(reader)?;
            //  println!("{} - {}", channel_def.chain_1.chain_name, channel_def.chain_2.chain_name);
            let channels = channel_def.channels;
            let transfers = channels.into_iter().filter(|c| { c.chain_1.port_id == "transfer" && c.chain_2.port_id == "transfer" }).collect::<Vec<_>>();
            let transfers_chain_1: HashMap<String, ChainChannelInfo> =
                transfers.iter().map(|transfer| {
                    let cci: ChainChannelInfo = ChainChannelInfo {
                        channel_src: transfer.chain_1.channel_id.clone(),
                        port_src: transfer.chain_1.port_id.clone(),
                        channel_dest: transfer.chain_2.channel_id.clone(),
                        port_dest: transfer.chain_2.port_id.clone(),
                        ordering: transfer.ordering.clone(),
                        version: transfer.version.clone(),
                    };
                    (channel_def.chain_2.chain_name.clone(), cci)
                }).collect();

            chains.entry(channel_def.chain_1.chain_name.clone()).and_modify(|e|
                {
                    for transfer in transfers_chain_1.iter() {
                        e.transfers.insert(channel_def.chain_2.chain_name.clone(), transfer.1.clone());
                    }
                }
            ).or_insert({

                Chain { chain_name: channel_def.chain_1.chain_name.to_string(), transfers: transfers_chain_1 }
            });
            let transfers_chain_2: HashMap<String, ChainChannelInfo> =
                transfers.iter().filter(|c| { c.chain_1.port_id == "transfer" && c.chain_2.port_id == "transfer" }).map(|transfer| {
                    let cci = ChainChannelInfo {
                        channel_dest: transfer.chain_1.channel_id.clone(),
                        port_dest: transfer.chain_1.port_id.clone(),
                        channel_src: transfer.chain_2.channel_id.clone(),
                        port_src: transfer.chain_2.port_id.clone(),
                        ordering: transfer.ordering.clone(),
                        version: transfer.version.clone(),
                    };

                    (channel_def.chain_1.chain_name.clone(), cci)
                }).collect();

            chains.entry(channel_def.chain_2.chain_name.clone()).and_modify(|e|
                {
                    for transfer in transfers_chain_2.iter() {
                        e.transfers.insert(channel_def.chain_1.chain_name.clone(), transfer.1.clone());
                    }
                }
            ).or_insert({

                Chain { chain_name: channel_def.chain_2.chain_name.to_string(), transfers: transfers_chain_2 }
            });
        }
    }
    fs::create_dir_all("/tmp/ibc")?;
    for chain in chains {
        let out_file = File::create(&format!("/tmp/ibc/{}.json", chain.0))?;
        serde_json::to_writer_pretty(out_file, &chain.1)?;
    }
    //   println!("Terra2 - {:?}", chains.get("terra2"));

    Ok(())
}
