use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

/// A moment in UTC, kept to millisecond precision and written as RFC 3339,
/// e.g. `2026-04-30T19:00:00.123Z`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(OffsetDateTime);

impl Timestamp {
    /// `YYYY-MM-DD` in UTC.
    pub fn date(&self) -> String {
        let format = format_description!("[year]-[month]-[day]");
        self.0.format(&format).expect("a date always formats")
    }

    /// The current time, to the millisecond.
    pub fn now() -> Timestamp {
        Timestamp::utc_millis(OffsetDateTime::now_utc())
    }

    /// Converts to UTC and drops everything below a millisecond.
    fn utc_millis(moment: OffsetDateTime) -> Timestamp {
        let utc = moment.to_offset(time::UtcOffset::UTC);
        let millis = utc.nanosecond() / 1_000_000;
        Timestamp(
            utc.replace_nanosecond(millis * 1_000_000)
                .expect("milliseconds are a valid nanosecond count"),
        )
    }
}

impl Serialize for Timestamp {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl std::str::FromStr for Timestamp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed = OffsetDateTime::parse(s, &Rfc3339).map_err(|_| {
            format!("'{s}' is not an RFC 3339 timestamp, e.g. 2026-04-30T19:00:00.123Z")
        })?;
        Ok(Timestamp::utc_millis(parsed))
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
        );
        f.write_str(&self.0.format(&format).map_err(|_| std::fmt::Error)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    #[test]
    fn round_trips_as_rfc_3339_utc_with_milliseconds() {
        let text = "2026-04-30T19:00:00.123Z";
        assert_eq!(ts(text).to_string(), text);
    }

    #[test]
    fn rejects_malformed_text_and_old_epoch_seconds_naming_the_value() {
        for bad in ["yesterday", "", "1791504758", "2026-04-30"] {
            let err = bad.parse::<Timestamp>().unwrap_err();
            assert!(err.contains(&format!("'{bad}'")), "{err}");
            assert!(err.contains("RFC 3339"), "{err}");
        }
    }

    #[test]
    fn converts_to_utc_and_truncates_to_milliseconds() {
        assert_eq!(
            ts("2026-04-30T21:00:00.1239+02:00").to_string(),
            "2026-04-30T19:00:00.123Z"
        );
        assert_eq!(
            ts("2026-04-30T19:00:00Z").to_string(),
            "2026-04-30T19:00:00.000Z"
        );
    }

    #[test]
    fn date_is_the_utc_calendar_day_including_leap_days() {
        assert_eq!(ts("2024-02-29T23:59:59.999Z").date(), "2024-02-29");
        assert_eq!(ts("2024-03-01T00:00:00Z").date(), "2024-03-01");
    }

    #[test]
    fn serialises_as_a_plain_string() {
        let t = ts("2026-04-30T19:00:00.123Z");
        assert_eq!(
            serde_yaml::to_string(&t).unwrap(),
            "2026-04-30T19:00:00.123Z\n"
        );
        let back: Timestamp = serde_yaml::from_str("2026-04-30T19:00:00.123Z").unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn now_has_no_sub_millisecond_part() {
        assert_eq!(Timestamp::now().0.nanosecond() % 1_000_000, 0);
    }
}
