// outcome of dvs operation e.g. get and add

// Outcome enum
#[derive(Clone, PartialEq, Debug)]
pub enum Outcome {
    Success,
    AlreadyPresent,
    Error,
}

impl Outcome {
    pub fn outcome_to_string(&self) -> String {
        match self {
            Outcome::Success => String::from("Success"),
            Outcome::AlreadyPresent => String::from("Already Present"),
            Outcome::Error => String::from("Error"),

        }
    }
}