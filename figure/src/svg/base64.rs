//! Base64 with the standard alphabet and `=` padding (RFC 4648 §4), for
//! the data URIs of embedded images.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `bytes` in base64.
pub(crate) fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let sextets = [18, 12, 6, 0].map(|shift| ALPHABET[((n >> shift) & 0x3f) as usize]);
        for (i, &c) in sextets.iter().enumerate() {
            out.push(if i <= chunk.len() { char::from(c) } else { '=' });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rfc_4648_test_vectors_encode() {
        let vectors = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (input, expected) in vectors {
            assert_eq!(encode(input.as_bytes()), expected, "{input:?}");
        }
    }

    #[test]
    fn every_sextet_maps_to_its_letter() {
        assert_eq!(encode(&[0xfb, 0xff, 0xbf]), "+/+/");
        assert_eq!(encode(&[0, 0, 0]), "AAAA");
    }
}
