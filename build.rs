use std::env;

fn main() {
    // Determine the build profile (debug or release)
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());

    // Get the current build timestamp
    let build_timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Set a different value for debug vs release
    let version = match profile.as_str() {
        "debug" => format!("DEBUG BUILD v{}", build_timestamp),
        "release" => format!("RELEASE BUILD v{}", build_timestamp),
        _ => format!("UNKNOWN BUILD ({}) v{}", profile, build_timestamp),
    };

    // Write the build timestamp/version to an environment variable
    println!("cargo:rustc-env=FALLBACK_APP_VERSION={}", version);

    // Optionally, generate a file with the timestamp if needed
    // let out_dir = env::var("OUT_DIR").unwrap();
    // let version_path = Path::new(&out_dir).join("build_version.txt");
    // fs::write(&version_path, build_timestamp).unwrap();
}
