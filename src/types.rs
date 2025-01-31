use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct JohnReedCourse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub slots: Vec<Slot>,
}

#[derive(Deserialize, Debug)]
pub struct Slot {
    #[serde(with = "chrono_datetime_fixed_offset")]
    pub startDateTime: Option<DateTime<FixedOffset>>,
    pub alreadyBooked: bool,
    pub bookable: bool,
    #[serde(with = "chrono_datetime_fixed_offset")]
    pub earliestBookingDateTime: Option<DateTime<FixedOffset>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookCorsePayload {
    pub courseAppointmentId: i64,
    pub expectedCustomerStatus: String,
}

pub mod chrono_datetime_fixed_offset {
    use chrono::{DateTime, FixedOffset};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deserialize_option = Option::<String>::deserialize(deserializer)?;
        if deserialize_option.is_none() {
            return Ok(None);
        }
        let date_string = deserialize_option.unwrap();

        // Split into ISO date-time and region
        let (iso_part, _region_part) =
            date_string.split_at(date_string.find('[').ok_or_else(|| {
                serde::de::Error::custom("Expected region in format [Region/Zone]")
            })?);

        // Parse ISO date-time
        let datetime = DateTime::parse_from_rfc3339(iso_part)
            .map_err(|err| serde::de::Error::custom(format!("Invalid datetime: {}", err)))?;

        return Result::Ok(Some(datetime));
    }
}
