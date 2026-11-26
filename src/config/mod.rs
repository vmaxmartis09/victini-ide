pub mod paths;
pub mod project_manager;
pub mod skill_discovery;
pub mod skill_import;

pub use project_manager::ProjectManager;
pub use skill_discovery::{
    DiscoveredAgent, DiscoveredSkill, SkillDiscovery, SkillScope, SkillSourceMeta, TrashedItem,
};
pub use skill_import::{
    spawn_install_candidates, spawn_scan_github_repo, spawn_update_skill_from_source,
    ConflictStrategy, ImportEvent, InstallReport, ScanRepoResult,
};
