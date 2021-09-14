use chrono::Utc;

#[derive(Debug)]
pub struct DateInfo {
    // origin_date_time: DateTime<Utc>,
    pub date_yyyymmdd: String,
    pub timestamp_iso8601: String,
    pub timestamp_rfc2822: String
}
impl DateInfo {
    pub fn now() -> DateInfo {
        let date_time = Utc::now();
        let timestamp = date_time.format("%Y%m%dT%H%M%SZ").to_string();
        let date = date_time.date().format("%Y%m%d").to_string();
        let rf2822 = date_time.to_rfc2822();
        DateInfo {
            // origin_date_time: date_time,
            date_yyyymmdd: date,
            timestamp_iso8601: timestamp,
            timestamp_rfc2822: rf2822
        }
    }
}
