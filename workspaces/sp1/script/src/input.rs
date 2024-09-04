use crate::utils::{get_light_blocks, verify_blocks};

mod utils;

pub fn input() {
    let (light_block_1, light_block_2) = get_light_blocks();

    let expected_verdict = verify_blocks(light_block_1.clone(), light_block_2.clone());

    let encoded_1 = serde_cbor::to_vec(&light_block_1).unwrap();
    let encoded_2 = serde_cbor::to_vec(&light_block_2).unwrap();

    zk_rust_io::write(&encoded_1);
    zk_rust_io::write(&encoded_2);
}
