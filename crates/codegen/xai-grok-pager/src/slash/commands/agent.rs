//! `/agent`: pick an agent interactively, or switch straight to a named one.
//!
//! The shell can swap a session's agent in place (`deepseek-build/session/switch_agent`), so the
//! switch keeps the UI, model, permissions, and history. The pager falls back to a relaunch
//! (`--resume <id> --agent <name>`) when the shell refuses or does not implement the method.
//! Either way it refuses while a turn is running or cancelling so the swap never races a turn.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand, slash_meta};

/// Switch the session's agent (or open the picker).
pub struct AgentCommand;

impl SlashCommand for AgentCommand {
    slash_meta! {
        name: "agent",
        aliases: ["agents"],
        description: "Switch the active agent (in place when the shell supports it)",
        usage: "/agent [name]",
        takes_args: true,
        session_scoped: true,
        arg_placeholder: "<name>",
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let name = args.trim();
        if name.is_empty() {
            CommandResult::Action(Action::OpenAgentPicker)
        } else {
            CommandResult::Action(Action::SwitchAgent(name.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::model_state::ModelState;
    use crate::slash::command::CommandResult;

    #[test]
    fn bare_agent_opens_the_picker() {
        let models = ModelState::default();
        let mut ctx = crate::slash::commands::tests::make_ctx(&models);
        let result = AgentCommand.run(&mut ctx, "");
        assert!(
            matches!(result, CommandResult::Action(Action::OpenAgentPicker)),
            "bare /agent must open the picker, got {result:?}"
        );
    }

    #[test]
    fn named_agent_requests_a_switch() {
        let models = ModelState::default();
        let mut ctx = crate::slash::commands::tests::make_ctx(&models);
        let result = AgentCommand.run(&mut ctx, " explore ");
        match result {
            CommandResult::Action(Action::SwitchAgent(name)) => assert_eq!(name, "explore"),
            other => panic!("expected SwitchAgent, got {other:?}"),
        }
    }
}
