use zk_rust_io;

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
