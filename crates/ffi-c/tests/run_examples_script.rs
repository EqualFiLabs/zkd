#![cfg(unix)]

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn write_executable(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents)?;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[test]
fn run_examples_clang_handles_empty_windows_flags_on_darwin() -> Result<()> {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("expected crates/ parent directory")?
        .parent()
        .context("expected workspace root")?
        .to_path_buf();

    let temp = TempDir::new().context("create temp dir")?;
    let bin_dir = temp.path().join("bin");
    fs::create_dir(&bin_dir).context("create stub bin directory")?;

    let cargo_stub = bin_dir.join("cargo");
    write_executable(
        &cargo_stub,
        r#"#!/usr/bin/env bash
set -euo pipefail
target_dir="${CARGO_TARGET_DIR:?}"
mkdir -p "$target_dir/release"
touch "$target_dir/release/libzkprov.dylib"
"#,
    )?;

    let clang_stub = bin_dir.join("clang");
    write_executable(
        &clang_stub,
        r#"#!/usr/bin/env bash
set -euo pipefail
output=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    -o)
      shift
      output="${1:-}"
      ;;
    -lws2_32|-luserenv|-lntdll)
      echo "unexpected windows linker flag: $1" >&2
      exit 1
      ;;
  esac
  shift || break
done
if [[ -z "$output" ]]; then
  echo "clang stub missing -o output" >&2
  exit 1
fi
mkdir -p "$(dirname "$output")"
cat <<'EOF' > "$output"
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$output"
"#,
    )?;

    let uname_stub = bin_dir.join("uname");
    write_executable(
        &uname_stub,
        r#"#!/usr/bin/env bash
if [[ "${1:-}" == "-s" ]]; then
  echo "Darwin"
else
  echo "Darwin"
fi
"#,
    )?;

    let mut path_entries = Vec::new();
    path_entries.push(bin_dir.clone());
    if let Some(existing) = env::var_os("PATH") {
        path_entries.extend(env::split_paths(&existing));
    }
    let path_value = env::join_paths(path_entries).context("join PATH entries")?;
    let target_dir = temp.path().join("target");

    let output = Command::new(project_root.join("scripts/run_examples.sh"))
        .arg("c")
        .env("PATH", &path_value)
        .env("CARGO", &cargo_stub)
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()
        .context("run examples script")?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "examples script failed\nstatus: {}\nstdout: {}\nstderr: {}",
            output.status,
            stdout,
            stderr
        );
    }

    let expected_exe = target_dir.join("examples").join("roundtrip_c");
    assert!(
        expected_exe.exists(),
        "expected stub clang to produce {}",
        expected_exe.display()
    );

    Ok(())
}
