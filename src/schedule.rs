/// Weekly temperature schedule
use std::usize;

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

// Define the enum to represent the two possible JSON formats for the schedule
#[derive(Deserialize)]
#[serde(untagged)]
enum ScheduleInput {
    Full([Vec<(u8, u8, f32)>; 7]), // Full 7-day schedule
    Single(Vec<(u8, u8, f32)>),    // Single day to replicate
}

#[derive(Serialize, Debug, PartialEq)]
pub struct Schedule([Vec<(u8, u8, f32)>; 7]);

// Custom Deserialize implementation for Schedule
impl<'de> Deserialize<'de> for Schedule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into ScheduleInput enum
        match ScheduleInput::deserialize(deserializer)? {
            ScheduleInput::Full(days) => Ok(Schedule(days)),
            ScheduleInput::Single(day) => {
                // Replicate the single day across all 7 days
                Ok(Schedule([
                    day.clone(),
                    day.clone(),
                    day.clone(),
                    day.clone(),
                    day.clone(),
                    day.clone(),
                    day,
                ]))
            }
        }
    }
}

impl Schedule {
    pub fn new() -> Schedule {
        Schedule([(); 7].map(|_| Vec::new()))
    }

    pub fn get_setpoint(&self, day: usize, hour: u8, minute: u8) -> f32 {
        let yesterday: usize = (day + 6) % 7;
        let mut temp = self.0[yesterday].last().unwrap().2;

        for (h, m, t) in &self.0[day] {
            if hour >= *h && minute >= *m {
                temp = *t;
            };
        }

        temp
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;

    use super::*;

    #[test]
    fn test_get_setpoint() {
        let mut schedule = Schedule::new();
        schedule.0[0].push((3, 30, 1.5));
        schedule.0[1].push((1, 30, 2.5));
        schedule.0[1].push((3, 50, 30.75));
        schedule.0[1].push((6, 10, 18.1));
        schedule.0[6].push((3, 23, 10.4));

        // Wrap to the previous day
        assert_eq!(schedule.get_setpoint(1, 1, 25), 1.5);

        // First setpoint of the day
        assert_eq!(schedule.get_setpoint(1, 1, 30), 2.5);
        assert_eq!(schedule.get_setpoint(1, 3, 49), 2.5);
        assert_eq!(schedule.get_setpoint(1, 3, 50), 30.75);
        assert_eq!(schedule.get_setpoint(1, 6, 13), 18.1);

        // Wrap around to the end of the week
        assert_eq!(schedule.get_setpoint(0, 2, 13), 10.4);
    }

    #[test]
    fn test_serialise_full() {
        let full_json = r#"
        [
            [[9, 30, 21.4], [10, 0, 17.0], [21, 30, 21.4]],
            [[9, 30, 21.4], [10, 0, 17.0], [21, 30, 21.4]],
            [[9, 30, 21.4], [10, 0, 17.0], [21, 30, 21.4]],
            [[9, 30, 21.4], [10, 0, 17.0], [21, 30, 21.4]],
            [[9, 30, 21.4], [10, 0, 17.0], [21, 30, 21.4]],
            [[9, 30, 21.4], [10, 0, 17.0], [21, 30, 21.4]],
            [[9, 30, 21.4], [10, 0, 17.0], [21, 30, 21.4]]
        ]
        "#;

        let _schedule: Schedule = from_str(full_json).unwrap();
    }

    #[test]
    fn test_serialise_single_day() {
        let full_json = r#"[[9, 30, 21.4], [10, 0, 17.0], [21, 30, 21.4]]"#;

        let _schedule: Schedule = from_str(full_json).unwrap();
    }
}
