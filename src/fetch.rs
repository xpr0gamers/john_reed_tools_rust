use std::sync::Arc;

use chrono::{DateTime, FixedOffset};
use reqwest::cookie::Jar;
use serde::{Deserialize, Serialize};

pub struct JohnReedApi {
    client: reqwest::Client,
}

impl JohnReedApi {
    pub fn new() -> Self {
        JohnReedApi {
            client: reqwest::Client::builder()
                .cookie_store(true)
                .cookie_provider(Arc::new(Jar::default()))
                .build()
                .unwrap(),
        }
    }

    pub async fn login(
        &self,
        payload: JohnReedLoginPayload,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .client
            .post("https://my.johnreed.fitness/login")
            .basic_auth(payload.username.to_owned(), payload.password.to_owned())
            .header("X-Nox-Client-Type", "WEB")
            .send()
            .await?;

        if !response.status().is_success() {
            return Result::Err("Unauthorized".into());
        }

        Result::Ok(())
    }

    pub async fn get_user(&self) -> Result<JohnReedMe, Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .client
            .get("https://my.johnreed.fitness/v1/me/info")
            .header("X-Nox-Client-Type", "WEB")
            .header(
                "X-Public-Facility-Group",
                "JOHNREED-65A11AB8FA704F88B2D8EF52523C576A",
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Result::Err("failed get user".into());
        }

        let john_reed_me: JohnReedMe = response.json().await?;
        Result::Ok(john_reed_me)
    }

    pub async fn get_home_studio(
        &self,
    ) -> Result<JohnReedHomeStudio, Box<dyn std::error::Error + Send + Sync>> {
        let response = self
            .client
            .get("https://my.johnreed.fitness/nox/v1/studios/home")
            .header("X-Nox-Client-Type", "WEB")
            .header(
                "X-Public-Facility-Group",
                "JOHNREED-65A11AB8FA704F88B2D8EF52523C576A",
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Result::Err("failed get home studio".into());
        }

        let john_reed_home_studio: JohnReedHomeStudio = response.json().await?;
        Result::Ok(john_reed_home_studio)
    }

    pub async fn get_bookable_courses(
        &self,
        params: BookableCoursesParams,
    ) -> Result<Vec<JohnReedCourse>, Box<dyn std::error::Error + Send + Sync>> {
        let mut url =
            "https://my.johnreed.fitness/nox/v2/bookableitems/courses/with-canceled?".to_string();
        url += &format!("startDate={}", params.start_date.format("%Y-%m-%d"));
        url += &params.end_date.map_or("".to_string(), |end_date| {
            format!("&endDate={}", end_date.format("%Y-%m-%d"))
        });
        url += &params
            .organization_unit_ids
            .map_or("".to_string(), |organization_unit_ids| {
                format!("&organizationUnitIds={}", organization_unit_ids)
            });

        let response = self
            .client
            .get(url)
            .header("X-Nox-Client-Type", "WEB")
            .header(
                "X-Public-Facility-Group",
                "JOHNREED-65A11AB8FA704F88B2D8EF52523C576A",
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Result::Err(
                format!(
                    "failed to get bookable courses with status {}",
                    response.status()
                )
                .into(),
            );
        }

        let john_reed_bookable_courses: Vec<JohnReedCourse> = response.json().await?;
        Result::Ok(john_reed_bookable_courses)
    }

    pub async fn book_course(
        &self,
        payload: JohnReedBookCorsePayload,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Booking course with payload: {:?}", payload);

        let response = self
            .client
            .post("https://my.johnreed.fitness/nox/v1/calendar/bookcourse")
            .json(&payload)
            .header("X-Nox-Client-Type", "WEB")
            .header(
                "X-Public-Facility-Group",
                "JOHNREED-65A11AB8FA704F88B2D8EF52523C576A",
            )
            .send()
            .await?;

        let response_status = response.status();
        if !response_status.is_success() {
            let body = response.text().await?;
            return Result::Err(
                format!(
                    "failed to book course with status {} and body {}",
                    response_status, body
                )
                .into(),
            );
        }

        let _: serde_json::Value = response.json().await?;
        Result::Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct JohnReedLoginPayload {
    pub username: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JohnReedBookCorsePayload {
    #[serde(rename = "courseAppointmentId")]
    pub course_appointment_id: i64,
    #[serde(rename = "expectedCustomerStatus")]
    pub expected_customer_status: String,
}

#[derive(Debug, Serialize)]
pub struct BookableCoursesParams {
    pub start_date: DateTime<FixedOffset>,
    pub end_date: Option<DateTime<FixedOffset>>,
    pub organization_unit_ids: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct JohnReedMe {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Deserialize, Debug)]
pub struct JohnReedHomeStudio {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct JohnReedCourse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "appointmentStatus")]
    pub appointment_status: String,
    pub slots: Vec<JohnReedSlot>,
}

impl JohnReedCourse {
    pub fn is_bookable(&self) -> bool {
        return self.appointment_status != "CANCELED";
    }
}

#[derive(Deserialize, Debug)]
pub struct JohnReedSlot {
    #[serde(rename = "startDateTime", with = "chrono_datetime_fixed_offset")]
    pub start_date_time: Option<DateTime<FixedOffset>>,
    #[serde(rename = "endDateTime", with = "chrono_datetime_fixed_offset")]
    pub end_date_time: Option<DateTime<FixedOffset>>,
    #[serde(rename = "alreadyBooked")]
    pub already_booked: bool,
    pub bookable: bool,
    #[serde(
        rename = "earliestBookingDateTime",
        with = "chrono_datetime_fixed_offset"
    )]
    pub earliest_booking_date_time: Option<DateTime<FixedOffset>>,
}

impl JohnReedSlot {
    pub fn is_bookable(&self) -> bool {
        if self.already_booked {
            return false;
        }
        if !self.bookable {
            return false;
        }
        if self.earliest_booking_date_time.is_none() {
            return false;
        }
        if self.start_date_time.unwrap().timestamp() < chrono::Local::now().timestamp() {
            return false;
        }
        return true;
    }
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
