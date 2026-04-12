use crate::event::{AgentEvent, AgentEventKind, EventAdapter};
use crate::tmux::PI_AGENT;
use serde_json::Value;

use super::{HookRegistration, json_str};

pub struct PiAdapter;

impl PiAdapter {
    /// Hook wiring for pi coding agent. pi uses a TypeScript extension
    /// (`~/.pi/agent/extensions/tmux-sidebar.ts`) to call hook.sh, mapping
    /// pi lifecycle events to the sidebar's event model.
    ///
    /// Supported events:
    /// - session_start → SessionStart
    /// - session_shutdown → SessionEnd
    /// - agent_start (user prompt) → UserPromptSubmit
    /// - agent_end → Stop
    /// - tool_call → ActivityLog
    pub const HOOK_REGISTRATIONS: &'static [HookRegistration] = &[
        HookRegistration {
            trigger: "SessionStart",
            matcher: None,
            kind: AgentEventKind::SessionStart,
        },
        HookRegistration {
            trigger: "SessionEnd",
            matcher: None,
            kind: AgentEventKind::SessionEnd,
        },
        HookRegistration {
            trigger: "UserPromptSubmit",
            matcher: None,
            kind: AgentEventKind::UserPromptSubmit,
        },
        HookRegistration {
            trigger: "Stop",
            matcher: None,
            kind: AgentEventKind::Stop,
        },
        HookRegistration {
            trigger: "PostToolUse",
            matcher: None,
            kind: AgentEventKind::ActivityLog,
        },
    ];
}

impl EventAdapter for PiAdapter {
    fn parse(&self, event_name: &str, input: &Value) -> Option<AgentEvent> {
        match event_name {
            "session-start" => Some(AgentEvent::SessionStart {
                agent: PI_AGENT.into(),
                cwd: json_str(input, "cwd").into(),
                permission_mode: json_str(input, "permission_mode").into(),
                source: json_str(input, "source").into(),
                worktree: None,
                agent_id: None,
                session_id: None,
            }),
            "session-end" => Some(AgentEvent::SessionEnd {
                end_reason: json_str(input, "end_reason").into(),
            }),
            "user-prompt-submit" => Some(AgentEvent::UserPromptSubmit {
                agent: PI_AGENT.into(),
                cwd: json_str(input, "cwd").into(),
                permission_mode: json_str(input, "permission_mode").into(),
                prompt: json_str(input, "prompt").into(),
                worktree: None,
                agent_id: None,
                session_id: None,
            }),
            "stop" => Some(AgentEvent::Stop {
                agent: PI_AGENT.into(),
                cwd: json_str(input, "cwd").into(),
                permission_mode: json_str(input, "permission_mode").into(),
                last_message: json_str(input, "last_assistant_message").into(),
                response: None,
                worktree: None,
                agent_id: None,
                session_id: None,
            }),
            "activity-log" => {
                let tool_name = json_str(input, "tool_name");
                if tool_name.is_empty() {
                    return None;
                }
                let tool_input = input.get("tool_input").cloned().unwrap_or(Value::Null);
                let tool_response = input.get("tool_response").cloned().unwrap_or(Value::Null);
                Some(AgentEvent::ActivityLog {
                    tool_name: tool_name.into(),
                    tool_input,
                    tool_response,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hook_registrations_match_parse_arms() {
        super::super::assert_table_drift_free("pi", PiAdapter::HOOK_REGISTRATIONS);
    }

    #[test]
    fn session_start() {
        let adapter = PiAdapter;
        let input = json!({"cwd": "/home/user"});
        let event = adapter.parse("session-start", &input).unwrap();
        assert_eq!(
            event,
            AgentEvent::SessionStart {
                agent: PI_AGENT.into(),
                cwd: "/home/user".into(),
                permission_mode: "".into(),
                source: "".into(),
                worktree: None,
                agent_id: None,
                session_id: None,
            }
        );
    }

    #[test]
    fn session_end() {
        let adapter = PiAdapter;
        assert_eq!(
            adapter.parse("session-end", &json!({})).unwrap(),
            AgentEvent::SessionEnd {
                end_reason: "".into(),
            }
        );
    }

    #[test]
    fn user_prompt_submit() {
        let adapter = PiAdapter;
        let input = json!({"cwd": "/tmp", "prompt": "fix bug"});
        let event = adapter.parse("user-prompt-submit", &input).unwrap();
        assert_eq!(
            event,
            AgentEvent::UserPromptSubmit {
                agent: PI_AGENT.into(),
                cwd: "/tmp".into(),
                permission_mode: "".into(),
                prompt: "fix bug".into(),
                worktree: None,
                agent_id: None,
                session_id: None,
            }
        );
    }

    #[test]
    fn stop() {
        let adapter = PiAdapter;
        let input = json!({"cwd": "/tmp", "last_assistant_message": "done"});
        let event = adapter.parse("stop", &input).unwrap();
        assert_eq!(
            event,
            AgentEvent::Stop {
                agent: PI_AGENT.into(),
                cwd: "/tmp".into(),
                permission_mode: "".into(),
                last_message: "done".into(),
                response: None,
                worktree: None,
                agent_id: None,
                session_id: None,
            }
        );
    }

    #[test]
    fn activity_log() {
        let adapter = PiAdapter;
        let input = json!({"tool_name": "Read", "tool_input": {"file_path": "/a/b.rs"}});
        let event = adapter.parse("activity-log", &input).unwrap();
        assert_eq!(
            event,
            AgentEvent::ActivityLog {
                tool_name: "Read".into(),
                tool_input: json!({"file_path": "/a/b.rs"}),
                tool_response: Value::Null,
            }
        );
    }

    #[test]
    fn activity_log_empty_tool_name_rejected() {
        assert!(PiAdapter.parse("activity-log", &json!({})).is_none());
    }

    #[test]
    fn unknown_event_ignored() {
        assert!(PiAdapter.parse("something-else", &json!({})).is_none());
    }

    #[test]
    fn notification_not_supported() {
        assert!(PiAdapter.parse("notification", &json!({})).is_none());
    }

    #[test]
    fn subagent_start_not_supported() {
        assert!(PiAdapter.parse("subagent-start", &json!({})).is_none());
    }

    #[test]
    fn subagent_stop_not_supported() {
        assert!(PiAdapter.parse("subagent-stop", &json!({})).is_none());
    }

    #[test]
    fn stop_failure_not_supported() {
        assert!(PiAdapter.parse("stop-failure", &json!({})).is_none());
    }

    #[test]
    fn permission_denied_not_supported() {
        assert!(PiAdapter.parse("permission-denied", &json!({})).is_none());
    }

    #[test]
    fn cwd_changed_not_supported() {
        assert!(PiAdapter.parse("cwd-changed", &json!({})).is_none());
    }

    #[test]
    fn task_created_not_supported() {
        assert!(PiAdapter.parse("task-created", &json!({})).is_none());
    }

    #[test]
    fn task_completed_not_supported() {
        assert!(PiAdapter.parse("task-completed", &json!({})).is_none());
    }

    #[test]
    fn teammate_idle_not_supported() {
        assert!(PiAdapter.parse("teammate-idle", &json!({})).is_none());
    }

    #[test]
    fn worktree_create_not_supported() {
        assert!(PiAdapter.parse("worktree-create", &json!({})).is_none());
    }

    #[test]
    fn worktree_remove_not_supported() {
        assert!(PiAdapter.parse("worktree-remove", &json!({})).is_none());
    }
}
