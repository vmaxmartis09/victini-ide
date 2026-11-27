use crate::ai::chat_session::{QuotedContext, ThreadAgentMode};
use crate::ai::elegans_agents::ElegansAgent;
use crate::ai::scheduler::ExecutionMode;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum AgentStreamEvent {
    AgentActivated {
        msg_id: String,
        agent: ElegansAgent,
    },
    Line {
        msg_id: String,
        text: String,
        elapsed_secs: f32,
    },
    Finished {
        msg_id: String,
        elapsed_secs: f32,
    },
}

pub struct AgyBridge;

impl AgyBridge {
    pub fn detect_agent_for_prompt(prompt: &str, fallback: ElegansAgent) -> ElegansAgent {
        let lower = prompt.to_lowercase();
        let lane = crate::elegans::router::classify_lane(&lower);
        if lane == "FE-DEPLOY" {
            return ElegansAgent::DevopsDeployer;
        }
        let agents = crate::elegans::router::determine_agents(lane, &lower);
        if let Some(primary) = agents.first().map(|s| s.as_str()) {
            match primary {
                "fe_design_scout" => return ElegansAgent::DesignScout,
                "fe_ui_ux_critic" => return ElegansAgent::UiUxCritic,
                "fe_code_auditor" => return ElegansAgent::CodeAuditor,
                "fe_qa_tester" => return ElegansAgent::QaTester,
                "fe_devops_deployer" => return ElegansAgent::DevopsDeployer,
                "fe_senior_dev" => {}
                _ => {}
            }
        }

        if lower.contains("design") || lower.contains("video") || lower.contains("reverse") {
            ElegansAgent::DesignScout
        } else if lower.contains("ui")
            || lower.contains("ux")
            || lower.contains("màu")
            || lower.contains("responsive")
            || lower.contains("font")
        {
            ElegansAgent::UiUxCritic
        } else if lower.contains("audit")
            || lower.contains("security")
            || lower.contains("bảo mật")
            || lower.contains("review")
        {
            ElegansAgent::CodeAuditor
        } else if lower.contains("qa") || lower.contains("test") || lower.contains("emu") {
            ElegansAgent::QaTester
        } else if lower.contains("deploy")
            || lower.contains("build")
            || lower.contains("release")
            || lower.contains("gitlab")
            || lower.contains("push")
        {
            ElegansAgent::DevopsDeployer
        } else {
            fallback
        }
    }

    pub fn build_contextual_prompt(
        raw_prompt: &str,
        quoted: &Option<QuotedContext>,
        agent_mode: &ThreadAgentMode,
        agent: ElegansAgent,
        exec_mode: ExecutionMode,
        attached_files: &[(String, String)],
        attached_skills: &[(String, String)],
    ) -> String {
        let mut full = String::new();
        let prompt_lc = raw_prompt.to_lowercase();
        let lane = crate::elegans::router::classify_lane(&prompt_lc);
        let (genre, seo) = crate::elegans::router::classify_genre(&prompt_lc);
        let (archetype, _) =
            crate::elegans::router::determine_solution_archetype(genre, &prompt_lc, "rust-egui");

        match agent_mode {
            ThreadAgentMode::SwarmAuto => {
                full.push_str(&format!(
                    "[Swarm Auto-Router -> Lane: {} | Genre: {} ({}) | Role: {} ({})]\n[Archetype: {}]\n",
                    lane,
                    genre,
                    seo,
                    agent.title(),
                    agent.ponytail_tier(),
                    archetype
                ));
            }
            ThreadAgentMode::DedicatedCore(ag) => {
                full.push_str(&format!(
                    "[DEDICATED 1-ON-1 AGENT SESSION: {} — {} (Tier: {})]\nAnswer strictly from the perspective and expertise of {}.\n",
                    ag.name(),
                    ag.title(),
                    ag.ponytail_tier(),
                    ag.title()
                ));
            }
            ThreadAgentMode::DedicatedCustom {
                name,
                role,
                system_prompt,
            } => {
                full.push_str(&format!(
                    "[DEDICATED 1-ON-1 CUSTOM SUBAGENT: @{} — {}]\nSystem Prompt & Rules:\n{}\n\n",
                    name, role, system_prompt
                ));
            }
        }

        let mode_note = match exec_mode {
            ExecutionMode::Auto => "Execution Mode: AUTO (Propose and execute clean surgical solutions).",
            ExecutionMode::Gatekeeper => "Execution Mode: GATEKEEPER (Explain plan and ask before destructive writes).",
            ExecutionMode::Bypass => "Execution Mode: BYPASS (Maximum speed direct execution).",
        };
        full.push_str(mode_note);
        full.push('\n');

        if !attached_skills.is_empty() {
            full.push_str("\n=== ACTIVE SKILLS ($SKILL.MD) ===\n");
            for (sk_name, sk_body) in attached_skills {
                let snippet: String = sk_body.chars().take(1800).collect();
                full.push_str(&format!("--- Skill: ${} ---\n{}\n", sk_name, snippet));
            }
            full.push('\n');
        }

        if !attached_files.is_empty() {
            full.push_str("\n=== ATTACHED WORKSPACE FILES (@FILE) ===\n");
            for (f_path, f_content) in attached_files {
                let snippet: String = f_content.chars().take(3500).collect();
                full.push_str(&format!("--- File: @{} ---\n{}\n", f_path, snippet));
            }
            full.push('\n');
        }

        if let Some(q) = quoted {
            let mode_tag = if q.is_quote_block {
                "Quoted Block"
            } else {
                "Replying to Message"
            };
            full.push_str(&format!(
                "> [{} from {} ({})]:\n> {}\n\n",
                mode_tag,
                q.author_label,
                q.source_msg_id,
                q.snippet.replace('\n', "\n> ")
            ));
        }
        full.push_str(raw_prompt);
        full
    }

