//! Time zones from jiff's bundled IANA database.

use std::sync::OnceLock;

use jiff::tz::{TimeZone, TimeZoneDatabase};

/// A time zone from jiff's bundled IANA database, never the host's.
///
/// Panics if the bundled database is empty (the `tzdb-bundle-always`
/// feature is off) or `name` is unknown, with jiff's error. Tests never
/// call `TimeZone::get`, which reads the host's tzdata first.
pub fn zone(name: &str) -> TimeZone {
    static DATABASE: OnceLock<TimeZoneDatabase> = OnceLock::new();
    let db = DATABASE.get_or_init(TimeZoneDatabase::bundled);
    assert!(
        !db.is_definitively_empty(),
        "jiff's bundled time zone database is empty: rustd3-testkit needs jiff's \
         `tzdb-bundle-always` feature"
    );
    db.get(name).unwrap_or_else(|e| panic!("{e}"))
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;
    use jiff::tz::Offset;

    use super::*;

    fn offset(name: &str, at: &str) -> Offset {
        zone(name).to_offset(at.parse::<Timestamp>().unwrap())
    }

    fn hours_minutes(hours: i32, minutes: i32) -> Offset {
        Offset::from_seconds(hours * 3600 + minutes * 60).unwrap()
    }

    #[test]
    fn los_angeles_springs_forward_on_8_march_2026() {
        assert_eq!(
            offset("America/Los_Angeles", "2026-03-08T09:59:59Z"),
            hours_minutes(-8, 0)
        );
        assert_eq!(
            offset("America/Los_Angeles", "2026-03-08T10:00:00Z"),
            hours_minutes(-7, 0)
        );
    }

    #[test]
    fn kathmandu_is_five_hours_forty_five_ahead() {
        assert_eq!(
            offset("Asia/Kathmandu", "2026-01-01T00:00:00Z"),
            hours_minutes(5, 45)
        );
    }

    #[test]
    fn berlin_falls_back_on_25_october_2026() {
        assert_eq!(
            offset("Europe/Berlin", "2026-10-25T00:59:59Z"),
            hours_minutes(2, 0)
        );
        assert_eq!(
            offset("Europe/Berlin", "2026-10-25T01:00:00Z"),
            hours_minutes(1, 0)
        );
    }

    #[test]
    #[should_panic(expected = "Mars/Olympus")]
    fn an_unknown_zone_panics_with_jiffs_error() {
        zone("Mars/Olympus");
    }
}
