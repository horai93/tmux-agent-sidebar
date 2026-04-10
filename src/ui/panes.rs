mod row;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::state::{AppState, Focus, RepoFilter, StatusFilter};
use crate::tmux::PaneStatus;

use super::text::{display_width, pad_to, truncate_to_width};

/// Render the filter bar. Returns (Line, repo_button_col).
fn render_filter_bar<'a>(state: &AppState, bar_width: u16) -> (Line<'a>, u16) {
    let theme = &state.theme;
    let icons = &state.icons;
    let (all, running, waiting, idle, error) = state.status_counts();

    let items: Vec<(StatusFilter, Option<(&str, ratatui::style::Color)>, usize)> = vec![
        (StatusFilter::All, None, all),
        (
            StatusFilter::Running,
            Some((
                icons.status_icon(&PaneStatus::Running),
                theme.status_running,
            )),
            running,
        ),
        (
            StatusFilter::Waiting,
            Some((
                icons.status_icon(&PaneStatus::Waiting),
                theme.status_waiting,
            )),
            waiting,
        ),
        (
            StatusFilter::Idle,
            Some((icons.status_icon(&PaneStatus::Idle), theme.status_idle)),
            idle,
        ),
        (
            StatusFilter::Error,
            Some((icons.status_icon(&PaneStatus::Error), theme.status_error)),
            error,
        ),
    ];

    let mut spans: Vec<Span<'a>> = Vec::new();
    spans.push(Span::raw(" "));
    let mut current_width: usize = 1;

    let selected_style = |style: Style| {
        style
            .underline_color(theme.text_active)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    };

    for (i, (filter, icon_info, count)) in items.into_iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
            current_width += 2;
        }

        let is_selected = state.global.status_filter == filter;

        if let Some((icon, icon_color)) = icon_info {
            let icon_style = Style::default().fg(icon_color);
            let icon_style = if is_selected {
                selected_style(icon_style)
            } else {
                icon_style
            };
            spans.push(Span::styled(icon.to_string(), icon_style));
            current_width += display_width(icon);

            let count_str = format!("{count}");
            let count_style = if count == 0 {
                Style::default().fg(theme.border_inactive)
            } else {
                Style::default().fg(theme.text_active)
            };
            let count_style = if is_selected {
                selected_style(count_style)
            } else {
                count_style
            };
            current_width += count_str.len();
            spans.push(Span::styled(count_str, count_style));
        } else {
            let style = if is_selected {
                selected_style(Style::default().fg(theme.text_active))
            } else {
                Style::default().fg(theme.text_muted)
            };
            spans.push(Span::styled("All", style));
            current_width += 3;
        }
    }

    // Repo filter button — right-aligned
    let repo_icon = "▼";
    let repo_label = match &state.global.repo_filter {
        RepoFilter::All => repo_icon.to_string(),
        RepoFilter::Repo(name) => {
            let max_w = 8;
            let truncated = truncate_to_width(name, max_w);
            format!("{} {}", repo_icon, truncated)
        }
    };
    let repo_btn_width = display_width(&repo_label) + 1; // 1 for leading space
    let gap = (bar_width as usize).saturating_sub(current_width + repo_btn_width);
    let repo_button_col = (current_width + gap) as u16;

    spans.push(Span::raw(" ".repeat(gap)));

    let repo_has_filter = !matches!(state.global.repo_filter, RepoFilter::All);
    let repo_style = if state.repo_popup_open {
        Style::default()
            .fg(theme.text_active)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    } else if repo_has_filter {
        Style::default()
            .fg(theme.text_active)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_muted)
    };
    spans.push(Span::styled(format!(" {}", repo_label), repo_style));

    (Line::from(spans), repo_button_col)
}

fn render_version_banner<'a>(state: &AppState, width: usize) -> Option<Line<'a>> {
    let theme = &state.theme;
    let notice = state.version_notice.as_ref()?;
    let text = format!("new release v{}!", notice.latest_version);
    let gap = pad_to(display_width(&text), width);

    Some(Line::from(vec![
        Span::raw(gap),
        Span::styled(text, Style::default().fg(theme.status_waiting)),
    ]))
}

