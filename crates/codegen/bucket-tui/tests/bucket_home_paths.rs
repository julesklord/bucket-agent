//! `BUCKET_HOME` override tests in an isolated binary so `bucket_home()`'s
//! process-wide `OnceLock` initializes from the overridden env var.

use std::path::PathBuf;

#[test]
fn bucket_home_override_path_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bucket_home = tmp.path().to_path_buf();
    unsafe {
        std::env::set_var("BUCKET_HOME", &bucket_home);
    }

    assert_eq!(
        bucket_tui::util::pager_toml_path(),
        bucket_home.join("pager.toml")
    );
    assert_eq!(
        bucket_tui::util::display_bucket_home_prefix(),
        "$BUCKET_HOME"
    );
    assert_eq!(
        bucket_tui::util::display_user_bucket_path("config.toml"),
        "$BUCKET_HOME/config.toml"
    );

    let memory_path = bucket_home.join("memory/MEMORY.md");
    assert_eq!(
        bucket_tui::util::abbreviate_path(&memory_path.display().to_string()),
        "$BUCKET_HOME/memory/MEMORY.md"
    );

    assert!(bucket_tui::util::is_under_user_bucket_home(&memory_path));
    assert!(!bucket_tui::util::is_under_user_bucket_home(
        PathBuf::from("/tmp/other").as_path()
    ));
}
