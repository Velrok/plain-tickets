use serde::{Deserialize, Serialize};

const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
/// 36^13 > 2^64, so 13 base36 digits hold any `u64`.
const WIDTH: usize = 13;

/// 64 random bits, stored and shown as 13 lowercase base36 characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ID(pub u64);

impl ID {
    /// A new random ID. Panics if the OS RNG is unavailable.
    pub fn generate() -> ID {
        ID(getrandom::u64().expect("OS random number generator unavailable"))
    }
}

impl std::str::FromStr for ID {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let valid = s.len() == WIDTH && s.bytes().all(|b| DIGITS.contains(&b));
        if !valid {
            return Err(format!(
                "'{s}' is not a {WIDTH}-character base36 ticket ID (0-9, a-z)"
            ));
        }
        u64::from_str_radix(s, 36)
            .map(ID)
            .map_err(|_| format!("'{s}' is too large to be a ticket ID"))
    }
}

impl std::fmt::Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut digits = [b'0'; WIDTH];
        let mut rest = self.0;
        for slot in digits.iter_mut().rev() {
            *slot = DIGITS[(rest % 36) as usize];
            rest /= 36;
        }
        f.write_str(std::str::from_utf8(&digits).expect("digits are ASCII"))
    }
}

impl Serialize for ID {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ID {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_ids_as_thirteen_character_base36() {
        assert_eq!(ID(171).to_string(), "000000000004r");
        assert_eq!(ID(u64::MAX).to_string(), "3w5e11264sgsf");
    }

    #[test]
    fn parses_only_thirteen_lowercase_base36_characters_that_fit_a_u64() {
        assert_eq!("000000000004r".parse::<ID>().unwrap(), ID(171));
        assert_eq!("3w5e11264sgsf".parse::<ID>().unwrap(), ID(u64::MAX));
        for bad in [
            "",
            "4r",
            "000000000004R",
            "0000000000004r0",
            "00000000000-4",
            "+00000000004r",
            "3w5e11264sgsg",
            "zzzzzzzzzzzzz",
        ] {
            assert!(bad.parse::<ID>().is_err(), "{bad}");
        }
    }

    #[test]
    fn serialises_as_the_base36_string() {
        assert_eq!(serde_yaml::to_string(&ID(171)).unwrap(), "000000000004r\n");
        let back: ID = serde_yaml::from_str("000000000004r").unwrap();
        assert_eq!(back, ID(171));
    }

    #[test]
    fn digit_only_ids_survive_a_yaml_round_trip() {
        let id = ID(1);
        let text = serde_yaml::to_string(&id).unwrap();
        assert_eq!(serde_yaml::from_str::<ID>(&text).unwrap(), id);
    }

    #[test]
    fn generated_ids_differ() {
        let ids: std::collections::HashSet<ID> = (0..1000).map(|_| ID::generate()).collect();
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn generated_ids_do_not_share_a_time_prefix() {
        let top: std::collections::HashSet<u64> = (0..64).map(|_| ID::generate().0 >> 48).collect();
        assert!(top.len() > 32, "{top:?}");
    }
}
