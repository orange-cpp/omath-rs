use omath::hashing::{self, Base64Error};

#[test]
fn fnv1a() {
    const HASH: u64 = hashing::fnv1a(b"hello");

    assert_eq!(HASH, 0xA430D84680AABD0Bu64);
}

#[test]
fn crc32() {
    const CRC: u32 = hashing::crc32(b"123456789");

    assert_eq!(CRC, 0xCBF43926u32);
}

#[test]
fn base64_encode() {
    assert_eq!(hashing::base64_encode(b"foo"), "Zm9v");
    assert_eq!(hashing::base64_encode(b"foobar"), "Zm9vYmFy");
}

#[test]
fn base64_decode() {
    assert!(hashing::base64_decode("Zm9v").is_ok());
    assert_eq!(hashing::base64_decode("Zm9v").unwrap(), b"foo");
    assert_eq!(hashing::base64_decode("Zm9vYmFy").unwrap(), b"foobar");
    assert_eq!(
        hashing::base64_decode("Zm9v=").unwrap_err(),
        Base64Error::InvalidLength
    );
    assert_eq!(
        hashing::base64_decode("Zm?v").unwrap_err(),
        Base64Error::InvalidCharacter
    );
    assert_eq!(
        hashing::base64_decode("Zm9=").unwrap_err(),
        Base64Error::InvalidPadding
    );
}
