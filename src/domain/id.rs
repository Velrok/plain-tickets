use serde::{Deserialize, Serialize};

/// Stored in the front matter as 16 hex digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ID(pub u64);

impl ID {
    /// A random ID from the OS RNG. Panics if the OS RNG is unavailable.
    pub fn rand() -> ID {
        ID(getrandom::u64().expect("OS random number generator unavailable"))
    }
}

impl std::str::FromStr for ID {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 16 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!("'{s}' is not a 16-digit hex ticket ID"));
        }
        u64::from_str_radix(s, 16)
            .map(ID)
            .map_err(|e| e.to_string())
    }
}

impl std::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl Serialize for ID {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{:016x}", self.0))
    }
}

impl<'de> Deserialize<'de> for ID {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        u64::from_str_radix(&s, 16)
            .map(ID)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_full_sixteen_digit_hex() {
        assert_eq!("00000000000000ab".parse::<ID>().unwrap(), ID(0xab));
        for bad in [
            "ab",
            "",
            "000000000000000g",
            "00000000000000abc",
            "+00000000000000a",
        ] {
            assert!(bad.parse::<ID>().is_err(), "{bad}");
        }
    }

    #[test]
    fn rand_ids_differ() {
        assert_ne!(ID::rand(), ID::rand());
    }
}
