use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct FuzzyDate {
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
}

impl FuzzyDate {
    pub fn new() -> Self {
        Self {
            year: None,
            month: None,
            day: None,
        }
    }

    pub fn year(mut self, year: u32) -> Self {
        self.year = Some(year);
        self
    }

    pub fn month(mut self, month: u32) -> Self {
        self.month = Some(month);
        self
    }

    pub fn day(mut self, day: u32) -> Self {
        self.day = Some(day);
        self
    }
}

impl Display for FuzzyDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let year = self.year.unwrap_or(0);
        let month = self.month.unwrap_or(0);
        let day = self.day.unwrap_or(0);
        // year should be 4 digits, month and day should be 2 digits
        write!(f, "{:04}-{:02}-{:02}", year, month, day)
    }
}

impl From<String> for FuzzyDate {
    fn from(date: String) -> Self {
        // split YYYYMMDD into year, month, day
        let year = date[0..4].parse().unwrap_or(0);
        let month = date[4..6].parse().unwrap_or(0);
        let day = date[6..8].parse().unwrap_or(0);
        Self {
            year: Some(year),
            month: Some(month),
            day: Some(day),
        }
    }
}

impl From<&str> for FuzzyDate {
    fn from(date: &str) -> Self {
        // split YYYYMMDD into year, month, day
        let year = date[0..4].parse().unwrap_or(0);
        let month = date[4..6].parse().unwrap_or(0);
        let day = date[6..8].parse().unwrap_or(0);
        Self {
            year: Some(year),
            month: Some(month),
            day: Some(day),
        }
    }
}

impl From<u32> for FuzzyDate {
    fn from(date: u32) -> Self {
        // split YYYYMMDD into year, month, day
        let year = date / 10000;
        let month = (date % 10000) / 100;
        let day = date % 100;
        Self {
            year: Some(year),
            month: Some(month),
            day: Some(day),
        }
    }
}
