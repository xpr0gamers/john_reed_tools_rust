pub mod fetch;
pub mod tasks;
pub mod utils;

use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() {
    let mut interval_30s = interval(Duration::from_secs(30));

    let app_state = match utils::get_app_state() {
        Ok(app_state) => app_state,
        Err(e) => {
            return eprintln!("Failed to get app state: {}", e);
        }
    };

    tokio::spawn(async move {
        loop {
            interval_30s.tick().await;
            tasks::schedule_book_courses(app_state.clone()).await;
        }
    });

    // Keep the main task alive
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await; // Sleep for an hour
    }
}

mod app_state {

    use chrono::{NaiveTime, Weekday};
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Clone)]
    pub struct AppState {
        pub users: Vec<User>,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct User {
        pub username: String,
        pub password: String,
        pub courses: Vec<CourseToBook>,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct CourseToBook {
        pub name: String,
        #[serde(rename = "timeSlots")]
        pub time_slots: Vec<TimeSlotToBook>,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct TimeSlotToBook {
        pub day: Weekday,
        #[serde(rename = "startTime")]
        pub start_time: NaiveTime,
        #[serde(rename = "endTime")]
        pub end_time: NaiveTime,
    }
}
