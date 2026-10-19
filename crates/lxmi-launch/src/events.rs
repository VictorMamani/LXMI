//! One JSON object per helper stdout line (`tools/lxmi-mod-loader/loader.c`).
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelperEvent {
    HookInstalled {
        code: i64,
    },
    Armed {
        timeout_seconds: u64,
    },
    /// Direct Inject: `Inject` returned for the bound game process (0 = success).
    InjectReturned {
        code: i64,
    },
    Unhooked {
        code: i64,
    },
    GameObserved {
        mods_module_present: bool,
        process_age_ms: u64,
    },
    GameAlive {
        observation_ms: u64,
    },
    GameExited {
        exit_code_hex: String,
    },
    /// Any other status, e.g. `game_not_started` or `loader_load_failed`.
    Failure {
        status: String,
        windows_error: Option<u64>,
    },
}

impl HelperEvent {
    pub fn parse(line: &str) -> Option<Self> {
        let value: Value = serde_json::from_str(line.trim()).ok()?;
        let status = value.get("status")?.as_str()?.to_string();
        let int = |key: &str| value.get(key).and_then(Value::as_i64);
        let uint = |key: &str| value.get(key).and_then(Value::as_u64);
        Some(match status.as_str() {
            "hook_library_returned" => Self::HookInstalled { code: int("code")? },
            "armed_hook_waiting_for_new_game" | "armed_inject_waiting_for_new_game" => {
                Self::Armed {
                    timeout_seconds: uint("timeout_seconds")?,
                }
            }
            "unhook_library_returned" => Self::Unhooked { code: int("code")? },
            "inject_returned" => Self::InjectReturned { code: int("code")? },
            "game_observed" => Self::GameObserved {
                mods_module_present: int("mods_module_present")? == 1,
                process_age_ms: uint("process_age_ms").unwrap_or(0),
            },
            "game_alive_after_load" => Self::GameAlive {
                observation_ms: uint("observation_ms")?,
            },
            "game_exited_after_load" => Self::GameExited {
                exit_code_hex: value.get("exit_code_hex")?.as_str()?.to_string(),
            },
            _ => Self::Failure {
                status,
                windows_error: uint("windows_error"),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_direct_inject_protocol() {
        assert_eq!(
            HelperEvent::parse(
                r#"{"status":"armed_inject_waiting_for_new_game","timeout_seconds":300}"#
            ),
            Some(HelperEvent::Armed {
                timeout_seconds: 300
            })
        );
        assert_eq!(
            HelperEvent::parse(r#"{"status":"inject_returned","code":600}"#),
            Some(HelperEvent::InjectReturned { code: 600 })
        );
    }

    #[test]
    fn parses_the_helper_protocol() {
        assert_eq!(
            HelperEvent::parse(r#"{"status":"hook_library_returned","code":0}"#),
            Some(HelperEvent::HookInstalled { code: 0 })
        );
        assert_eq!(
            HelperEvent::parse(
                r#"{"status":"game_observed","windows_pid":1868,"mods_module_present":1,"process_age_ms":3210}"#
            ),
            Some(HelperEvent::GameObserved {
                mods_module_present: true,
                process_age_ms: 3210
            })
        );
        assert_eq!(
            HelperEvent::parse(
                r#"{"status":"game_exited_after_load","exit_code":3221225781,"exit_code_hex":"0xc0000135"}"#
            ),
            Some(HelperEvent::GameExited {
                exit_code_hex: "0xc0000135".into()
            })
        );
        assert_eq!(
            HelperEvent::parse(r#"{"status":"game_not_started","windows_error":0}"#),
            Some(HelperEvent::Failure {
                status: "game_not_started".into(),
                windows_error: Some(0)
            })
        );
        assert_eq!(HelperEvent::parse("Proton: Upgrading prefix"), None);
        assert_eq!(
            HelperEvent::parse(r#"{"status":"hook_library_returned"}"#),
            None
        );
    }
}
