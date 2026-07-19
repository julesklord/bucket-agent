//! `/feedback` -- send session feedback.

use crate::app::actions::Action;
use crate::slash::command::{CommandExecCtx, CommandResult, SlashCommand};

pub const FEEDBACK_ISSUE_URL: &str =
    "https://github.com/julesklord/bucket-agent/issues/new?template=feedback.md";

/// Open GitHub issue template to send feedback or report a bug.
pub struct FeedbackCommand;

impl SlashCommand for FeedbackCommand {
    fn name(&self) -> &str {
        "feedback"
    }

    fn description(&self) -> &str {
        "Open GitHub issue to send feedback or report a bug"
    }

    fn usage(&self) -> &str {
        "/feedback [text]"
    }

    fn takes_args(&self) -> bool {
        true
    }

    fn args_required(&self) -> bool {
        false
    }

    fn arg_placeholder(&self) -> Option<&str> {
        Some("[feedback text]")
    }

    fn run(&self, _ctx: &mut CommandExecCtx, args: &str) -> CommandResult {
        let trimmed = args.trim();
        let url = if trimmed.is_empty() {
            FEEDBACK_ISSUE_URL.to_string()
        } else {
            format!(
                "{}&title={}",
                FEEDBACK_ISSUE_URL,
                urlencoding::encode(trimmed)
            )
        };
        CommandResult::Action(Action::OpenUrl(url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_command_opens_issue_template() {
        let cmd = FeedbackCommand;
        let mut ctx = CommandExecCtx::mock();
        let res = cmd.run(&mut ctx, "");
        if let CommandResult::Action(Action::OpenUrl(url)) = res {
            assert_eq!(url, FEEDBACK_ISSUE_URL);
        } else {
            panic!("Expected Action::OpenUrl");
        }
    }

    #[test]
    fn test_feedback_command_prefills_title() {
        let cmd = FeedbackCommand;
        let mut ctx = CommandExecCtx::mock();
        let res = cmd.run(&mut ctx, "Bug in terminal render");
        if let CommandResult::Action(Action::OpenUrl(url)) = res {
            assert!(url.starts_with(FEEDBACK_ISSUE_URL));
            assert!(url.contains("Bug%20in%20terminal%20render"));
        } else {
            panic!("Expected Action::OpenUrl");
        }
    }
}
