/// Weekly temperature schedule
use std::usize;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Schedule([Vec<(u8, u8, f64)>; 7]);

impl Schedule {
    pub fn new() -> Schedule {
        Schedule([(); 7].map(|_| Vec::new()))
    }

    pub fn get_setpoint(&self, day: usize, hour: u8, minute: u8) -> f64 {
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
    use super::*;

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
}
