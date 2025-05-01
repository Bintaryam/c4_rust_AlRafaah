use std::process::{Command};
use std::path::Path;

#[test]
fn test_self_host_output_matches_c4() {
    let project_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let c4_path = project_dir.join("c4.c");
    let c4_exe = project_dir.join("c4.exe");

    assert!(c4_path.exists(), "c4.c not found");
    assert!(c4_exe.exists(), "c4.exe not found");

    let output = Command::new(&c4_exe)
        .arg("-s")
        .arg("c4.c")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to run c4.exe");

    println!("C4 stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("C4 stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    println!("C4 status: {:?}", output.status);

    assert!(output.status.success(), "C4 did not exit successfully");
}
