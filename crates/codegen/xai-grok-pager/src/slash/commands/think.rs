//! `/think`: toggle the agent's reasoning (thinking) blocks on or off across the transcript.
//!
//! This is the slash-command spelling of `Ctrl+E`. It flips the same process-wide appearance
//! toggle, so the two stay in sync and a session can hide reasoning without a config edit.
//! The persisted default is `[ui].show_thinking_blocks`.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand, slash_meta};

pub struct ThinkCommand;

impl SlashCommand for ThinkCommand {
    slash_meta! {
        name: "think",
        description: "Show/hide reasoning (thinking) blocks",
        usage: "/think",
        arg_placeholder: "on/off",
    }

    fn run(&self, _ctx: &mut CommandExecCtx, _args: &str) -> CommandResult {
        let new = !crate::appearance::cache::load_show_thinking_blocks();
        CommandResult::Action(Action::SetShowThinkingBlocks(new))
    }
}
