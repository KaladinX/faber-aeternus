use iocraft::prelude::*;
use crate::state::AppState;
use std::process;
use std::sync::Arc;

use crate::tui::theme::THEME;

#[derive(Default, Props)]
pub struct FaberAppProps {
    pub state: Option<Arc<AppState>>,
}

#[component]
pub fn FaberApp(mut hooks: Hooks, props: &FaberAppProps) -> impl Into<AnyElement<'static>> {
    let mut input = hooks.use_state(|| String::new());

    hooks.use_terminal_events(|event| {
        if let iocraft::TerminalEvent::Key(key) = event {
            match key.code {
                iocraft::KeyCode::Char('c') if key.modifiers.contains(iocraft::KeyModifiers::CONTROL) => {
                    process::exit(0);
                }
                _ => {}
            }
        }
    });

    let history_elements = if let Some(state) = &props.state {
        state.chat_history.iter().map(|msg| {
            element! {
                Text(color: THEME.text, content: msg.content.clone())
            }
        }).collect::<Vec<_>>()
    } else {
        vec![]
    };

    element! {
        View(display: Display::Flex, flex_direction: FlexDirection::Row, width: Percent(100.0), height: Percent(100.0), background_color: THEME.surface0) {
            
            View(width: Percent(25.0), flex_direction: FlexDirection::Column, padding: 1, background_color: THEME.surface1) {
                Text(color: THEME.primary, content: "📁 Project Tree")
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
                Text(color: THEME.secondary, content: "📝 Preview")
            }
        }
    }
}

pub async fn run_tui(cli: crate::cli::Cli) -> anyhow::Result<()> {
    let state = Arc::new(AppState::new(cli));
    
    element! {
        FaberApp(state: Some(state))
    }
    .print();

    Ok(())
}