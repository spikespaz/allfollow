use data_encoding::{BASE64, DecodeError, DecodePartial, Encoding, HEXLOWER};
use data_encoding_macro::new_encoding;
use strum::{EnumString, IntoStaticStr};

const MAX_HASH_SIZE: usize = 64;
const HASH_TYPES_LIST: &str = "`blake3`, `md5`, `sha1`, `sha256`, or `sha512`";

// FIXME: Ensure that this matches the format of:
// <https://github.com/NixOS/nix/blob/c9211b0b2d52a26ed666780b763b39a5bddd3fb3/src/libutil/base-nix-32.cc>
pub const BASE32NIX: Encoding = new_encoding! {
    symbols: "0123456789abcdfghijklmnpqrsvwxyz",
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hash {
    algo: HashAlgo,
    bytes: [u8; MAX_HASH_SIZE],
    format: Option<HashFormat>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display, EnumString, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum HashAlgo {
    Blake3,
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashFormat {
    Base64,
    Nix32,
    Base16,
    Sri,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("hash does not specify a type, which is not otherwise known from context")]
    MissingPrefix,
    #[error("hash has an unknown prefix `{found}`, expected one of {HASH_TYPES_LIST}")]
    UnknownPrefix { found: String },
    #[error("attempted to parse a hash of type `{want}`, found `{found}` instead")]
    ExpectedPrefix { want: HashAlgo, found: HashAlgo },
    #[error("hash of type `{algo}` with length `{n_chars}` does not match any encoding")]
    WrongLength { algo: HashAlgo, n_chars: usize },
    #[error("decoded bytes are not a valid `{algo}` hash, expected {} bytes, found {n_bytes}", algo.size())]
    InvalidHash { algo: HashAlgo, n_bytes: usize },
    #[error("hash has an invalid encoding: {0}")]
    InvalidEncoding(#[from] DecodeError),
}

impl Hash {
    pub(crate) fn _new(algo: HashAlgo, bytes: [u8; MAX_HASH_SIZE], format: HashFormat) -> Self {
        Self {
            algo,
            bytes,
            format: Some(format),
        }
    }

    pub fn algorithm(&self) -> HashAlgo {
        self.algo
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.algo.size()]
    }

    pub fn format(&self) -> Option<HashFormat> {
        self.format
    }

    pub fn to_string(&self, format: &HashFormat, show_algo: bool) -> String {
        let mut buf = String::with_capacity(match format {
            HashFormat::Base64 | HashFormat::Sri => BASE64.encode_len(self.algo.size()),
            HashFormat::Nix32 => BASE32NIX.encode_len(self.algo.size()),
            HashFormat::Base16 => HEXLOWER.encode_len(self.algo.size()),
        });
        self.encode(format, show_algo, &mut buf).unwrap();
        buf
    }

    pub(crate) fn encode(
        &self,
        format: &HashFormat,
        show_algo: bool,
        mut buf: impl std::fmt::Write,
    ) -> std::fmt::Result {
        if matches!(format, HashFormat::Sri) {
            buf.write_fmt(format_args!("{}-", <&str>::from(self.algo)))?;
        } else if show_algo {
            buf.write_fmt(format_args!("{}:", <&str>::from(self.algo)))?;
        }
        match format {
            HashFormat::Base64 | HashFormat::Sri => BASE64.encode_write(self.bytes(), &mut buf)?,
            HashFormat::Nix32 => BASE32NIX.encode_write(self.bytes(), &mut buf)?,
            HashFormat::Base16 => HEXLOWER.encode_write(self.bytes(), &mut buf)?,
        }
        Ok(())
    }

    pub fn parse(input: &str) -> Result<Self, ParseError> {
        Self::parse_(input, None)
    }

    pub fn parse_as(input: &str, algo: HashAlgo) -> Result<Self, ParseError> {
        Self::parse_(input, Some(algo))
    }

    pub(crate) fn parse_(input: &str, algo: Option<HashAlgo>) -> Result<Self, ParseError> {
        let (algo_prefix, is_sri, hash) = Self::parse_prefix(input)?;
        let algo = match (algo, algo_prefix) {
            (None, None) => Err(ParseError::MissingPrefix),
            (Some(algo), None) | (None, Some(algo)) => Ok(algo),
            (Some(want), Some(found)) if want != found => {
                Err(ParseError::ExpectedPrefix { want, found })
            }
            (Some(algo), Some(_)) => Ok(algo),
        }?;
        Self::decode(hash, algo, is_sri)
    }

    pub(crate) fn parse_prefix(input: &str) -> Result<(Option<HashAlgo>, bool, &str), ParseError> {
        let (prefix, is_sri, hash);
        if let Some(pair) = input.split_once(':') {
            (prefix, hash) = (Some(pair.0), pair.1);
            is_sri = false;
        } else if let Some(pair) = input.split_once('-') {
            (prefix, hash) = (Some(pair.0), pair.1);
            is_sri = true;
        } else {
            (prefix, hash) = (None, input);
            is_sri = false;
        }
        let algo = prefix
            .map(|prefix| {
                prefix.parse().map_err(|_| ParseError::UnknownPrefix {
                    found: prefix.to_string(),
                })
            })
            .transpose()?;
        Ok((algo, is_sri, hash))
    }

    pub(crate) fn decode(hash: &str, algo: HashAlgo, is_sri: bool) -> Result<Self, ParseError> {
        let hash = hash.as_bytes();
        if !is_sri && hash.len() == HEXLOWER.encode_len(algo.size()) {
            let mut bytes = [0; MAX_HASH_SIZE];
            HEXLOWER.decode_mut(hash, &mut bytes[..algo.size()])?;
            Ok(Self::_new(algo, bytes, HashFormat::Base16))
        } else if !is_sri && hash.len() == BASE32NIX.encode_len(algo.size()) {
            let mut bytes = [0; MAX_HASH_SIZE];
            BASE32NIX.decode_mut(hash, &mut bytes[..algo.size()])?;
            Ok(Self::_new(algo, bytes, HashFormat::Nix32))
        } else if is_sri || hash.len() == BASE64.encode_len(algo.size()) {
            let mut buf = [0; MAX_HASH_SIZE + 2];
            let wrote = BASE64.decode_mut(hash, &mut buf[..BASE64.decode_len(hash.len())?])?;
            if wrote == algo.size() {
                let mut bytes = [0; MAX_HASH_SIZE];
                bytes[..wrote].copy_from_slice(&buf[..wrote]);
                let format = if is_sri {
                    HashFormat::Sri
                } else {
                    HashFormat::Base64
                };
                Ok(Self::_new(algo, bytes, format))
            } else {
                Err(ParseError::InvalidHash {
                    algo,
                    n_bytes: wrote,
                })
            }
        } else {
            Err(ParseError::WrongLength {
                algo,
                n_chars: hash.len(),
            })
        }
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.encode(&HashFormat::Sri, true, f)
    }
}

impl HashAlgo {
    pub const fn size(&self) -> usize {
        match self {
            HashAlgo::Blake3 => 32,
            HashAlgo::Md5 => 16,
            HashAlgo::Sha1 => 20,
            HashAlgo::Sha256 => 32,
            HashAlgo::Sha512 => 64,
        }
    }
}

impl From<DecodePartial> for ParseError {
    fn from(other: DecodePartial) -> Self {
        other.error.into()
    }
}

#[cfg(test)]
mod tests {
    use digest::Digest;
    use test_case::{test_case, test_matrix};

    use super::{Hash, HashAlgo, HashFormat, MAX_HASH_SIZE, ParseError};

    fn hash_string(s: &str, algo: HashAlgo) -> Hash {
        let mut bytes = [0; MAX_HASH_SIZE];
        let buf = &mut bytes[..algo.size()];
        match algo {
            HashAlgo::Blake3 => buf.copy_from_slice(blake3::Hasher::digest(s).as_slice()),
            HashAlgo::Md5 => buf.copy_from_slice(md5::Md5::digest(s).as_slice()),
            HashAlgo::Sha1 => buf.copy_from_slice(sha1::Sha1::digest(s).as_slice()),
            HashAlgo::Sha256 => buf.copy_from_slice(sha2::Sha256::digest(s).as_slice()),
            HashAlgo::Sha512 => buf.copy_from_slice(sha2::Sha512::digest(s).as_slice()),
        };
        Hash {
            algo,
            bytes,
            format: None,
        }
    }

    // values taken from: https://tools.ietf.org/html/rfc4634
    #[test_case(
        "abc", HashAlgo::Blake3
        => "blake3:6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
    )]
    #[test_case(
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", HashAlgo::Blake3
        => "blake3:c19012cc2aaf0dc3d8e5c45a1b79114d2df42abb2a410bf54be09e891af06ff8"
    )]
    // values taken from: https://www.ietf.org/archive/id/draft-aumasson-blake3-00.txt
    #[test_case(
        "IETF", HashAlgo::Blake3
        => "blake3:83a2de1ee6f4e6ab686889248f4ec0cf4cc5709446a682ffd1cbb4d6165181e2"
    )]
    // values taken from: https://tools.ietf.org/html/rfc1321
    #[test_case(
        "", HashAlgo::Md5
        => "md5:d41d8cd98f00b204e9800998ecf8427e"
    )]
    #[test_case(
        "abc", HashAlgo::Md5
        => "md5:900150983cd24fb0d6963f7d28e17f72"
    )]
    // values taken from: https://tools.ietf.org/html/rfc3174
    #[test_case(
        "abc", HashAlgo::Sha1
        => "sha1:a9993e364706816aba3e25717850c26c9cd0d89d"
    )]
    #[test_case(
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", HashAlgo::Sha1
        => "sha1:84983e441c3bd26ebaae4aa1f95129e5e54670f1"
    )]
    // values taken from: https://tools.ietf.org/html/rfc4634
    #[test_case(
        "abc", HashAlgo::Sha256
        => "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    )]
    #[test_case(
        "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", HashAlgo::Sha256
        => "sha256:248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    )]
    #[test_case(
        "abc", HashAlgo::Sha512
        => "sha512:ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
    )]
    #[test_case(
        "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu", HashAlgo::Sha512
        => "sha512:8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909"
    )]
    fn assert_known_hashes(s: &str, algo: HashAlgo) -> &str {
        Box::leak(Box::new(
            hash_string(s, algo).to_string(&HashFormat::Base16, true),
        ))
    }

    #[test_matrix(
        [HashAlgo::Blake3, HashAlgo::Md5, HashAlgo::Sha1, HashAlgo::Sha256, HashAlgo::Sha512],
        [HashFormat::Base16, HashFormat::Nix32, HashFormat::Base64, HashFormat::Sri],
        [true, false]
    )]
    fn roundtrip(algo: HashAlgo, format: HashFormat, show_algo: bool) {
        static S: &str = "Rust is okay, but C++ is a blight.";
        let hash = hash_string(S, algo);
        eprintln!("encoded = {hash}");
        let encoded = hash.to_string(&format, show_algo);
        let decoded = if show_algo {
            Hash::parse(&encoded).unwrap()
        } else {
            Hash::parse_as(&encoded, algo).unwrap()
        };
        eprintln!("decoded = {decoded}");
        assert_eq!(hash, decoded);
    }

    // MD5 (16 bytes): non-SRI cannot be too short by length-inference; but it
    // CAN be too long (18). SRI can be too short (15) or too long (18).
    #[test_case(
        "md5:AAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Md5, n_bytes: 18 }
        ; "MD5 non-SRI too long (18 bytes)"
    )]
    #[test_case(
        "md5-AAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Md5, n_bytes: 18 }
        ; "MD5 SRI too long (18 bytes)"
    )]
    #[test_case(
        "md5-AAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Md5, n_bytes: 15 }
        ; "MD5 SRI too short (15 bytes)"
    )]
    // SHA1 (20 bytes): non-SRI can be too short (19) or too long (21) since
    // base64 28 chars is ambiguous; SRI likewise.
    #[test_case(
        "sha1:AAAAAAAAAAAAAAAAAAAAAAAAAA=="
        => ParseError::InvalidHash { algo: HashAlgo::Sha1, n_bytes: 19 }
        ; "SHA1 non-SRI too short (19 bytes)"
    )]
    #[test_case(
        "sha1:AAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Sha1, n_bytes: 21 }
        ; "SHA1 non-SRI too long (21 bytes)"
    )]
    #[test_case(
        "sha1-AAAAAAAAAAAAAAAAAAAAAAAAAA=="
        => ParseError::InvalidHash { algo: HashAlgo::Sha1, n_bytes: 19 }
        ; "SHA1 SRI too short (19 bytes)"
    )]
    #[test_case(
        "sha1-AAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Sha1, n_bytes: 21 }
        ; "SHA1 SRI too long (21 bytes)"
    )]
    // SHA256 (32 bytes): 31 and 33 are both representable at 44 base64 chars.
    #[test_case(
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
        => ParseError::InvalidHash { algo: HashAlgo::Sha256, n_bytes: 31 }
        ; "SHA256 non-SRI too short (31 bytes)"
    )]
    #[test_case(
        "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Sha256, n_bytes: 33 }
        ; "SHA256 non-SRI too long (33 bytes)"
    )]
    #[test_case(
        "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
        => ParseError::InvalidHash { algo: HashAlgo::Sha256, n_bytes: 31 }
        ; "SHA256 SRI too short (31 bytes)"
    )]
    #[test_case(
        "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Sha256, n_bytes: 33 }
        ; "SHA256 SRI too long (33 bytes)"
    )]
    // BLAKE3 (32 bytes): same sizes as SHA256.
    #[test_case(
        "blake3:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
        => ParseError::InvalidHash { algo: HashAlgo::Blake3, n_bytes: 31 }
        ; "BLAKE3 non-SRI too short (31 bytes)"
    )]
    #[test_case(
        "blake3:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Blake3, n_bytes: 33 }
        ; "BLAKE3 non-SRI too long (33 bytes)"
    )]
    #[test_case(
        "blake3-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
        => ParseError::InvalidHash { algo: HashAlgo::Blake3, n_bytes: 31 }
        ; "BLAKE3 SRI too short (31 bytes)"
    )]
    #[test_case(
        "blake3-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Blake3, n_bytes: 33 }
        ; "BLAKE3 SRI too long (33 bytes)"
    )]
    // SHA512 (64 bytes): non-SRI cannot be too short by length-inference; but it
    // CAN be too long (66). SRI can be too short (63) or too long (66).
    #[test_case(
        "sha512:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Sha512, n_bytes: 66 }
        ; "SHA512 non-SRI too long (66 bytes)"
    )]
    #[test_case(
        "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Sha512, n_bytes: 66 }
        ; "SHA512 SRI too long (66 bytes)"
    )]
    #[test_case(
        "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        => ParseError::InvalidHash { algo: HashAlgo::Sha512, n_bytes: 63 }
        ; "SHA512 SRI too short (63 bytes)"
    )]
    fn invalid_hash(input: &str) -> ParseError {
        Hash::parse(input).unwrap_err()
    }
}
