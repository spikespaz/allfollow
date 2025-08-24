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
