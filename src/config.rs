use actions::{get_input, get_multiline_input};

#[cfg(not(test))]
pub use imp::*;
#[cfg(test)]
pub use tests::*;

mod imp {
    use super::Config;

    pub fn config() -> &'static Config {
        use lazy_static::lazy_static;

        lazy_static! {
            static ref CONFIG: Config = Config::from_actions();
        }
        &CONFIG
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use lazy_static::lazy_static;

    use super::Config;

    lazy_static! {
        static ref CONFIG: Mutex<Config> = Mutex::new(Config::from_actions());
    }

    pub fn config() -> Config {
        CONFIG.lock().unwrap().clone()
    }

    pub fn replace_config(c: Config) {
        *CONFIG.lock().unwrap() = c;
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    // Specifies whether everyone is allowed to cherry pick.
    pub allow_all: bool,
    // Specifies whether to create an Issue when there is a PR conflict.
    pub create_issue_on_conflict: bool,
    // Specifies the label prefix for cherrypicker.
    pub label_prefix: String,
    // Specifies the label prefix after picked.
    pub picked_label_prefix: String,
    // Specifies the labels that need to be excluded when copying the labels of the original PR.
    pub exclude_labels: Vec<String>,
    // Specifies whether to copy the issue numbers from the squashed commit message.
    pub copy_issue_numbers_from_squashed_commit: bool,
}

impl Config {
    pub fn from_actions() -> Self {
        Self {
            allow_all: get_input("allow-all").unwrap_or(false),
            create_issue_on_conflict: get_input("create-issue-on-conflict").unwrap_or(false),
            label_prefix: get_input("label-prefix")
                .unwrap_or_else(|| "needs-cherry-pick/".to_string()),
            picked_label_prefix: get_input("picked-label-prefix")
                .unwrap_or_else(|| "cherry-picked/".to_string()),
            exclude_labels: get_multiline_input("exclude-labels"),
            copy_issue_numbers_from_squashed_commit: get_input(
                "copy-issue-numbers-from-squashed-commit",
            )
            .unwrap_or_default(),
        }
    }
}
