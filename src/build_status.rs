#[derive(Debug, PartialEq)]
pub struct TimeInfo {
    pub completed_at: String,
    pub duration_secs: u32
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Green,
    Red
}

#[derive(Debug, PartialEq)]
pub struct BuildStatus {
    pub status: Status, 
    pub url: String, 
    pub time_info: Option<TimeInfo>
}