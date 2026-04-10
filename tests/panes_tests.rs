#[allow(dead_code, unused_imports)]
mod test_helpers;

use test_helpers::*;
use tmux_agent_sidebar::state::Focus;
use tmux_agent_sidebar::tmux::{AgentType, PaneStatus, SessionInfo, WindowInfo};

// ─── Agents: auto-scroll behavior Tests ─────────────────────────────

#[test]
fn test_agents_auto_scroll_keeps_selected_visible() {
    // Create enough agents to overflow a small viewport
    let mut panes = Vec::new();
    for i in 0..10 {
        let mut pane = make_pane(AgentType::Claude, PaneStatus::Idle);
        pane.pane_id = format!("%{}", i);
        panes.push(pane);
    }

    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: panes.clone(),
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", panes)];
    state.sidebar_focused = true;
    state.focus = Focus::Panes;
    state.rebuild_row_targets();

    // Render with a small height (only 6 lines visible for agents, 20 for bottom)
    // Total height = 26, bottom = 20, agents = 6
    let _ = render_to_string(&mut state, 28, 26);
    assert_eq!(state.panes_scroll.offset, 0, "initially at top");

    // Select last agent and re-render
    state.global.selected_pane_row = 9;
    let _ = render_to_string(&mut state, 28, 26);
    assert!(
        state.panes_scroll.offset > 0,
        "should scroll down to show selected agent"
    );
}

#[test]
fn test_panes_scroll_offset_tracks_total_and_visible() {
    let pane = make_pane(AgentType::Claude, PaneStatus::Idle);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", vec![pane])];
    state.rebuild_row_targets();

    let _ = render_to_string(&mut state, 28, 26);
    // After rendering, panes_scroll.total_lines and panes_scroll.visible_height should be set
    assert!(
        state.panes_scroll.total_lines > 0,
        "total lines should be populated"
    );
    assert!(
        state.panes_scroll.visible_height > 0,
        "visible height should be populated"
    );
}

// ─── Agents: Codex agent color ──────────────────────────────────────

#[test]
fn snapshot_codex_agent_styled() {
    let pane = make_pane(AgentType::Codex, PaneStatus::Idle);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", vec![pane])];
    state.rebuild_row_targets();
    state.sidebar_focused = false; // so colors show, not REVERSED

    let output = render_to_styled_string(&mut state, 28, 24);
    // Codex agent should use agent_codex color (141)
    assert!(
        output.contains("fg:141"),
        "Codex agent should use codex color (141)"
    );
}

// ─── Agents: Unknown agent type ─────────────────────────────────────

#[test]
fn snapshot_unknown_agent_styled() {
    let pane = make_pane(AgentType::Unknown, PaneStatus::Idle);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", vec![pane])];
    state.rebuild_row_targets();
    state.sidebar_focused = false;

    let output = render_to_styled_string(&mut state, 28, 24);
    // Unknown agent uses status_unknown color (244)
    assert!(
        output.contains("fg:244"),
        "Unknown agent should use unknown color (244)"
    );
}

// ─── Agents: running icon variants via render ───────────────────────

#[test]
fn test_running_icon_blink_off() {
    let pane = make_pane(AgentType::Claude, PaneStatus::Running);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", vec![pane])];
    state.rebuild_row_targets();
    state.sidebar_focused = false;
    state.spinner_frame = 0;

    let output = render_to_string(&mut state, 28, 24);
    assert!(output.contains("●"), "spinner frame 0 should show ●");
}

#[test]
fn test_running_spinner_frame_advances() {
    let pane = make_pane(AgentType::Claude, PaneStatus::Running);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", vec![pane])];
    state.rebuild_row_targets();
    state.sidebar_focused = false;
    state.spinner_frame = 3;

    let output = render_to_string(&mut state, 28, 24);
    assert!(output.contains("●"), "spinner frame 3 should show ●");
}

#[test]
fn test_waiting_icon() {
    let pane = make_pane(AgentType::Claude, PaneStatus::Waiting);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", vec![pane])];
    state.rebuild_row_targets();
    state.sidebar_focused = false;

    let output = render_to_string(&mut state, 28, 24);
    assert!(output.contains("◐"), "waiting pane should show ◐ icon");
}

#[test]
fn test_error_icon() {
    let pane = make_pane(AgentType::Claude, PaneStatus::Error);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", vec![pane])];
    state.rebuild_row_targets();
    state.sidebar_focused = false;

    let output = render_to_string(&mut state, 28, 24);
    assert!(output.contains("✕"), "error pane should show ✕ icon");
}

#[test]
fn test_unknown_status_icon() {
    let pane = make_pane(AgentType::Claude, PaneStatus::Unknown);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", vec![pane])];
    state.rebuild_row_targets();
    state.sidebar_focused = false;

    let output = render_to_string(&mut state, 28, 24);
    assert!(output.contains("·"), "unknown status should show · icon");
}