fn render_repo_popup(frame: &mut Frame, state: &mut AppState, area: Rect) {
    let theme = &state.theme;
    let repos = state.repo_names();
    if repos.is_empty() {
        return;
    }

    let max_name_len = repos.iter().map(|r| display_width(r)).max().unwrap_or(3);
    // Width: marker(2) + name + padding(1) + borders(2)
    let popup_width = (max_name_len + 5).min(area.width as usize).max(10) as u16;
    let popup_height = (repos.len() as u16 + 2).min(area.height.saturating_sub(1)); // +2 for borders

    // Right-aligned, below filter bar
    let popup_x = area.x + area.width.saturating_sub(popup_width);
    let popup_y = area.y + 1;

    let popup_rect = Rect::new(popup_x, popup_y, popup_width, popup_height);
    state.repo_popup_area = Some(popup_rect);

    frame.render_widget(Clear, popup_rect);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border_active));
    let inner = block.inner(popup_rect);
    frame.render_widget(block, popup_rect);

    let inner_width = inner.width as usize;
    for (i, name) in repos.iter().enumerate() {
        if i >= inner.height as usize {
            break;
        }

        let is_highlighted = i == state.repo_popup_selected;
        let is_current = match &state.global.repo_filter {
            RepoFilter::All => i == 0,
            RepoFilter::Repo(n) => *n == *name,
        };

        let marker = if is_current { "● " } else { "  " };
        let truncated = truncate_to_width(name, inner_width.saturating_sub(2));
        let text = format!("{}{}", marker, truncated);
        let text_dw = display_width(&text);
        let padding = " ".repeat(inner_width.saturating_sub(text_dw));

        let style = if is_highlighted {
            Style::default()
                .fg(theme.text_active)
                .bg(theme.selection_bg)
                .add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default()
                .fg(theme.text_active)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_muted)
        };

        let line_rect = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!("{}{}", text, padding),
                style,
            ))),
            line_rect,
        );
    }
}

