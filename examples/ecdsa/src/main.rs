use k256::{
    ecdsa::{
        signature::{Signer, Verifier},
        Signature, SigningKey, VerifyingKey,
    },
    EncodedPoint,
};
use rand_core::OsRng;

fn main() {
    // Generate a random secp256k1 keypair and sign the message.
    let encoded_verifying_key: EncodedPoint = zk_rust_io::read(); // Serialize with `::to_bytes()`
    let message: Vec<u8> = zk_rust_io::read();
    let signature: Signature = zk_rust_io::read();

    let verifying_key = VerifyingKey::from_encoded_point(&encoded_verifying_key).unwrap();

    // Verify the signature, panicking if verification fails.
    verifying_key
        .verify(&message, &signature)
        .expect("ECDSA signature verification failed");

    zk_rust_io::commit(&(encoded_verifying_key, message));
}

fn input() {
    let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
    let message = b"This is a message that will be signed, and verified within the zkVM".to_vec();
    let signature: Signature = signing_key.sign(&message);
    let vk = signing_key.verifying_key().to_encoded_point(true);
    zk_rust_io::write(&vk);
    zk_rust_io::write(&message);
    zk_rust_io::write(&signature);
}

fn output() {
    let (receipt_verifying_key, receipt_message): (EncodedPoint, Vec<u8>) = zk_rust_io::out();

    println!(
        "Verified the signature over message {:?} with key {}",
        std::str::from_utf8(&receipt_message[..]).unwrap(),
        receipt_verifying_key,
    );
}
