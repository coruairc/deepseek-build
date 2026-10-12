//! Captures the exact request headers the sampler sends to the model provider
//! in a real headless run against the local mock server, and guards the wire
//! against pre-rebrand identifiers.
//!
//! Run with `--nocapture` to print the full header list.

use xai_grok_test_support::{
    MockInferenceServer, MockModelEntry, assert_headless_success, run_headless,
};

#[tokio::test]
async fn headless_chat_headers_are_brand_clean() {
    let server =
        MockInferenceServer::start_with_models(vec![MockModelEntry::new("deepseek-v4-pro")])
            .await
            .expect("start mock inference server");
    server.set_response("hello from the mock server");

    let cwd = std::env::temp_dir().join(format!(
        "deepseek-build-header-capture-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&cwd).expect("create scratch cwd");

    let out = run_headless(
        &server,
        &["-p", "say hello", "--yolo", "--output-format", "json"],
        &cwd,
    )
    .await;
    assert_headless_success(&out, "header capture headless run", Some(&server));

    server.wait_for_inference_requests(1).await;
    let requests = server.requests();
    let inference = requests
        .iter()
        .find(|entry| entry.path.contains("/chat/completions"))
        .expect("sampler chat/completions request captured");
    assert!(
        !inference.headers.is_empty(),
        "no request headers captured for {inference:?}"
    );

    println!(
        "full sampler request headers ({}):",
        inference.headers.len()
    );
    for (name, value) in &inference.headers {
        if name.eq_ignore_ascii_case("authorization") {
            println!("  {name}: <redacted>");
        } else {
            println!("  {name}: {value}");
        }
    }

    let mut checked = 0;
    for (name, value) in &inference.headers {
        checked += 1;
        let name_lc = name.to_ascii_lowercase();
        for banned in ["x-xai", "x-grok", "x-authenticate"] {
            assert!(
                !name_lc.starts_with(banned),
                "legacy header name `{name}` sent to the provider"
            );
        }
        let value_lc = value.to_ascii_lowercase();
        assert!(
            !value_lc.contains("xai-grok-cli"),
            "legacy token in header `{name}`"
        );
        assert!(
            !value_lc.contains("grok"),
            "legacy product name in header `{name}`: {value}"
        );
    }
    if let Some(auth) = &inference.authorization {
        let auth_lc = auth.to_ascii_lowercase();
        assert!(
            !auth_lc.contains("xai-grok-cli") && !auth_lc.contains("grok"),
            "legacy token in the Authorization header"
        );
    }
    assert!(checked > 0, "no headers were checked");
}
