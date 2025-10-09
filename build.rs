use std::env;

fn main() {
    // Only embed Windows resources when targeting Windows
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        // Embed Windows resource file with version info and manifest
        embed_resource::compile("resources/windows/build-it-agent.rc", embed_resource::NONE);
    }
}
