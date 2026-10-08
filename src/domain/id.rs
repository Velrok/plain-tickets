use serde::{Deserialize, Serialize};

/// Stored in the front matter as 16 hex digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ID(pub u64);

impl ID {
    /// A random ID from the OS RNG. Panics if the OS RNG is unavailable.
    pub fn rand() -> ID {
        ID(getrandom::u64().expect("OS random number generator unavailable"))
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
    fn rand_ids_differ() {
        assert_ne!(ID::rand(), ID::rand());
    }
}
