use k256::{
    ecdsa::{signature::{Signer, Verifier}, Signature, SigningKey},
};
use rand_core::OsRng;

fn main() {
    // Generate a random secp256k1 keypair and sign the message.
    let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
    let message = b"This is a message that will be signed, and verified within the zkVM";
    let signature: Signature = signing_key.sign(message);

    let verifying_key = signing_key.verifying_key();

    // Verify the signature, panicking if verification fails.
    verifying_key
        .verify(message, &signature)
        .expect("ECDSA signature verification failed");
}