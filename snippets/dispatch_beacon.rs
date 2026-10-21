//! Labyrica Dispatch beacon.
//!
//! Copy into src-tauri/src/, set ID and NAME, call `dispatch_beacon::write(app.handle())`
//! from `.setup(...)`. Needs serde_json.
//!
//! On launch, write `<local data>/Labyrica/apps/<ID>.json` so Labyrica
//! Dispatch can find this install, see the running version, and close it
//! before updating. Contract: labyrica-dispatch `docs/BEACON.md`.
//! Best effort: every error is ignored.

use serde_json::json;
use tauri::{AppHandle, Manager};

const ID: &str = "myapp"; // the app's id in the Dispatch catalog
const NAME: &str = "My App";

pub fn write(app: &AppHandle) {
    let Ok(base) = app.path().local_data_dir() else { return };
    let dir = base.join("Labyrica").join("apps");
    let exe = std::env::current_exe().ok();

    let mut beacon = json!({
        "schema": 1,
        "id": ID,
        "name": NAME,
        "version": app.package_info().version.to_string(),
        "executable": exe,
        "pid": std::process::id(),
    });
    // macOS: the enclosing .app, if we're running from one.
    if cfg!(target_os = "macos") {
        let bundle = exe.as_deref().and_then(|e| {
            e.ancestors().find(|p| p.extension().is_some_and(|x| x == "app"))
        });
        if let Some(b) = bundle {
            beacon["bundle"] = json!(b);
        }
    }

    let _ = std::fs::create_dir_all(&dir);
    let tmp = dir.join(format!("{ID}.json.tmp"));
    if std::fs::write(&tmp, beacon.to_string()).is_ok() {
        let _ = std::fs::rename(&tmp, dir.join(format!("{ID}.json")));
    }
}
