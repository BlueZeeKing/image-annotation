use std::process::Command;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let mut command = Command::new("npx");

    command.args([
        "tailwindcss",
        "-i",
        "assets/styles.css",
        "-o",
        "public/styles.css",
    ]);

    if profile == "release" {
        command.arg("-m");
    }

    command.output().unwrap();

    println!("cargo::rerun-if-changed=assets/styles.css");
    println!("cargo::rerun-if-changed=templates/");
}
