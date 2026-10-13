use std::io::Write;
use std::process::{Command, Stdio};

/// OS のクリップボードコマンドにテキストを渡してコピーする
pub fn copy(text: &str) -> Result<(), String> {
    let candidates: &[(&str, &[&str])] = if cfg!(target_os = "macos") {
        &[("pbcopy", &[])]
    } else {
        &[("wl-copy", &[]), ("xclip", &["-selection", "clipboard"]), ("xsel", &["--clipboard", "--input"])]
    };

    for (cmd, args) in candidates {
        let Ok(mut child) = Command::new(cmd)
            .args(*args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            continue;
        };
        child
            .stdin
            .take()
            .expect("stdin is piped")
            .write_all(text.as_bytes())
            .map_err(|e| format!("{}: {}", cmd, e))?;
        let status = child.wait().map_err(|e| format!("{}: {}", cmd, e))?;
        return if status.success() {
            Ok(())
        } else {
            Err(format!("{} exited with {}", cmd, status))
        };
    }
    Err("no clipboard command found".to_string())
}
