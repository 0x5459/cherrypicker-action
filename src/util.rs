use itertools::Itertools;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

use crate::config::config;

pub fn match_cherry_pick_command(text: impl AsRef<str>) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = RegexBuilder::new(r"^(?:/cherrypick|/cherry-pick)\s+(.+)$")
            .multi_line(true)
            .build()
            .unwrap();
    }

    RE.captures_iter(text.as_ref())
        .filter_map(|caps| caps.get(1).map(|x| x.as_str().trim().to_string()))
        .unique()
        .collect()
}

pub fn match_label(text: impl AsRef<str>) -> Option<String> {
    let label_prefix = &config().label_prefix;
    if label_prefix.is_empty() {
        return None;
    }
    text.as_ref()
        .strip_prefix(label_prefix)
        .map(|x| x.trim().to_string())
}

pub fn is_cherry_pick_invite_command(text: impl AsRef<str>) -> bool {
    lazy_static! {
        static ref RE: Regex = RegexBuilder::new(r"^(?:/cherrypick|/cherry-pick)-invite\b")
            .multi_line(true)
            .build()
            .unwrap();
    }

    return RE.is_match(text.as_ref());
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;

    use crate::config::{replace_config, Config};

    use super::*;

    #[test]
    fn test_match_cherry_pick_command() {
        let cases = vec![
            ("/cherrypick xx", vec!["xx"]),
            ("/cherry-pick xx", vec!["xx"]),
            ("/cherrypickxxx", vec![]),
            ("/cherry-pickxxx", vec![]),
            (
                r#"
/cherry-pick r
xxxx
/cherry-pick    releasev0.3
/cherrypick releasev0.3
/cherrypick release/v0.5
/cherrypick release/v0.5😊
        "#,
                vec!["r", "releasev0.3", "release/v0.5", "release/v0.5😊"],
            ),
        ];

        for (cmd, expected) in cases {
            assert_eq!(match_cherry_pick_command(cmd), expected, "cmd: {}", cmd);
        }
    }

    #[test]
    fn test_match_label() {
        let cases = vec![
            (
                "needs-cherry-pick-",
                "needs-cherry-pick-lbwnb",
                Some("lbwnb"),
            ),
            ("lbw", "lbwnb", Some("nb")),
            ("lbwnb", "", None),
            ("", "lbwnb", None),
            ("", "needs-cherry-pick-lbw", None),
        ];

        for (label_prefix, label, expected) in cases {
            let mut c = Config::from_actions();
            c.label_prefix = label_prefix.to_string();
            replace_config(c);

            assert_eq!(
                match_label(label).as_deref(),
                expected,
                "label_prefix: {}, label: {}",
                label_prefix,
                label
            );
        }
    }

    #[test]
    fn test_is_cherry_pick_invite_command() {
        let cases = vec![
            ("lbwnb", false),
            ("/cherrypick-", false),
            ("/cherry-pick", false),
            ("/cherrypick-invite", true),
            ("/cherry-pick-invite", true),
            ("/cherrypick-invitexx", false),
            ("/cherrypick-invite lbw", true),
            ("/cherrypick-invite_lbw", false),
        ];

        for (cmd, expected) in cases {
            assert_eq!(is_cherry_pick_invite_command(cmd), expected, "cmd: {}", cmd);
        }
    }
}
