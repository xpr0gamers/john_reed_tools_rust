use std::{
    future::Future,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use chrono::Datelike;

use crate::{app_state, fetch};

pub struct BackgroundWorker {
    pub app_state: app_state::AppState,
    pub john_reed_api: fetch::JohnReedApi,
}

impl BackgroundWorker {
    pub fn run_at_specific_time<F, Fut>(&self, start_time: u64, callback: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        thread::spawn(move || {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs();

            if start_time > now {
                let wait_time = start_time - now;
                thread::sleep(Duration::from_secs(wait_time));
            }

            callback();
        });
    }

    pub async fn book_courses(&self, user_id: String, course_id: i64) {
        println!("Book new courses")
    }

    /// Schedule new bookable courses for all users
    pub async fn schedule_book_courses(&self) {
        println!(
            "⏳Checking for new courses to book at: {:?}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        );

        let john_reed_api = fetch::JohnReedApi::new();

        for user in self.app_state.users.iter() {
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

                    let time_to_run = Instant::now() + Duration::from_secs(5);
                    thread::spawn(move || {
                        tokio::runtime::Runtime::new().unwrap().block_on(async {
                            let now = Instant::now();
                            let delay = time_to_run.saturating_duration_since(now);
                            if delay > Duration::ZERO {
                                thread::sleep(delay);
                            }

                            // self.book_courses(user.username.clone(), bookable_course.id.clone())
                            //     .await;
                        });
                    });

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
}
