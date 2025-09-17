/// Weekly temperature schedule
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Schedule([Vec<(u8, u8, f64)>; 7]);

impl Schedule {
    pub fn new() -> Schedule {
        Schedule([(); 7].map(|_| Vec::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule() {
        let mut schedule = Schedule::new();
        schedule.0[1].push((1, 30, 20.5));
        schedule.0[1].push((6, 10, 18.1));
    }
}
