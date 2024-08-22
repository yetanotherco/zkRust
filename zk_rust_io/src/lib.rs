use serde::{de::DeserializeOwned, Serialize};

//TODO: How to handle read case
#[inline(never)]
pub fn read<T: DeserializeOwned>() -> T { unimplemented!("ZK Rust funcion `io::commit()` is a compile time symbol" ) }
#[inline(never)]
pub fn commit<T: Serialize>(_value: &T) { println!("ZK Rust function `io::commit()` is a compile time symbol") }
#[inline(never)]
pub fn write(_buf: &[u8]) { println!("ZK Rust function `io::write() is a compile time symbol") }
#[inline(never)]
pub fn out() { println!("Zk Rust function `io::out()` is a compile time symbol") }
