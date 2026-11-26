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

        match agent_mode {
            ThreadAgentMode::SwarmAuto => {
                full.push_str(&format!(
                    "[Swarm Auto-Router -> Role: {} ({})]\n",
                    agent.title(),
                    agent.ponytail_tier()
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
