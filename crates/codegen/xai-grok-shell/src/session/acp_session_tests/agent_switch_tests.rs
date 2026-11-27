//! Mid-session `/agent` switch: the harness is rebuilt in place, the conversation survives.

use std::sync::Arc;

use xai_grok_sampling_types::conversation::ConversationItem;

use super::model_switch::AGENT_SWITCH_TURN_BUSY_REFUSAL;
use super::support::{build_actor, running_task_stub};
use crate::tools::bridge::ToolBridge;

fn tool_names(bridge: &Arc<ToolBridge>) -> Vec<String> {
    bridge
        .toolset()
        .tool_definitions()
        .into_iter()
        .map(|definition| definition.function.name)
        .collect()
}

fn system_head(conv: &[ConversationItem]) -> String {
    match conv.first() {
        Some(ConversationItem::System(sys)) => sys.content.as_ref().to_owned(),
        other => panic!("conversation must start with a System item, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn switch_rebuilds_identity_prompt_and_toolset_and_keeps_history() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let (actor, _gateway_rx) = build_actor().await;
            actor.chat_state_handle.replace_conversation(vec![
                ConversationItem::system("spawn-time prompt"),
                ConversationItem::user("<user_info>startup prefix</user_info>"),
                ConversationItem::user("please find the bug"),
                ConversationItem::system_reminder("<system-reminder>keep me</system-reminder>"),
                ConversationItem::assistant("looking now"),
            ]);
            let model_before = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor has a sampling config");
            let before_bridge = actor.agent.borrow().tool_bridge().clone();
            let before_tools = tool_names(&before_bridge);
            let before_name = actor.agent.borrow().definition().name.clone();
            assert_ne!(before_name, "explore");

            let switched = actor
                .handle_switch_agent_definition(
                    xai_grok_agent::AgentDefinition::explore(),
                    xai_grok_agent::DEFAULT_SYSTEM_PROMPT_LABEL.to_owned(),
                )
                .await
                .expect("in-process switch succeeds");
            assert_eq!(switched, "explore");

            let after_bridge = actor.agent.borrow().tool_bridge().clone();
            let after_tools = tool_names(&after_bridge);
            assert_eq!(
                actor.agent.borrow().definition().name,
                "explore",
                "identity changes to the requested agent"
            );
            assert!(
                !Arc::ptr_eq(&before_bridge, &after_bridge),
                "the tool bridge is rebuilt, not mutated in place"
            );
            assert_ne!(before_tools, after_tools, "the new agent's toolset wins");
            assert!(
                before_tools.is_empty(),
                "the fixture actor starts with no registered tools"
            );
            for expected in ["read_file", "grep", "list_dir"] {
                assert!(
                    after_tools.iter().any(|name| name == expected),
                    "explore's toolset must expose {expected}: {after_tools:?}"
                );
            }
            for forbidden in ["run_terminal_cmd", "search_replace"] {
                assert!(
                    !after_tools.iter().any(|name| name == forbidden),
                    "explore is read-only; {forbidden} must not be registered: {after_tools:?}"
                );
            }

            let conv = actor.chat_state_handle.get_conversation().await;
            let head = system_head(&conv);
            assert_eq!(
                head,
                actor.agent.borrow().system_prompt(),
                "the stored system head is the new agent's prompt"
            );
            assert!(
                !head.contains("spawn-time prompt"),
                "the old agent's prompt is gone from the head"
            );
            assert!(
                conv.iter()
                    .any(|item| item.text_content().contains("please find the bug")),
                "the user message survives"
            );
            assert!(
                conv.iter().any(|item| item.text_content() == "looking now"),
                "the assistant reply survives"
            );
            assert!(
                conv.iter()
                    .any(|item| item.text_content().contains("keep me")),
                "earlier system reminders are history and must not be stripped"
            );
            assert!(
                conv.iter()
                    .any(|item| item.text_content().contains("startup prefix")),
                "the original startup prefix is left verbatim"
            );

            let plan_path = actor.plan_mode.lock().plan_file_path().to_path_buf();
            let plan_resource = after_bridge
                .toolset()
                .get_resource_cloned::<xai_grok_tools::types::resources::PlanFilePath>()
                .await
                .expect("plan file path resource on the rebuilt bridge");
            assert_eq!(
                plan_resource.0, plan_path,
                "the plan file resource follows the new harness"
            );

            let model_after = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("sampling config survives the switch");
            assert_eq!(model_after.model, model_before.model, "model is preserved");
            assert_eq!(
                model_after.reasoning_effort, model_before.reasoning_effort,
                "effort is preserved"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn switch_is_refused_while_a_turn_runs() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let (actor, _gateway_rx) = build_actor().await;
            actor.chat_state_handle.replace_conversation(vec![
                ConversationItem::system("spawn-time prompt"),
                ConversationItem::user("hi"),
            ]);
            let before_name = actor.agent.borrow().definition().name.clone();
            actor.state.lock().await.running_task = Some(running_task_stub("running"));

            let err = actor
                .handle_switch_agent_definition(
                    xai_grok_agent::AgentDefinition::explore(),
                    xai_grok_agent::DEFAULT_SYSTEM_PROMPT_LABEL.to_owned(),
                )
                .await
                .expect_err("a running turn must refuse the switch");

            let data = err.data.expect("structured refusal data");
            assert_eq!(data["code"], "agent_busy");
            assert_eq!(
                data["message"].as_str(),
                Some(AGENT_SWITCH_TURN_BUSY_REFUSAL)
            );
            assert_eq!(
                actor.agent.borrow().definition().name,
                before_name,
                "the refused switch leaves the harness untouched"
            );
        })
        .await;
}

#[tokio::test(flavor = "current_thread")]
async fn switch_applies_an_agent_pinned_model_without_a_restart() {
    tokio::task::LocalSet::new()
        .run_until(async {
            let (actor, _gateway_rx) = build_actor().await;
            let mut definition = xai_grok_agent::AgentDefinition::explore();
            definition.model =
                xai_grok_agent::config::ModelOverride::Override("deepseek-v4-pro".to_owned());

            let switched = actor
                .handle_switch_agent_definition(
                    definition,
                    xai_grok_agent::DEFAULT_SYSTEM_PROMPT_LABEL.to_owned(),
                )
                .await
                .expect("an agent that pins a model switches in place");
            assert_eq!(switched, "explore");

            let cfg = actor
                .chat_state_handle
                .get_sampling_config()
                .await
                .expect("test actor has a sampling config");
            assert_eq!(
                cfg.model, "deepseek-v4-pro",
                "the pinned model is applied to the live session"
            );
        })
        .await;
}
