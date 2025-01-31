pub mod fetch;

use std::{error::Error, fs::File, io::BufReader, time::Duration};
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut interval = interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        check_for_new_courses().await;
    }
}

async fn check_for_new_courses() {
    let file = File::open("settings.json").expect("Settings file not found");
    let app_state: app_state::AppState = serde_json::from_reader(BufReader::new(file)).unwrap();

    println!(
        "⏳Checking for new courses to book at: {:?}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    );

    let john_reed_api = fetch::JohnReedApi::new();

    for user in app_state.users {
        let login_result = john_reed_api
            .login(fetch::JohnReedLoginPayload {
                username: user.username.clone(),
                password: Some(user.password.clone()),
            })
            .await;
        match login_result {
            Ok(_) => println!("Logged in successfully as: {}", user.username),
            Err(e) => {
                println!("Failed to login as: {}", user.username);
                println!("{:?}", e);
                continue;
            }
        }

        let home_studio_result = john_reed_api.get_home_studio().await;
        let home_studio = match home_studio_result {
            Ok(home_studio) => {
                println!("Home studio: {}", home_studio.name);
                home_studio
            }
            Err(e) => {
                println!("Failed to get home studio for: {}", user.username);
                println!("{:?}", e);
                continue;
            }
        };

        let bookable_courses_params = fetch::BookableCoursesParams {
            start_date: chrono::Local::now().fixed_offset(),
            end_date: Some(
                (chrono::Local::now() + Duration::from_secs(60 * 60 * 24 * 7)).fixed_offset(),
            ),
            organization_unit_ids: Some(home_studio.id),
        };
        let bookable_courses_result = john_reed_api
            .get_bookable_courses(bookable_courses_params)
            .await;
        let bookable_courses = match bookable_courses_result {
            Ok(bookable_courses) => bookable_courses,
            Err(e) => {
                println!("Failed to get bookable courses for: {}", user.username);
                println!("{:?}", e);
                continue;
            }
        };

        for course in user.courses {
            let bookable_course = bookable_courses
                .iter()
                .find(|bookable_course| bookable_course.name == course.name);
            let bookable_course = match bookable_course {
                Some(bookable_course) => bookable_course,
                None => {
                    continue;
                }
            };

            if !bookable_course.is_bookable() {
                continue;
            }

            for time_slot in course.time_slots {
                println!(
                    "Checking for course: {} on {} from {}",
                    course.name, time_slot.start_time, time_slot.end_time
                );
            }
        }
    }
}

mod app_state {

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
        pub day: Day,
        #[serde(rename = "startTime")]
        pub start_time: String,
        #[serde(rename = "endTime")]
        pub end_time: String,
    }

    #[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
    pub enum Day {
        Sunday,
        Monday,
        Tuesday,
        Wednesday,
        Thursday,
        Friday,
        Saturday,
    }
}
