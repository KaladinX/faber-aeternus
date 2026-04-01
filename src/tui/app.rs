// src/tui/app.rs
use iocraft::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::{AppState, PermissionState};
use crate::tui::theme::THEME;
use std::process;

#[derive(Default, Clone, Props)]
pub struct FaberAppProps {
    pub state: Option<Arc<Mutex<AppState>>>,
}

#[component]
pub fn FaberApp(mut hooks: Hooks, props: &FaberAppProps) -> impl Into<AnyElement<'static>> {
    let mut input = hooks.use_state(|| String::new());
    let mut tick = hooks.use_state(|| 0);
    let state_arc = props.state.as_ref().expect("AppState required").clone();

    hooks.use_terminal_events({
        let state_arc = state_arc.clone();
        let mut input = input.clone();
        let mut tick = tick.clone();
        move |event| {
            if let iocraft::TerminalEvent::Key(key) = event {
                if key.code == iocraft::KeyCode::Char('c') && key.modifiers.contains(iocraft::KeyModifiers::CONTROL) {
                    process::exit(0);
                }

                let permission_mode = {
                    if let Ok(s) = state_arc.try_lock() {
                        matches!(s.permission_state, PermissionState::Pending { .. })
                    } else {
                        false
                    }
                };

                if permission_mode {
                    let state_bg = state_arc.clone();
                    let mut tick_bg = tick.clone();
                    
                    if key.code == iocraft::KeyCode::Char('y') || key.code == iocraft::KeyCode::Char('Y') {
                        tokio::spawn(async move {
                            let mut s = state_bg.lock().await;
                            if let PermissionState::Pending { tool_name, params } = &s.permission_state {
                                s.permission_state = PermissionState::Approved { tool_name: tool_name.clone(), params: params.clone() };
                                tick_bg.set(tick_bg.to_string().parse::<i32>().unwrap_or(0) + 1);
                            }
                        });
                    } else if key.code == iocraft::KeyCode::Char('n') || key.code == iocraft::KeyCode::Char('N') || key.code == iocraft::KeyCode::Esc {
                        tokio::spawn(async move {
                            let mut s = state_bg.lock().await;
                            s.permission_state = PermissionState::Aborted;
                            tick_bg.set(tick_bg.to_string().parse::<i32>().unwrap_or(0) + 1);
                        });
                    }
                    return; // intercept keystrokes
                }

                if key.code == iocraft::KeyCode::Enter {
                    let msg = input.to_string();
                    if !msg.is_empty() {
                        input.set(String::new());
                        let state_bg = state_arc.clone();
                        let mut tick_bg = tick.clone();
                        
                        tokio::spawn(async move {
                            {
                                let mut s = state_bg.lock().await;
                                s.add_message(format!("👤 You: {}", msg));
                            }
                            tick_bg.set(tick_bg.to_string().parse::<i32>().unwrap_or(0) + 1);

                            let (llm_arc, chat_history) = {
                                let s = state_bg.lock().await;
                                (s.llm.clone(), s.chat_history.clone())
                            };
                            
                            let prompt = "You are faber-aeternus, an agentic coding harness. You can run tools via exactly <tool_call name=\"xxx\">{\"json\":...}</tool_call>. Tools allowed: read_file, edit_file (give proper diff inside {diff: ...}), run_shell.";
                            let stream_res = {
                                let llm = llm_arc.lock().await;
                                llm.generate_stream(prompt, &chat_history).await
                            };
                            
                            if let Ok(mut stream) = stream_res {
                                use futures::stream::StreamExt;
                                let mut parser = crate::tools::StreamingParser::new();

                                while let Some(chunk) = stream.next().await {
                                    if let Ok(text) = chunk {
                                        let (safe_text, tools) = parser.push(&text);
                                        
                                        if !safe_text.is_empty() {
                                            let mut s = state_bg.lock().await;
                                            s.add_message(format!("🤖 {}", safe_text));
                                            tick_bg.set(tick_bg.to_string().parse::<i32>().unwrap_or(0) + 1);
                                        }
                                        
                                        for tool in tools {
                                            {
                                                let mut s = state_bg.lock().await;
                                                s.permission_state = PermissionState::Pending { 
                                                    tool_name: tool.name.clone(), 
                                                    params: tool.params.clone() 
                                                };
                                            }
                                            tick_bg.set(tick_bg.to_string().parse::<i32>().unwrap_or(0) + 1);

                                            // Await user permission actively
                                            loop {
                                                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                                                let state_val = state_bg.lock().await.permission_state.clone();
                                                match state_val {
                                                    PermissionState::Approved { tool_name, params } => {
                                                        let registry = state_bg.lock().await.tools.clone();
                                                        let mut s = state_bg.lock().await;
                                                        s.permission_state = PermissionState::Idle;
                                                        s.add_message(format!("⚡ Executing {}...", tool_name));
                                                        drop(s);
                                                        tick_bg.set(tick_bg.to_string().parse::<i32>().unwrap_or(0) + 1);
                                                        
                                                        let result = registry.execute(&tool_name, params).await;
                                                        
                                                        let mut s = state_bg.lock().await;
                                                        match result {
                                                            Ok(out) => s.add_message(format!("✅ Done: {}", out)),
                                                            Err(e) => s.add_message(format!("❌ Failed: {}", e)),
                                                        }
                                                        break;
                                                    }
                                                    PermissionState::Aborted => {
                                                        let mut s = state_bg.lock().await;
                                                        s.permission_state = PermissionState::Idle;
                                                        s.add_message(format!("🚫 Tool call {} aborted by user.", tool.name));
                                                        break;
                                                    }
                                                    PermissionState::Pending { .. } => {
                                                        // Keep spinning
                                                    }
                                                    _ => break,
                                                }
                                            }
                                            tick_bg.set(tick_bg.to_string().parse::<i32>().unwrap_or(0) + 1);
                                        }
                                    }
                                }
                            } else {
                                let mut s = state_bg.lock().await;
                                s.add_message(format!("❌ Error connecting to LLM endpoint: {:?}", stream_res.err()));
                            }
                            tick_bg.set(tick_bg.to_string().parse::<i32>().unwrap_or(0) + 1);
                        });
                    }
                }
            }
        }
    });

    let s = match state_arc.try_lock() {
        Ok(guard) => guard,
        Err(_) => return element! { View(width: Percent(100.0), height: Percent(100.0)) { Text(content: "Loading...") } }.into(),
    };
    
    let history_elements = s.chat_history.iter().map(|msg| {
        element! {
            Text(color: THEME.text, content: msg.clone())
        }
    }).collect::<Vec<_>>();
    
    let preview_diff = s.preview_diff.clone();
    
    let (has_pending, t_name, t_param) = match &s.permission_state {
        PermissionState::Pending { tool_name, params } => {
            (true, tool_name.clone(), serde_json::to_string_pretty(params).unwrap_or_default())
        }
        _ => (false, String::new(), String::new()),
    };

    element! {
        View(display: Display::Flex, flex_direction: FlexDirection::Row, width: Percent(100.0), height: Percent(100.0), background_color: THEME.surface0) {
            
            View(width: Percent(25.0), flex_direction: FlexDirection::Column, padding: 1, background_color: THEME.surface1) {
                Text(color: THEME.primary, weight: Weight::Bold, content: "📁 Project Tree")
            }

            View(flex_grow: 1.0, flex_direction: FlexDirection::Column, padding: 1) {
                View(flex_grow: 1.0, flex_direction: FlexDirection::Column) {
                    Text(color: THEME.primary, weight: Weight::Bold, content: "🤖 faber-aeternus")
                    #(history_elements)
                }
                View(border_style: BorderStyle::Round, border_color: THEME.primary, padding: 1) {
                    TextInput(
                        value: input.to_string(),
                        on_change: move |v| input.set(v),
                        has_focus: true,
                    )
                }
            }

            View(width: Percent(30.0), flex_direction: FlexDirection::Column, padding: 1, background_color: THEME.surface1) {
                Text(color: THEME.secondary, weight: Weight::Bold, content: "📝 Live Preview")
                #(if has_pending {
                    vec![
                        element! { Text(color: THEME.error, content: "🔒 Permission Required") },
                        element! { Text(color: THEME.primary, content: t_name) },
                        element! { Text(color: THEME.text, content: t_param) },
                        element! { Text(color: THEME.secondary, content: "\nApprove? [Y/n]") },
                    ]
                } else {
                    vec![element! { Text(color: THEME.text, content: preview_diff) }]
                })
            }
        }
    }
}

pub async fn run_tui(cli: crate::cli::Cli) -> anyhow::Result<()> {
    let state = Arc::new(Mutex::new(AppState::new(cli)));
    
    element! {
        FaberApp(state: Some(state))
    }
    .print();

    Ok(())
}