    pub fn spawn_vibe_session(
        msg_id: String,
        prompt: String,
        agent: ElegansAgent,
        workdir: PathBuf,
        cancel_flag: Arc<AtomicBool>,
        sender: mpsc::UnboundedSender<AgentStreamEvent>,
    ) {
        tokio::spawn(async move {
            let start = Instant::now();
            let agy_bin = crate::config::paths::get_agy_path();

            let _ = sender.send(AgentStreamEvent::AgentActivated {
                msg_id: msg_id.clone(),
                agent,
            });

            let mut cmd = Command::new(&agy_bin);
            cmd.current_dir(&workdir);
            cmd.arg("-p").arg(&prompt);
            cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
            cmd.kill_on_drop(true);

            match cmd.spawn() {
                Ok(mut child) => {
                    if let Some(stdout) = child.stdout.take() {
                        let mut reader = BufReader::new(stdout).lines();
                        while let Ok(Some(line)) = reader.next_line().await {
                            if cancel_flag.load(Ordering::Relaxed) {
                                let _ = child.kill().await;
                                let _ = sender.send(AgentStreamEvent::Line {
                                    msg_id: msg_id.clone(),
                                    text: "[Đã dừng tiến trình theo lệnh của Ngài Vmax]"
                                        .to_string(),
                                    elapsed_secs: start.elapsed().as_secs_f32(),
                                });
                                break;
                            }
                            if !line.trim().is_empty() {
                                let _ = sender.send(AgentStreamEvent::Line {
                                    msg_id: msg_id.clone(),
                                    text: line,
                                    elapsed_secs: start.elapsed().as_secs_f32(),
                                });
                            }
                        }
                    }
                    if !cancel_flag.load(Ordering::Relaxed) {
                        if let Some(stderr) = child.stderr.take() {
                            let mut reader = BufReader::new(stderr).lines();
                            while let Ok(Some(err_line)) = reader.next_line().await {
                                if !err_line.trim().is_empty() && !err_line.contains("DEBUG") {
                                    let _ = sender.send(AgentStreamEvent::Line {
                                        msg_id: msg_id.clone(),
                                        text: format!("Warning: {}", err_line),
                                        elapsed_secs: start.elapsed().as_secs_f32(),
                                    });
                                }
                            }
                        }
                        let _ = child.wait().await;
                    }
                    let _ = sender.send(AgentStreamEvent::Finished {
                        msg_id,
                        elapsed_secs: start.elapsed().as_secs_f32(),
                    });
                }
                Err(e) => {
                    let _ = sender.send(AgentStreamEvent::Line {
                        msg_id: msg_id.clone(),
                        text: format!("Lỗi khởi chạy AGY CLI ({:?}): {}", agy_bin, e),
                        elapsed_secs: start.elapsed().as_secs_f32(),
                    });
                    let _ = sender.send(AgentStreamEvent::Finished {
                        msg_id,
                        elapsed_secs: start.elapsed().as_secs_f32(),
                    });
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_agent_and_build_contextual_prompt_work() {
        assert_eq!(
            AgyBridge::detect_agent_for_prompt("Review và audit bảo mật code", ElegansAgent::SeniorDev),
            ElegansAgent::CodeAuditor
        );
        assert_eq!(
            AgyBridge::detect_agent_for_prompt("Deploy ứng dụng lên Cloudflare", ElegansAgent::SeniorDev),
            ElegansAgent::DevopsDeployer
        );
        assert_eq!(
            AgyBridge::detect_agent_for_prompt("Bóc tách video TikTok này", ElegansAgent::SeniorDev),
            ElegansAgent::DesignScout
        );

        let prompt = AgyBridge::build_contextual_prompt(
            "Tối ưu hàm xử lý giỏ hàng",
            &Some(QuotedContext {
                source_msg_id: "MSG-1".to_string(),
                author_label: "fe_senior_dev".to_string(),
                snippet: "fn calc() {}".to_string(),
                is_quote_block: true,
            }),
            &ThreadAgentMode::SwarmAuto,
            ElegansAgent::SeniorDev,
            ExecutionMode::Gatekeeper,
            &[("src/cart.rs".to_string(), "pub fn cart() {}".to_string())],
            &[("ponytail".to_string(), "Zero bloat rule".to_string())],
        );

        assert!(prompt.contains("Execution Mode: GATEKEEPER"));
        assert!(prompt.contains("--- Skill: $ponytail ---"));
        assert!(prompt.contains("--- File: @src/cart.rs ---"));
        assert!(prompt.contains("Quoted Block from fe_senior_dev"));
        assert!(prompt.contains("Tối ưu hàm xử lý giỏ hàng"));
    }
}
