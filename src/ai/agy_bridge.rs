use crate::ai::chat_session::QuotedContext;
use crate::ai::elegans_agents::ElegansAgent;
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
        agent: ElegansAgent,
    ) -> String {
        let mut full = String::new();
        full.push_str(&format!(
            "[Role: {} ({})]\n",
            agent.title(),
            agent.ponytail_tier()
        ));
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
                                    text: "⏹ [Đã dừng tiến trình theo lệnh của Ngài Vmax]".to_string(),
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
                                        text: format!("⚠️ {}", err_line),
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
                        text: format!("❌ Lỗi khởi chạy AGY CLI ({:?}): {}", agy_bin, e),
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
