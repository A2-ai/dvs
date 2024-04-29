// outcome of dvs operation e.g. get and add

// Outcome enum
#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Success,
    AlreadyPresent,
    Error,
    NotPresent,
    OutOfSync,
    UpToDate,
}

impl Outcome {
    pub fn outcome_to_string(&self) -> String {
        match self {
            Outcome::Success => String::from("Success"),
            Outcome::AlreadyPresent => String::from("Already Present"),
            Outcome::Error => String::from("Error"),
            Outcome::OutOfSync => String::from("out-of-sync"),
            Outcome::NotPresent => String::from("not-present"),
            Outcome::UpToDate => String::from("up-to-date")

        }
    }
}