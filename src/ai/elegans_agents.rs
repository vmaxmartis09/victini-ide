use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ElegansAgent {
    DesignScout,
    UiUxCritic,
    SeniorDev,
    CodeAuditor,
    QaTester,
    DevopsDeployer,
}

impl ElegansAgent {
    pub fn name(&self) -> &'static str {
        match self {
            Self::DesignScout => "fe_design_scout",
            Self::UiUxCritic => "fe_ui_ux_critic",
            Self::SeniorDev => "fe_senior_dev",
            Self::CodeAuditor => "fe_code_auditor",
            Self::QaTester => "fe_qa_tester",
            Self::DevopsDeployer => "fe_devops_deployer",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::DesignScout => "🔭",
            Self::UiUxCritic => "🎨",
            Self::SeniorDev => "⚡",
            Self::CodeAuditor => "🛡️",
            Self::QaTester => "📱",
            Self::DevopsDeployer => "🚀",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::DesignScout => "Design Scout & Reverse",
            Self::UiUxCritic => "Art Director & Anti-Slop",
            Self::SeniorDev => "Senior Full-Spectrum Dev",
            Self::CodeAuditor => "Quality & Security Auditor",
            Self::QaTester => "QA & 140+ Emu Tester",
            Self::DevopsDeployer => "Multi-Cloud Deployer",
        }
    }

    pub fn ponytail_tier(&self) -> &'static str {
        match self {
            Self::DesignScout => "ZERO-LOSS-REVERSE",
            Self::UiUxCritic => "ZERO-SLOP-JUDGE",
            Self::SeniorDev => "SURGICAL-SENIOR",
            Self::CodeAuditor => "ULTRA-RUTHLESS",
            Self::QaTester => "FULL-COVERAGE",
            Self::DevopsDeployer => "ZERO-CONFIG-SAFE",
        }
    }
}

pub const ALL_AGENTS: &[ElegansAgent] = &[
    ElegansAgent::DesignScout,
    ElegansAgent::UiUxCritic,
    ElegansAgent::SeniorDev,
    ElegansAgent::CodeAuditor,
    ElegansAgent::QaTester,
    ElegansAgent::DevopsDeployer,
];
