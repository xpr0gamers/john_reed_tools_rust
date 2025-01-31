pub mod fetch;

use chrono::Datelike;
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
    let file = File::open("./settings.json").expect("Settings file not found");
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

            //check if course is bookable
            if !bookable_course.is_bookable() {
                continue;
            }

            for time_slot in course.time_slots {
                // Find a slot that matches the time slot
                let bookable_slot = bookable_course.slots.iter().find(|bookable_slot| {
                    let start_date_time = match bookable_slot.start_date_time {
                        Some(start_date_time) => start_date_time,
                        None => {
                            return false;
                        }
                    };
                    let end_date_time = match bookable_slot.end_date_time {
                        Some(end_date_time) => end_date_time,
                        None => {
                            return false;
                        }
                    };

                    let weekday_as_name = start_date_time.weekday().to_string();
                    if !weekday_as_name.eq_ignore_ascii_case(&time_slot.day.to_string()) {
                        return false;
                    }

                    if time_slot.start_time > start_date_time.time() {
                        return false;
                    }

                    if time_slot.end_time < end_date_time.time() {
                        return false;
                    }

                    return true;
                });

                let bookable_slot = match bookable_slot {
                    Some(bookable_slot) => bookable_slot,
                    None => {
                        continue;
                    }
                };

                //check if slot is bookable
                if !bookable_slot.is_bookable() {
                    continue;
                }

                //book course
                let book_course_result = john_reed_api
                    .book_course(fetch::JohnReedBookCorsePayload {
                        course_appointment_id: bookable_course.id,
                        expected_customer_status: "BOOKED".to_string(),
                    })
                    .await;
                match book_course_result {
                    Ok(_) => {
                        println!(
                            "✔Booked course: {} at: {}",
                            bookable_course.name,
                            bookable_slot
                                .start_date_time
                                .unwrap()
                                .format("%Y-%m-%d %H:%M:%S")
                        );
                    }
                    Err(e) => {
                        println!("❌Failed to book course: {}", bookable_course.name);
                        println!("{:?}", e);
                    }
                }
            }
        }
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
