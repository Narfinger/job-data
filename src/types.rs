use serde::{Deserialize, Serialize};
use yansi::{Paint, Painted};

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub(crate) enum Status {
    Todo,
    Pending,
    Rejected,
    Declined,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Todo => f.write_str("Todo"),
            Status::Pending => f.write_str("Pending"),
            Status::Rejected => f.write_str("Rejected"),
            Status::Declined => f.write_str("Declined"),
        }
    }
}

impl Status {
    pub(crate) fn print(&self) -> Painted<&str> {
        match self {
            Status::Todo => "TODO".red(),
            Status::Pending => "Pending".yellow(),
            Status::Declined => "Declined".green(),
            Status::Rejected => "Rejected".green(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Record {
    pub(crate) last_action_date: String,
    pub(crate) name: String,
    pub(crate) subname: String,
    pub(crate) stage: String,
    pub(crate) additional_info: String,
    pub(crate) status: Status,
}
