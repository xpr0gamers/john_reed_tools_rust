use std::time::Duration;

use chrono::Datelike;
use tokio::time::sleep_until;

use crate::{
    app_state, fetch,
    utils::{datetime_to_instant, instant_to_datetime},
};

pub async fn book_courses(app_state: app_state::AppState, user_name: String, course_id: i64) {
    println!(
        "📅Try to book course at: {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    let user = app_state
        .users
        .iter()
        .find(|user| user.username == user_name);

    let user = match user {
        Some(user) => user,
        None => {
            return;
        }
    };

    let john_reed_api = fetch::JohnReedApi::new();

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
            return;
        }
    }

    //book course
    let book_course_result = john_reed_api
        .book_course(fetch::JohnReedBookCorsePayload {
            course_appointment_id: course_id,
            expected_customer_status: "BOOKED".to_string(),
        })
        .await;
    match book_course_result {
        Ok(_) => {
            println!("✔Success");
        }
        Err(e) => {
            println!("❌Failed to book for course id: {}", course_id);
            println!("{:?}", e);
        }
    }
}

/// Schedule new bookable courses for all users
pub async fn schedule_book_courses(app_state: app_state::AppState) {
    println!(
        "⏳Checking for new courses to book at: {:?}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    );

    let john_reed_api = fetch::JohnReedApi::new();

    for user in app_state.users.iter() {
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

        for course in user.courses.iter() {
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

            for time_slot in course.time_slots.iter() {
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

                let earliest_booking_date_time = bookable_slot.earliest_booking_date_time.unwrap();
                let app_state_clone = app_state.clone();
                let user_name = user.username.clone();
                let course_id = bookable_course.id.clone();

                let start_time_as_instant = datetime_to_instant(earliest_booking_date_time);
                println!(
                    "📅 New task for booking course is planned for: {} at: {}",
                    bookable_course.name,
                    instant_to_datetime(start_time_as_instant).format("%Y-%m-%d %H:%M:%S")
                );

                tokio::spawn(async move {
                    sleep_until(start_time_as_instant).await;
                    book_courses(app_state_clone, user_name, course_id).await;
                });
            }
        }
    }
}