pub fn draw_agents(frame: &mut Frame, state: &mut AppState, area: Rect) {
    let theme = &state.theme;
    let width = area.width as usize;

    // Fixed filter bar (1 row)
    let filter_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1.min(area.height),
    };
    let (filter_line, repo_btn_col) = render_filter_bar(state, area.width);
    state.repo_button_col = repo_btn_col;
    frame.render_widget(Paragraph::new(vec![filter_line]), filter_area);

    // Scrollable agent list below
    let list_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: area.height.saturating_sub(1),
    };

    let mut lines: Vec<Line<'_>> = Vec::new();
    let mut line_to_row: Vec<Option<usize>> = Vec::new();
    let mut row_index: usize = 0;

    if let Some(version_banner) = render_version_banner(state, width) {
        lines.push(version_banner);
        line_to_row.push(None);
    }

    let filter = state.global.status_filter;

    for group in &state.repo_groups {
        if !state.global.repo_filter.matches_group(&group.name) {
            continue;
        }
        let filtered_panes: Vec<_> = group
            .panes
            .iter()
            .filter(|(pane, _)| filter.matches(&pane.status))
            .collect();
        if filtered_panes.is_empty() {
            continue;
        }

        let group_has_focused_pane = state.focused_pane_id.as_ref().map_or(false, |fid| {
            group.panes.iter().any(|(p, _)| p.pane_id == *fid)
        });

        let border_color = if group_has_focused_pane {
            theme.border_active
        } else {
            theme.border_inactive
        };
        let title = &group.name;

        let title_dw = display_width(title);
        let fill_len = width.saturating_sub(3 + title_dw + 1);
        let title_color = if group_has_focused_pane {
            theme.border_active
        } else {
            theme.text_muted
        };
        lines.push(Line::from(vec![
            Span::styled("╭ ", Style::default().fg(border_color)),
            Span::styled(title.clone(), Style::default().fg(title_color)),
            Span::styled(
                format!(" {}╮", "─".repeat(fill_len)),
                Style::default().fg(border_color),
            ),
        ]));
        line_to_row.push(None);

        for (pi, (pane, git_info)) in filtered_panes.iter().enumerate() {
            if pi > 0 {
                let gray = Style::default().fg(theme.border_inactive);
                let dashes = "─".repeat(width.saturating_sub(4));
                lines.push(Line::from(vec![
                    Span::styled("│", Style::default().fg(border_color)),
                    Span::styled(format!(" {} ", dashes), gray),
                    Span::styled("│", Style::default().fg(border_color)),
                ]));
                line_to_row.push(None);
            }

            let is_selected = state.sidebar_focused
                && state.focus == Focus::Panes
                && row_index == state.global.selected_pane_row;

            let is_active = state
                .focused_pane_id
                .as_ref()
                .map_or(false, |id| id == &pane.pane_id);

            let pane_state = state.pane_state(&pane.pane_id);
            let ports = pane_state.map(|s| s.ports.as_slice());
            let command = None;
            let task_progress = pane_state.and_then(|s| s.task_progress.as_ref());
            let pane_lines = row::render_pane_lines_with_ports(
                pane,
                git_info,
                ports,
                command,
                task_progress,
                is_selected,
                is_active,
                border_color,
                width,
                &state.icons,
                theme,
                state.spinner_frame,
                state.now,
            );
            let pane_line_count = pane_lines.len();
            lines.extend(pane_lines);
            for _ in 0..pane_line_count {
                line_to_row.push(Some(row_index));
            }

            row_index += 1;
        }

        let bottom_line = format!("╰{}╯", "─".repeat(width.saturating_sub(2)));
        lines.push(Line::from(Span::styled(
            bottom_line,
            Style::default().fg(border_color),
        )));
        line_to_row.push(None);
    }

    state.line_to_row = line_to_row;
    state.panes_scroll.total_lines = lines.len();
    state.panes_scroll.visible_height = list_area.height as usize;

    // Auto-scroll to keep selected agent visible
    if state.sidebar_focused && state.focus == Focus::Panes {
        let mut first_line: Option<usize> = None;
        let mut last_line: Option<usize> = None;
        for (i, mapping) in state.line_to_row.iter().enumerate() {
            if *mapping == Some(state.global.selected_pane_row) {
                if first_line.is_none() {
                    first_line = Some(i);
                }
                last_line = Some(i);
            }
        }
        if let (Some(first), Some(last)) = (first_line, last_line) {
            let mut effective_last = last;
            for i in (last + 1)..state.line_to_row.len() {
                if state.line_to_row[i].is_none() {
                    effective_last = i;
                } else {
                    break;
                }
            }
            let visible_h = list_area.height as usize;
            let offset = state.panes_scroll.offset;
            if first < offset {
                state.panes_scroll.offset = first.saturating_sub(1);
            } else if effective_last >= offset + visible_h {
                state.panes_scroll.offset = (effective_last + 1).saturating_sub(visible_h);
            }
        }
    }

    let paragraph = Paragraph::new(lines).scroll((state.panes_scroll.offset as u16, 0));
    frame.render_widget(paragraph, list_area);

    // Render popup overlay on top if open
    if state.repo_popup_open {
        render_repo_popup(frame, state, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn render_version_banner_right_aligns() {
        let mut state = crate::state::AppState::new(String::new());
        state.version_notice = Some(crate::version::UpdateNotice {
            local_version: "0.2.6".into(),
            latest_version: "0.2.7".into(),
        });

        let line = render_version_banner(&state, 30).expect("banner should render");
        let text = line_text(&line);

        assert!(text.ends_with("new release v0.2.7!"));
        assert_eq!(display_width(&text), 30);
    }

    // ─── render_filter_bar tests ──────────────────────────────

    fn make_state_with_groups(groups: Vec<crate::group::RepoGroup>) -> AppState {
        let mut state = AppState::new("%99".into());
        state.repo_groups = groups;
        state.rebuild_row_targets();
        state
    }

    fn filter_bar_text(state: &AppState, width: u16) -> String {
        let (line, _) = render_filter_bar(state, width);
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn render_filter_bar_includes_repo_button() {
        let state = make_state_with_groups(vec![]);
        let text = filter_bar_text(&state, 28);
        assert!(
            text.contains("▼"),
            "filter bar should contain repo button ▼"
        );
    }

    #[test]
    fn render_filter_bar_repo_button_col_returned() {
        let state = make_state_with_groups(vec![]);
        let (_, col) = render_filter_bar(&state, 28);
        // repo button should be near the right edge
        assert!(
            col > 15,
            "repo button col should be right-aligned, got {col}"
        );
        assert!(
            col < 28,
            "repo button col should be within width, got {col}"
        );
    }

    #[test]
    fn render_filter_bar_shows_repo_name_when_filtered() {
        let mut state = make_state_with_groups(vec![crate::group::RepoGroup {
            name: "my-app".into(),
            has_focus: true,
            panes: vec![],
        }]);
        state.global.repo_filter = RepoFilter::Repo("my-app".into());
        let text = filter_bar_text(&state, 40);
        assert!(
            text.contains("my-app"),
            "filter bar should show filtered repo name, got: {text}"
        );
    }

    #[test]
    fn render_filter_bar_truncates_long_repo_name() {
        let mut state = make_state_with_groups(vec![crate::group::RepoGroup {
            name: "very-long-repository-name".into(),
            has_focus: true,
            panes: vec![],
        }]);
        state.global.repo_filter = RepoFilter::Repo("very-long-repository-name".into());
        let text = filter_bar_text(&state, 28);
        // Should be truncated, not the full name
        assert!(
            !text.contains("very-long-repository-name"),
            "long repo name should be truncated, got: {text}"
        );
        assert!(text.contains("▼"));
    }

    #[test]
    fn render_filter_bar_popup_open_styling() {
        let mut state = make_state_with_groups(vec![]);
        state.repo_popup_open = true;
        let (line, _) = render_filter_bar(&state, 28);
        // Find the repo button span and check it has UNDERLINED modifier
        let last_span = line.spans.last().unwrap();
        assert!(
            last_span.style.add_modifier.contains(Modifier::UNDERLINED),
            "repo button should be underlined when popup is open"
        );
    }
}