// ─── Agents: auto-scroll includes trailing border ──────────────────

#[test]
fn test_agents_auto_scroll_includes_group_bottom_border() {
    // When the last agent in a group is selected, the auto-scroll
    // should include the group's bottom border line (╰───╯) so it
    // is not clipped off the viewport.
    let mut panes = Vec::new();
    for i in 0..6 {
        let mut pane = make_pane(AgentType::Claude, PaneStatus::Idle);
        pane.pane_id = format!("%{}", i);
        panes.push(pane);
    }

    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: panes.clone(),
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", panes)];
    state.sidebar_focused = true;
    state.focus = Focus::Panes;
    state.rebuild_row_targets();

    // Select the last agent
    state.global.selected_pane_row = 5;
    // Use a tight height so agents area is small (height - 1 margin - 20 bottom)
    let output = render_to_string(&mut state, 28, 26);

    // The output should contain the bottom border of the group
    assert!(
        output.contains("╰"),
        "group bottom border should be visible when last agent is selected"
    );
}

#[test]
fn test_agents_auto_scroll_up_shows_group_header() {
    // After scrolling down, selecting the first agent should scroll
    // back up enough to show the group header.
    let mut panes = Vec::new();
    for i in 0..8 {
        let mut pane = make_pane(AgentType::Claude, PaneStatus::Idle);
        pane.pane_id = format!("%{}", i);
        panes.push(pane);
    }

    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "project".into(),
            window_active: true,
            auto_rename: false,
            panes: panes.clone(),
        }],
    }]);
    state.repo_groups = vec![make_repo_group("project", panes)];
    state.sidebar_focused = true;
    state.focus = Focus::Panes;
    state.rebuild_row_targets();

    // Scroll to bottom
    state.global.selected_pane_row = 7;
    let _ = render_to_string(&mut state, 28, 26);
    assert!(state.panes_scroll.offset > 0, "should have scrolled down");

    // Now select first agent and re-render
    state.global.selected_pane_row = 0;
    let output = render_to_string(&mut state, 28, 26);

    // The group header should be visible
    assert!(
        output.contains("╭ project"),
        "group header should be visible when first agent is selected"
    );
}

// ─── Repo popup rendering ───────────────────────────────────────────

#[test]
fn repo_popup_renders_repo_names_when_open() {
    let pane = make_pane(AgentType::Claude, PaneStatus::Idle);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "frontend".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![
        make_repo_group("frontend", vec![pane.clone()]),
        make_repo_group("backend", vec![pane.clone()]),
    ];
    state.rebuild_row_targets();
    state.repo_popup_open = true;

    let output = render_to_string(&mut state, 40, 30);
    assert!(output.contains("All"), "popup should list 'All' entry");
    assert!(
        output.contains("frontend"),
        "popup should list frontend repo"
    );
    assert!(output.contains("backend"), "popup should list backend repo");
    assert!(
        state.repo_popup_area.is_some(),
        "render should populate repo_popup_area for hit-testing"
    );
}

#[test]
fn repo_popup_highlights_selected_entry_with_background() {
    let pane = make_pane(AgentType::Claude, PaneStatus::Idle);
    let mut state = make_state(vec![SessionInfo {
        session_name: "main".into(),
        windows: vec![WindowInfo {
            window_id: "@1".into(),
            window_name: "frontend".into(),
            window_active: true,
            auto_rename: false,
            panes: vec![pane.clone()],
        }],
    }]);
    state.repo_groups = vec![
        make_repo_group("frontend", vec![pane.clone()]),
        make_repo_group("backend", vec![pane.clone()]),
    ];
    state.rebuild_row_targets();
    state.sidebar_focused = false; // surface raw colors instead of REVERSED
    state.repo_popup_open = true;
    state.repo_popup_selected = 2; // "backend" (0=All, 1=frontend, 2=backend)

    let styled = render_to_styled_string(&mut state, 40, 30);
    // The highlighted row should carry the selection background.
    // render_to_styled_string interleaves style annotations between glyphs, so
    // "backend" never appears as a contiguous substring — match on the styled
    // bytes of each character ("b[fg:...,bg:237,bold]") to detect the selected
    // row precisely.
    let theme = &state.theme;
    let bg_idx = match theme.selection_bg {
        ratatui::style::Color::Indexed(n) => n,
        _ => panic!("selection_bg should be an indexed color in the default theme"),
    };
    let bg_marker = format!("bg:{bg_idx}");
    let selected_line = styled
        .lines()
        .find(|l| {
            l.contains(&format!("b[fg:255,{bg_marker},bold]"))
                && l.contains(&format!("d[fg:255,{bg_marker},bold]"))
        })
        .expect("popup should render 'backend' with selection_bg and bold");
    assert!(selected_line.contains("bold"));
}
