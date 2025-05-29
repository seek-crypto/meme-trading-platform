use chrono::{DateTime, Duration, Utc};

pub fn round_to_interval(timestamp: DateTime<Utc>, interval: &str) -> DateTime<Utc> {
    let duration = match interval {
        "1s" => Duration::seconds(1),
        "1m" => Duration::minutes(1),
        "5m" => Duration::minutes(5),
        "15m" => Duration::minutes(15),
        "1h" => Duration::hours(1),
        _ => Duration::minutes(1), // default
    };

    let seconds_since_epoch = timestamp.timestamp();
    let interval_seconds = duration.num_seconds();
    let rounded_seconds = (seconds_since_epoch / interval_seconds) * interval_seconds;
    
    DateTime::from_timestamp(rounded_seconds, 0).unwrap_or(timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_round_to_minute() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 34, 45).unwrap();
        let rounded = round_to_interval(timestamp, "1m");
        assert_eq!(rounded, Utc.with_ymd_and_hms(2024, 1, 1, 12, 34, 0).unwrap());
    }

    #[test]
    fn test_round_to_5_minutes() {
        let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 37, 45).unwrap();
        let rounded = round_to_interval(timestamp, "5m");
        assert_eq!(rounded, Utc.with_ymd_and_hms(2024, 1, 1, 12, 35, 0).unwrap());
    }
} 