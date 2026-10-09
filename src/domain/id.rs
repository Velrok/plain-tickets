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

/// A leading part of a ticket ID, as typed on the command line. A full 13-character ID is one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdPrefix(String);

impl IdPrefix {
    /// The ID itself when this is a full ID, which needs no lookup.
    pub fn full(&self) -> Option<ID> {
        self.0.parse().ok()
    }

    pub fn matches(&self, id: ID) -> bool {
        id.to_string().starts_with(&self.0)
    }
}

impl std::str::FromStr for IdPrefix {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let valid = (1..=WIDTH).contains(&s.len()) && s.bytes().all(|b| DIGITS.contains(&b));
        if !valid {
            return Err(format!(
                "'{s}' is not a ticket ID or ID prefix (1 to {WIDTH} characters, 0-9 a-z)"
            ));
        }
        if s.len() == WIDTH {
            s.parse::<ID>()?;
        }
        Ok(IdPrefix(s.to_string()))
    }
}

impl std::fmt::Display for IdPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
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

/// The fewest characters of an ID that pretty output shows.
const MIN_SHORT: usize = 3;

/// Works out how much of an ID is needed to tell it apart from every other known ID.
pub struct IdAbbrev {
    /// Every known ID as text, sorted. Empty with `shorten` off.
    known: Vec<String>,
    shorten: bool,
}

impl IdAbbrev {
    pub fn new(ids: &[ID]) -> IdAbbrev {
        let mut known: Vec<String> = ids.iter().map(ID::to_string).collect();
        known.sort();
        IdAbbrev {
            known,
            shorten: true,
        }
    }

    /// Treats every ID as needing all its characters.
    #[cfg(test)]
    pub fn full() -> IdAbbrev {
        IdAbbrev {
            known: Vec::new(),
            shorten: false,
        }
    }

    /// The length of the shortest prefix of `id` that no other known ID shares, at least
    /// `MIN_SHORT`. A full ID when abbreviation is off.
    pub fn unique_len(&self, id: ID) -> usize {
        let text = id.to_string();
        if !self.shorten {
            return WIDTH;
        }
        let at = self.known.partition_point(|k| k.as_str() < text.as_str());
        let shared = |other: &String| {
            text.bytes()
                .zip(other.bytes())
                .take_while(|(a, b)| a == b)
                .count()
        };
        let before = at.checked_sub(1).map(|i| shared(&self.known[i]));
        // A known copy of this very ID sits at `at`; compare with the one after it.
        let after_at = if self.known.get(at) == Some(&text) {
            at + 1
        } else {
            at
        };
        let after = self.known.get(after_at).map(shared);
        let needed = before.max(after).map_or(0, |n| n + 1);
        needed.clamp(MIN_SHORT, WIDTH)
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

    fn abbrev(known: &[&str]) -> IdAbbrev {
        let ids: Vec<ID> = known.iter().map(|s| s.parse().unwrap()).collect();
        IdAbbrev::new(&ids)
    }

    #[test]
    fn needs_the_shortest_unique_prefix_of_at_least_three_characters() {
        let a = abbrev(&["0abc000000001", "0abd000000002", "0xyz000000003"]);
        let len = |s: &str| a.unique_len(s.parse().unwrap());
        assert_eq!(len("0xyz000000003"), 3);
        assert_eq!(len("0abc000000001"), 4);
        assert_eq!(len("0abd000000002"), 4);
    }

    #[test]
    fn identical_up_to_the_last_character_needs_the_full_id() {
        let a = abbrev(&["0abc000000001", "0abc000000002"]);
        assert_eq!(a.unique_len("0abc000000002".parse().unwrap()), 13);
    }

    #[test]
    fn an_unknown_id_is_shortened_against_the_known_ones() {
        let a = abbrev(&["0abc000000001"]);
        assert_eq!(a.unique_len("0abd000000009".parse().unwrap()), 4);
        assert_eq!(a.unique_len("0xyz000000003".parse().unwrap()), 3);
    }

    #[test]
    fn full_needs_every_character() {
        assert_eq!(IdAbbrev::full().unique_len(ID(171)), 13);
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
