use crate::ai::ElegansAgent;
use egui::{include_image, pos2, vec2, Color32, Image, ImageSource, Pos2, Rounding, Stroke, Ui};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentAvatarState {
    Idle,
    Thinking,
    Answering,
    Offline,
}

impl AgentAvatarState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::Thinking => "Thinking...",
            Self::Answering => "Streaming...",
            Self::Offline => "Standby",
        }
    }
}

pub struct AvatarManager;

impl AvatarManager {
    pub fn get_avatar_for_agent(
        agent: ElegansAgent,
        state: AgentAvatarState,
    ) -> ImageSource<'static> {
        match (agent, state) {
            // SeniorDev & DevopsDeployer -> ag-mac
            (ElegansAgent::SeniorDev | ElegansAgent::DevopsDeployer, AgentAvatarState::Idle) => {
                include_image!("../../assets/avatars/ag-mac-idle.svg")
            }
            (ElegansAgent::SeniorDev | ElegansAgent::DevopsDeployer, AgentAvatarState::Thinking) => {
                include_image!("../../assets/avatars/ag-mac-thinking.svg")
            }
            (ElegansAgent::SeniorDev | ElegansAgent::DevopsDeployer, AgentAvatarState::Answering) => {
                include_image!("../../assets/avatars/ag-mac-answering.svg")
            }
            (ElegansAgent::SeniorDev | ElegansAgent::DevopsDeployer, AgentAvatarState::Offline) => {
                include_image!("../../assets/avatars/ag-mac-offline.svg")
            }

            // UiUxCritic -> claude-mac
            (ElegansAgent::UiUxCritic, AgentAvatarState::Idle) => {
                include_image!("../../assets/avatars/claude-mac-idle.svg")
            }
            (ElegansAgent::UiUxCritic, AgentAvatarState::Thinking) => {
                include_image!("../../assets/avatars/claude-mac-thinking.svg")
            }
            (ElegansAgent::UiUxCritic, AgentAvatarState::Answering) => {
                include_image!("../../assets/avatars/claude-mac-answering.svg")
            }
            (ElegansAgent::UiUxCritic, AgentAvatarState::Offline) => {
                include_image!("../../assets/avatars/claude-mac-offline.svg")
            }

            // DesignScout -> ag-tma
            (ElegansAgent::DesignScout, AgentAvatarState::Idle) => {
                include_image!("../../assets/avatars/ag-tma-idle.svg")
            }
            (ElegansAgent::DesignScout, AgentAvatarState::Thinking) => {
                include_image!("../../assets/avatars/ag-tma-thinking.svg")
            }
            (ElegansAgent::DesignScout, AgentAvatarState::Answering) => {
                include_image!("../../assets/avatars/ag-tma-answering.svg")
            }
            (ElegansAgent::DesignScout, AgentAvatarState::Offline) => {
                include_image!("../../assets/avatars/ag-tma-offline.svg")
            }

            // CodeAuditor -> claude-tma
            (ElegansAgent::CodeAuditor, AgentAvatarState::Idle) => {
                include_image!("../../assets/avatars/claude-tma-idle.svg")
            }
            (ElegansAgent::CodeAuditor, AgentAvatarState::Thinking) => {
                include_image!("../../assets/avatars/claude-tma-thinking.svg")
            }
            (ElegansAgent::CodeAuditor, AgentAvatarState::Answering) => {
                include_image!("../../assets/avatars/claude-tma-answering.svg")
            }
            (ElegansAgent::CodeAuditor, AgentAvatarState::Offline) => {
                include_image!("../../assets/avatars/claude-tma-offline.svg")
            }

            // QaTester -> iphone
            (ElegansAgent::QaTester, AgentAvatarState::Idle) => {
                include_image!("../../assets/avatars/iphone-idle.svg")
            }
            (ElegansAgent::QaTester, AgentAvatarState::Thinking) => {
                include_image!("../../assets/avatars/iphone-thinking.svg")
            }
            (ElegansAgent::QaTester, AgentAvatarState::Answering) => {
                include_image!("../../assets/avatars/iphone-answering.svg")
            }
            (ElegansAgent::QaTester, AgentAvatarState::Offline) => {
                include_image!("../../assets/avatars/iphone-offline.svg")
            }
        }
    }

    /// Renders the Notionist SVG avatar with live motion physics (breathing, pulsing neon halo, orbital arc)
    pub fn render_animated_avatar(
        ui: &mut Ui,
        agent: ElegansAgent,
        state: AgentAvatarState,
        size: f32,
        elapsed_secs: f32,
        accent_color: Color32,
    ) {
        let (rect, _) = ui.allocate_exact_size(vec2(size + 6.0, size + 6.0), egui::Sense::hover());
        let center = rect.center();
        let radius = size * 0.5;

        // Motion offset (gentle bobbing when active)
        let bob_y = match state {
            AgentAvatarState::Thinking => (elapsed_secs * 4.5).sin() * 1.8,
            AgentAvatarState::Answering => (elapsed_secs * 6.5).sin() * 2.2,
            AgentAvatarState::Idle => (elapsed_secs * 1.6).sin() * 0.6,
            AgentAvatarState::Offline => 0.0,
        };
        let avatar_center = pos2(center.x, center.y + bob_y);
        let avatar_rect = egui::Rect::from_center_size(avatar_center, vec2(size, size));

        // 1. Background halo & animated rings
        let painter = ui.painter();
        match state {
            AgentAvatarState::Thinking => {
                let pulse = ((elapsed_secs * 3.5).sin() * 0.5) + 0.5;
                let ring_col = Color32::from_rgba_unmultiplied(0xF7, 0xB9, 0x55, (90.0 + pulse * 130.0) as u8);
                painter.circle_stroke(avatar_center, radius + 1.5 + pulse * 2.0, Stroke::new(1.5, ring_col));

                // Rotating orbital dot
                let angle = elapsed_secs * 3.8;
                let orb_pos = Pos2::new(
                    avatar_center.x + (radius + 2.5) * angle.cos(),
                    avatar_center.y + (radius + 2.5) * angle.sin(),
                );
                painter.circle_filled(orb_pos, 2.5, accent_color);
                ui.ctx().request_repaint();
            }
            AgentAvatarState::Answering => {
                let pulse = ((elapsed_secs * 5.0).sin() * 0.5) + 0.5;
                let ring_col = Color32::from_rgba_unmultiplied(
                    accent_color.r(),
                    accent_color.g(),
                    accent_color.b(),
                    (110.0 + pulse * 140.0) as u8,
                );
                painter.circle_stroke(avatar_center, radius + 1.5 + pulse * 2.5, Stroke::new(1.8, ring_col));
                ui.ctx().request_repaint();
            }
            AgentAvatarState::Idle => {
                painter.circle_stroke(
                    avatar_center,
                    radius + 1.0,
                    Stroke::new(1.0, Color32::from_rgb(0x26, 0x2C, 0x3A)),
                );
            }
            AgentAvatarState::Offline => {
                painter.circle_stroke(
                    avatar_center,
                    radius + 1.0,
                    Stroke::new(1.0, Color32::from_rgb(0x18, 0x1A, 0x22)),
                );
            }
        }

        // 2. Render SVG Image inside avatar_rect
        let src = Self::get_avatar_for_agent(agent, state);
        let img = Image::new(src)
            .fit_to_exact_size(vec2(size, size))
            .rounding(Rounding::same(radius));
        ui.put(avatar_rect, img);

        // 3. Corner status dot
        let dot_color = match state {
            AgentAvatarState::Idle => Color32::from_rgb(0x4E, 0xC9, 0xB0),
            AgentAvatarState::Thinking => Color32::from_rgb(0xF7, 0xB9, 0x55),
            AgentAvatarState::Answering => accent_color,
            AgentAvatarState::Offline => Color32::from_rgb(0x55, 0x5C, 0x68),
        };
        let dot_pos = pos2(avatar_rect.max.x - 3.0, avatar_rect.max.y - 3.0);
        ui.painter().circle_filled(dot_pos, 4.2, Color32::from_rgb(0x0B, 0x0C, 0x10));
        ui.painter().circle_filled(dot_pos, 3.0, dot_color);
    }
}
