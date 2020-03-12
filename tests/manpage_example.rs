#[test]
fn run_manpage_example() {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--example", "manpage", "--", "3"])
        .output()
        .expect("manpage example failed to start");
    assert!(output.status.success(), "manpage example succeeded");
}
