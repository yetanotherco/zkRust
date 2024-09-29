#![allow(dead_code)]
use std::error::Error;

use std::time::Duration;
use std::{fs::File, io::Read};

use tendermint_light_client_verifier::{options::Options, types::LightBlock};
use tendermint_light_client_verifier::{ProdVerifier, Verdict, Verifier};

pub const BLOCK_2279100: &[u8] = include_bytes!("./files/block_2279100.json");
pub const BLOCK_2279130: &[u8] = include_bytes!("./files/block_2279130.json");

pub fn verify_blocks(light_block_1: LightBlock, light_block_2: LightBlock) -> Verdict {
    let vp = ProdVerifier::default();
    let opt = Options {
        trust_threshold: Default::default(),
        trusting_period: Duration::from_secs(500),
        clock_drift: Default::default(),
    };
    let verify_time = light_block_2.time() + Duration::from_secs(20);
    vp.verify_update_header(
        light_block_2.as_untrusted_state(),
        light_block_1.as_trusted_state(),
        &opt,
        verify_time.unwrap(),
    )
}

pub fn get_light_blocks() -> (LightBlock, LightBlock) {
    //let light_block_1 = load_light_block(2279100).expect("Failed to generate light block 1");
    let light_block_1 = serde_json::from_slice(&BLOCK_2279100).unwrap();
    //let light_block_2 = load_light_block(2279130).expect("Failed to generate light block 2");
    let light_block_2 = serde_json::from_slice(&BLOCK_2279130).unwrap();
    (light_block_1, light_block_2)
}
