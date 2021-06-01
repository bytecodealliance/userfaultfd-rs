#[test]
fn run_manpage_example() {
    #[cfg(feature = "linux5_11")]
    let args = [
        "run",
        "--example",
        "manpage",
        "--features",
        "linux5_11",
        "--",
        "3",
    ];
    #[cfg(not(feature = "linux5_11"))]
    let args = ["run", "--example", "manpage", "--", "3"];
    let output = std::process::Command::new("cargo")
        .args(&args)
        .output()
        .expect("manpage example failed to start");
    assert!(output.status.success(), "manpage example succeeded");
}
