use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponsiveBreakpoint {
    Compact, // < 920px: Mini sidebar rail, auto-hide right dock, 1-2 col kanban
    Medium,  // 920px..1280px: Resizable left sidebar, optional right dock, 2-4 col kanban
    Wide,    // >= 1280px: Full 3-pane studio layout
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CenterViewMode {
    ChatTimeline,
    KanbanBoard,
    SplitChatKanban,
    GitDiffModal,
    ProjectsModal,
    AgentSkillStudio,
    SettingsModal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayoutState {
    pub show_left_sidebar: bool,
    pub left_mini_mode: bool,
    pub show_right_dock: bool,
    pub center_mode: CenterViewMode,
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub auto_canh_gio: bool,
    pub last_window_width: f32,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            show_left_sidebar: true,
            left_mini_mode: false,
            show_right_dock: true,
            center_mode: CenterViewMode::ChatTimeline,
            left_panel_width: 230.0,
            right_panel_width: 290.0,
            auto_canh_gio: false,
            last_window_width: 1280.0,
        }
    }
}

impl LayoutState {
    pub fn compute_breakpoint(&mut self, window_width: f32) -> ResponsiveBreakpoint {
        let prev_bp = Self::breakpoint_for(self.last_window_width);
        let next_bp = Self::breakpoint_for(window_width);

        // Automatically adapt panels when crossing breakpoints
        if prev_bp != next_bp {
            match next_bp {
                ResponsiveBreakpoint::Compact => {
                    self.left_mini_mode = true;
                    self.show_right_dock = false;
                }
                ResponsiveBreakpoint::Medium => {
                    self.left_mini_mode = false;
                    self.show_left_sidebar = true;
                }
                ResponsiveBreakpoint::Wide => {
                    self.left_mini_mode = false;
                    self.show_left_sidebar = true;
                    self.show_right_dock = true;
                }
            }
        }
        self.last_window_width = window_width;
        next_bp
    }

    pub fn breakpoint_for(width: f32) -> ResponsiveBreakpoint {
        if width < 920.0 {
            ResponsiveBreakpoint::Compact
        } else if width < 1280.0 {
            ResponsiveBreakpoint::Medium
        } else {
            ResponsiveBreakpoint::Wide
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn responsive_breakpoints_auto_adapt_side_panels() {
        let mut layout = LayoutState::default();
        assert_eq!(layout.compute_breakpoint(1320.0), ResponsiveBreakpoint::Wide);
        assert!(layout.show_left_sidebar && !layout.left_mini_mode && layout.show_right_dock);

        assert_eq!(layout.compute_breakpoint(760.0), ResponsiveBreakpoint::Compact);
        assert!(layout.left_mini_mode);
        assert!(!layout.show_right_dock);

        assert_eq!(layout.compute_breakpoint(1080.0), ResponsiveBreakpoint::Medium);
        assert!(layout.show_left_sidebar && !layout.left_mini_mode);

        assert_eq!(layout.compute_breakpoint(1400.0), ResponsiveBreakpoint::Wide);
        assert!(layout.show_right_dock && !layout.left_mini_mode);
    }
}
