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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString, IntoStaticStr)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum ParseError {
    MissingPrefix,
    UnknownPrefix { found: String },
    ExpectedPrefix { want: HashAlgo, found: HashAlgo },
    WrongLength { algo: HashAlgo, n_chars: usize },
    InvalidEncoding(DecodeError),
}

impl Hash {
    pub fn algorithm(&self) -> HashAlgo {
        self.algo
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.algo.size()]
    }

    pub fn to_string(&self, format: &HashFormat, show_algo: bool) -> String {
        // TODO: Use `String::with_capacity`, size should be predictable.
        // XREF: <https://git.lix.systems/lix-project/lix/src/commit/7b6a85982b3442e5371e5c248708fe41ebf2e1c8/lix/libutil/hash.hh>
        let mut buf = String::new();
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
            Ok(Self { algo, bytes })
        } else if !is_sri && hash.len() == BASE32NIX.encode_len(algo.size()) {
            let mut bytes = [0; MAX_HASH_SIZE];
            BASE32NIX.decode_mut(hash, &mut bytes[..algo.size()])?;
            Ok(Self { algo, bytes })
        } else if is_sri || hash.len() == BASE64.encode_len(algo.size()) {
            let mut buf = [0; MAX_HASH_SIZE + 2];
            BASE64.decode_mut(hash, &mut buf[..BASE64.decode_len(hash.len())?])?;
            let mut bytes = [0; MAX_HASH_SIZE];
            bytes.copy_from_slice(&buf[..MAX_HASH_SIZE]);
            Ok(Self { algo, bytes })
        } else {
            Err(ParseError::WrongLength {
                algo,
                n_chars: hash.len(),
            })
        }
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

impl From<DecodeError> for ParseError {
    fn from(other: DecodeError) -> Self {
        Self::InvalidEncoding(other)
    }
}

impl From<DecodePartial> for ParseError {
    fn from(other: DecodePartial) -> Self {
        other.error.into()
    }
}
