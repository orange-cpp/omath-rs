//! Reversible in-memory obfuscation. This is not cryptographic encryption.
use crate::rev_eng::MemoryValue;
use std::{
    marker::PhantomData,
    mem::{MaybeUninit, size_of},
    ops::{Deref, DerefMut},
};

pub struct EncryptedVariable<T: MemoryValue, const N: usize> {
    bytes: Vec<u8>,
    key: [u8; N],
    encrypted: bool,
    marker: PhantomData<T>,
}
impl<T: MemoryValue, const N: usize> EncryptedVariable<T, N> {
    pub fn new(value: T, key: [u8; N]) -> Self {
        assert!(N > 0, "an encryption key must contain at least one byte");
        let bytes =
            unsafe { std::slice::from_raw_parts((&value as *const T).cast(), size_of::<T>()) }
                .to_vec();
        let mut result = Self {
            bytes,
            key,
            encrypted: false,
            marker: PhantomData,
        };
        result.encrypt();
        result
    }
    fn xor(&mut self) {
        for (i, byte) in self.bytes.iter_mut().enumerate() {
            *byte ^= self.key[i % N].wrapping_add(i.wrapping_mul(N) as u8);
        }
    }
    pub fn encrypt(&mut self) {
        if !self.encrypted {
            self.xor();
            self.encrypted = true;
        }
    }
    pub fn decrypt(&mut self) {
        if self.encrypted {
            self.xor();
            self.encrypted = false;
        }
    }
    pub fn is_encrypted(&self) -> bool {
        self.encrypted
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    /// Returns the current representation, encrypted or plaintext, as in C++.
    pub fn value(&self) -> T {
        let mut value = MaybeUninit::<T>::uninit();
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.bytes.as_ptr(),
                value.as_mut_ptr().cast(),
                self.bytes.len(),
            );
            value.assume_init()
        }
    }
    /// Decrypts until the guard drops; mutations are written back and re-encrypted even on unwinding.
    pub fn drop_anchor(&mut self) -> VarAnchor<'_, T, N> {
        self.decrypt();
        let value = self.value();
        VarAnchor { owner: self, value }
    }
}
pub struct VarAnchor<'a, T: MemoryValue, const N: usize> {
    owner: &'a mut EncryptedVariable<T, N>,
    value: T,
}
impl<T: MemoryValue, const N: usize> Deref for VarAnchor<'_, T, N> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T: MemoryValue, const N: usize> DerefMut for VarAnchor<'_, T, N> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
impl<T: MemoryValue, const N: usize> Drop for VarAnchor<'_, T, N> {
    fn drop(&mut self) {
        let bytes =
            unsafe { std::slice::from_raw_parts((&self.value as *const T).cast(), size_of::<T>()) };
        self.owner.bytes.copy_from_slice(bytes);
        self.owner.encrypt();
    }
}
pub const fn random_key<const N: usize>(mut seed: u64) -> [u8; N] {
    let mut key = [0; N];
    let mut i = 0;
    while i < N {
        seed = seed.wrapping_add(0x9E3779B97F4A7C15);
        let mut x = seed;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
        key[i] = (x ^ (x >> 31)) as u8;
        i += 1;
    }
    key
}
#[macro_export]
macro_rules! encrypted_variable {
    ($value:expr, $size:expr) => {
        $crate::containers::EncryptedVariable::new(
            $value,
            $crate::containers::random_key::<$size>((line!() as u64) << 32 | column!() as u64),
        )
    };
}
