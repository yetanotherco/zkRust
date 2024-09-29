use crate::utils::{get_light_blocks, verify_blocks};
use tendermint_light_client_verifier::{
    options::Options, types::LightBlock, ProdVerifier, Verdict, Verifier,
};
mod utils;
use zk_rust_io;

pub fn main() {
    println!("cycle-tracker-start: io");
    println!("cycle-tracker-start: reading bytes");
    let encoded_1: Vec<u8> = zk_rust_io::read();
    let encoded_2: Vec<u8> = zk_rust_io::read();
    println!("cycle-tracker-end: reading bytes");
    println!("first 10 bytes: {:?}", &encoded_1[..10]);
    println!("first 10 bytes: {:?}", &encoded_2[..10]);

    println!("cycle-tracker-start: serde");
    let light_block_1: LightBlock = serde_cbor::from_slice(&encoded_1).unwrap();
    let light_block_2: LightBlock = serde_cbor::from_slice(&encoded_2).unwrap();
    println!("cycle-tracker-end: serde");
    println!("cycle-tracker-end: io");

    println!(
        "LightBlock1 number of validators: {}",
        light_block_1.validators.validators().len()
    );
    println!(
        "LightBlock2 number of validators: {}",
        light_block_2.validators.validators().len()
    );

    println!("cycle-tracker-start: header hash");
    let header_hash_1 = light_block_1.signed_header.header.hash();
    let header_hash_2 = light_block_2.signed_header.header.hash();
    println!("cycle-tracker-end: header hash");

    println!("cycle-tracker-start: public input headers");
    zk_rust_io::commit(&header_hash_1.as_bytes());
    zk_rust_io::commit(&header_hash_2.as_bytes());
    println!("cycle-tracker-end: public input headers");

    println!("cycle-tracker-start: verify");
    let vp = ProdVerifier::default();
    let opt = Options {
        trust_threshold: Default::default(),
        trusting_period: std::time::Duration::from_secs(500),
        clock_drift: Default::default(),
    };
    let verify_time = light_block_2.time() + std::time::Duration::from_secs(20);
    let verdict = vp.verify_update_header(
        light_block_2.as_untrusted_state(),
        light_block_1.as_trusted_state(),
        &opt,
        verify_time.unwrap(),
    );
    println!("cycle-tracker-end: verify");

    println!("cycle-tracker-start: public inputs verdict");
    let verdict_encoded = serde_cbor::to_vec(&verdict).unwrap();
    zk_rust_io::commit(&verdict_encoded.as_slice());
    println!("cycle-tracker-end: public inputs verdict");

    match verdict {
        Verdict::Success => {
            println!("success");
        }
        v => panic!("expected success, got: {:?}", v),
    }
}

pub fn input() {
    let (light_block_1, light_block_2) = get_light_blocks();

    let expected_verdict = verify_blocks(light_block_1.clone(), light_block_2.clone());

    let encoded_1 = serde_cbor::to_vec(&light_block_1).unwrap();
    let encoded_2 = serde_cbor::to_vec(&light_block_2).unwrap();

    zk_rust_io::write(&encoded_1);
    zk_rust_io::write(&encoded_2);
}

pub fn output() {
    let (light_block_1, light_block_2) = get_light_blocks();
    // Verify the public values
    let mut expected_public_values: Vec<u8> = Vec::new();
    expected_public_values.extend(light_block_1.signed_header.header.hash().as_bytes());
    expected_public_values.extend(light_block_2.signed_header.header.hash().as_bytes());
    expected_public_values.extend(serde_cbor::to_vec(&expected_verdict).unwrap());

    let public_inputs: Vec<u8> = zk_rust_io::out();

    assert_eq!(public_inputs.as_ref(), expected_public_values);
}
