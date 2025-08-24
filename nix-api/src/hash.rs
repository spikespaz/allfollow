use data_encoding::{BASE64, Encoding, HEXLOWER};
use data_encoding_macro::new_encoding;
use strum::{EnumString, IntoStaticStr};

const MAX_HASH_SIZE: usize = 64;

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
