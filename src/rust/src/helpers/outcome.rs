// outcome of dvs operation e.g. get and add

// Outcome enum
#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Copied,
    Present,
    Error,
}

// Outcome enum
#[derive(Clone, PartialEq, Debug)]
pub enum Status {
    Absent,
    Unsynced,
    Current,
    Error,
}

impl Outcome {
    pub fn outcome_to_string(&self) -> String {
        match self {
            Outcome::Copied => String::from("copied"),
            Outcome::Present => String::from("present"),
            Outcome::Error => String::from("error")
        }
    }
}

impl Status {
    pub fn outcome_to_string(&self) -> String {
        match self {
            Status::Absent => String::from("absent"),
            Status::Unsynced => String::from("unsynced"),
            Status::Current => String::from("current"),
            Status::Error => String::from("error"),
        }
    }
}

