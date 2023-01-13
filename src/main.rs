use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::{BufReader, LineWriter, Write};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use clap::Parser;
use error_chain::error_chain;

use crate::chain::{Chain, ChainChannelInfo};
use crate::ibc::IbcData;

mod ibc;
mod chain;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        SystemTimeError(std::time::SystemTimeError);
        SerdeError(serde_json::Error);
    }
}

/// Program to transform ibc chain files into a single file per chain
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory
    #[arg(short, long, default_value = "./chain-registry/_IBC")]
    in_directory: String,

    /// Output director
    #[arg(short, long, default_value = "/tmp/IBC")]
    out_directory: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let current_dir = PathBuf::from(args.in_directory);

    let mut chains: BTreeMap<String, Chain> = BTreeMap::new();

    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::metadata(&path)?;
        if metadata.is_file() {
            let reader = BufReader::new(File::open(path)?);
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
    fs::create_dir_all(&args.out_directory)?;
    let now: DateTime<Utc> = Utc::now();
    let out_index_file = File::create(format!("{}/index.ts", &args.out_directory))?;
    let mut out_line = LineWriter::new(out_index_file);

    out_line.write(format!("// automatically generated: {}\n\n", now.to_rfc3339()).as_bytes())?;


    for chain in &chains {
        out_line.write(format!("import {} from './{}.json';\n", &chain.0, &chain.0).as_bytes())?;
        let out_file = File::create(format!("{}/{}.json", &args.out_directory, chain.0))?;
        serde_json::to_writer_pretty(out_file, &chain.1)?;
    }
    out_line.write("\nexport interface ChainDeets {
  chain_name: string;
  transfers: {
    [chainId: string]: {
      channel_src: string;
      port_src: string;
      channel_dest: string;
      port_dest: string;
      ordering: string;
      version: string;
    };
  };
}\n".as_bytes())?;
    out_line.write(format!("export const ibcData: {{ [chainId:string]: ChainDeets }} = {{\n").as_bytes())?;
    for chain in &chains {
        out_line.write(format!("\t'{}':{},\n", &chain.0, &chain.0).as_bytes())?;
    }
    out_line.write(format!("}};\n").as_bytes())?;

    println!(
        "{} chain entries created in {:?}:", chains.len(),
        args.out_directory
    );

    Ok(())
}
