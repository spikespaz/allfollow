use strum::EnumString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumString)]
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
