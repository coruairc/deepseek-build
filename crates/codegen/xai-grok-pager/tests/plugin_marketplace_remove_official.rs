//! Regression: CLI `marketplace remove` must remove a source known only via the
//! JSON store (`known_marketplaces.json`), not just one declared in `config.toml`.

#[test]
fn cli_remove_of_json_store_source_removes_it() {
    // One #[test] per binary: the env is process-global.
    let grok_home = tempfile::tempdir().expect("grok home");
    // SAFETY: no other threads are running yet.
    unsafe { std::env::set_var("GROK_HOME", grok_home.path()) };

    // The source is known ONLY via the JSON store.
    let plugins_dir = grok_home.path().join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();
    let known_path = plugins_dir.join("known_marketplaces.json");
    std::fs::write(
        &known_path,
        r#"{"xai": {"source": {"source": "github", "repo": "xai-org/plugin-marketplace"}}}"#,
    )
    .unwrap();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(xai_grok_pager::plugin_cmd::run(
            xai_grok_pager::plugin_cmd::PluginArgs {
                command: xai_grok_pager::plugin_cmd::PluginCommand::Marketplace(
                    xai_grok_pager::plugin_cmd::MarketplaceArgs {
                        command: xai_grok_pager::plugin_cmd::MarketplaceCommand::Remove {
                            source: "https://github.com/xai-org/plugin-marketplace.git".into(),
                        },
                    },
                ),
            },
        ))
        .expect("remove succeeds");

    let known: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&known_path).unwrap()).unwrap();
    assert!(
        known.get("xai").is_none(),
        "source must be removed from the JSON store: {known}"
    );
}
