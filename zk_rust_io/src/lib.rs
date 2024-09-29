use serde::{de::DeserializeOwned, Serialize};

#[inline(never)]
pub fn read<T: DeserializeOwned + Default>() -> T {
    T::default()
}
#[inline(never)]
pub fn commit<T: Serialize>(_value: &T) {}
#[inline(never)]
pub fn write<T: Serialize>(_buf: &T) {}
#[inline(never)]
pub fn out() {}
