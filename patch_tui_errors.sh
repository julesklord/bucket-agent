sed -i '5532d' crates/codegen/bucket-tui/src/app/app_view.rs
sed -i '249d' crates/codegen/bucket-tui/src/app/dispatch/tests/mod.rs
sed -i 's/CommandExecCtx::mock()/crate::slash::commands::tests::make_ctx(\&crate::slash::ModelState::default())/g' crates/codegen/bucket-tui/src/slash/commands/feedback.rs
sed -i 's/let mut ctx = crate::slash::commands::tests::make_ctx(\&crate::slash::ModelState::default());/let model_state = crate::slash::ModelState::default();\n        let mut ctx = crate::slash::commands::tests::make_ctx(\&model_state);/g' crates/codegen/bucket-tui/src/slash/commands/feedback.rs
