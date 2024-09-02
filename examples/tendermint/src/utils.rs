#![allow(dead_code)]
use std::error::Error;

use std::time::Duration;
use std::{fs::File, io::Read};

use tendermint_light_client_verifier::{options::Options, types::LightBlock};
use tendermint_light_client_verifier::{ProdVerifier, Verdict, Verifier};

pub fn load_light_block(block_height: u64) -> Result<LightBlock, Box<dyn Error>> {
    let mut file = File::open(&format!("src/files/block_{}.json", block_height))?;
    let mut block_response_raw = String::new();
    file.read_to_string(&mut block_response_raw)
        .expect(&format!("Failed to read block number {}", block_height));
    Ok(serde_json::from_str(&block_response_raw)?)
}

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
    let light_block_1 = load_light_block(2279100).expect("Failed to generate light block 1");
    let light_block_2 = load_light_block(2279130).expect("Failed to generate light block 2");
    (light_block_1, light_block_2)
}
