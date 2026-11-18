//! Release checks and verified in-place updates through Nodalia's shared updater, `nodalia-update`:
//! it finds Ánima's own releases, checks each package's SHA-256, and replaces the AppImage or
//! `Anima.app`, or installs the MSI on Windows. Other installs are pointed at the release page.
use nodalia_update::{App, Check, Failure, Release};
use std::sync::mpsc::{Receiver, TryRecvError, channel};

// i18n-exempt: stable release identity, not interface text.
pub const APP: App = App { name: "Anima", repo: "getnodalia/anima", bundle: "Anima.app", version: VERSION, edition: None };
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where every release is listed.
fn releases_url() -> String {
    format!("https://github.com/{}/releases", APP.repo)
}

/// Whether the update can be installed from inside the app (an AppImage, `Anima.app` or the MSI).
fn installs_in_place(release: &Release) -> bool {
    release.asset.is_some() && nodalia_update::install_kind(&APP).can_apply_in_place()
}

/// Whether an update may close the editor without losing pending document work.
pub(crate) fn safe_to_install(app: &crate::app::App) -> bool {
    !app.s.is_dirty()
        && !app.s.in_gesture()
        && !app.playing
        && app.text_edit.is_none()
        && app.pen.is_none()
        && app.pencil.is_empty()
        && app.brush_samples.is_empty()
        && app.raw_touched.is_empty()
        && app.drag == crate::app::Drag::None
        && app.motion_drag.is_none()
        && app.tl_drag == crate::timeline::TlDrag::None
        && app.me_drag.is_none()
        && app.plugin_dialog.is_none()
        && app.envelope.is_none()
}

enum Reply {
    Checked(Result<Option<Release>, String>, bool),
    Installed(Result<(), String>),
}
#[derive(Default)]
pub struct Updates {
    open: bool,
    read_outcome_requested: bool,
    requested: bool,
    message: String,
    release: Option<Release>,
    installable: bool,
    worker: Option<Receiver<Reply>>,
    installing: bool,
    /// The new version is starting: this process closes and exits.
    restart: bool,
}
impl Updates {
    /// Read the previous update result when the first desktop frame can show it.
    pub fn startup(&mut self) {
        self.read_outcome_requested = true;
    }

    /// Queue a check from the command-line panel selector.
    pub fn request(&mut self) {
        self.requested = true;
    }

    /// Start a manual check off the UI thread; repeated requests share the running check.
    pub fn check(&mut self, ctx: &egui::Context) {
        self.check_using(ctx, || {
            let result = nodalia_update::check(&APP).map_err(|e| e.to_string()).map(|check| match check {
                Check::Available(release) => Some(release),
                Check::UpToDate => None,
            });
            let installable = result.as_ref().ok().and_then(Option::as_ref).is_some_and(installs_in_place);
            (result, installable)
        });
    }

    fn check_using(&mut self, ctx: &egui::Context, check: impl FnOnce() -> (Result<Option<Release>, String>, bool) + Send + 'static) {
        self.open = true;
        if self.worker.is_some() {
            return;
        }
        self.message = tr!("Checking for updates…").into();
        self.release = None;
        self.installable = false;
        let (tx, rx) = channel();
        self.worker = Some(rx);
        let ctx = ctx.clone();
        let result = std::thread::Builder::new().name("release-check".into()).spawn(move || {
            let (result, installable) = check();
            let _ = tx.send(Reply::Checked(result, installable));
            ctx.request_repaint();
        });
        if let Err(error) = result {
            self.worker = None;
            self.message = error.to_string();
        }
    }
    fn install(&mut self, ctx: &egui::Context, release: Release) {
        self.install_using(ctx, move || {
            let package = nodalia_update::download(&APP, &release, |_, _| {}).map_err(|e| e.to_string())?;
            nodalia_update::apply(&APP, &package, || Ok(())).map_err(|e| e.to_string())
        });
    }

    fn install_using(&mut self, ctx: &egui::Context, install: impl FnOnce() -> Result<(), String> + Send + 'static) {
        self.installing = true;
        self.message = tr!("Downloading and verifying the update…").into();
        let (tx, rx) = channel();
        self.worker = Some(rx);
        let ctx = ctx.clone();
        let result = std::thread::Builder::new().name("release-install".into()).spawn(move || {
            let result = install();
            let _ = tx.send(Reply::Installed(result));
            ctx.request_repaint();
        });
        if let Err(error) = result {
            self.worker = None;
            self.installing = false;
            self.message = error.to_string();
        }
    }
    /// Returns true while installing: the caller pauses editing and starting work until restart.
    pub fn show(&mut self, ctx: &egui::Context, safe_to_install: bool) -> bool {
        if std::mem::take(&mut self.requested) {
            self.check(ctx);
        }
        if std::mem::take(&mut self.read_outcome_requested) {
            self.apply_outcome(nodalia_update::take_last_failure(&APP));
        }
        let reply = self.worker.as_ref().map(Receiver::try_recv);
        match reply {
            Some(Ok(Reply::Checked(result, installable))) => {
                self.installable = installable;
                self.worker = None;
                self.checked(result);
            }
            Some(Ok(Reply::Installed(result))) => {
                self.worker = None;
                match result {
                    Ok(()) => {
                        self.restart = true;
                        self.message = tr!("Update ready. Restarting…").into();
                    }
                    Err(error) => {
                        self.installing = false;
                        self.message = error;
                    }
                }
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.worker = None;
                self.installing = false;
                self.message = tr!("The update worker stopped unexpectedly. Try checking again.").into();
            }
            _ => {}
        }
        if self.restart {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if self.installing && ctx.input(|i| i.viewport().close_requested()) {
            // The platform helper may be replacing files. Keep this process alive until it finishes.
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.worker.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if self.open {
            let mut open = true;
            let mut install = false;
            let mut again = false;
            crate::windows::new(ctx, tr!("Check for Updates"))
                // NDL: its own header (the title and the round close button) instead of egui's title bar.
                .title_bar(false)
                .open(&mut { open })
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .default_width(440.0)
                .max_width((ctx.content_rect().width() - 32.0).max(160.0))
                .show(ctx, |ui| {
                    if crate::theme::dialog_header(ui, Some(&*tr!("Check for Updates"))) {
                        open = false;
                    }
                    ui.label(tr!("Ánima {version}", version = VERSION));
                    ui.label(&self.message);
                    if !safe_to_install && self.release.is_some() {
                        ui.label(tr!("Save your work and stop running tasks before installing an update."));
                    }
                    if self.release.is_some() && !self.installable {
                        ui.label(tr!("Use Releases or your package manager to update this installation."));
                    }
                    ui.horizontal(|ui| {
                        if let Some(release) = &self.release {
                            ui.hyperlink_to(tr!("Releases"), &release.page);
                            install = ui
                                .add_enabled(
                                    safe_to_install && self.installable && self.worker.is_none() && !self.restart,
                                    egui::Button::new(tr!("Update and restart")),
                                )
                                .clicked();
                        } else {
                            ui.hyperlink_to(tr!("Releases"), releases_url());
                        }
                        again = ui.add_enabled(self.worker.is_none() && !self.installing, egui::Button::new(tr!("Check again"))).clicked();
                    });
                });
            self.open = open || self.installing;
            if install && let Some(release) = self.release.clone() {
                self.install(ctx, release);
            }
            if again {
                self.check(ctx);
            }
        }
        self.installing
    }
    /// What the last in-place update left behind: nothing after a good one, or why it failed.
    fn apply_outcome(&mut self, failure: Option<Failure>) {
        let Some(failure) = failure else { return };
        self.open = true;
        self.message = match failure {
            Failure::StillRunning => tr!("The update was not installed: Ánima did not close in time.").into(),
            Failure::Cancelled => tr!("The update was cancelled. Nothing was changed.").into(),
            Failure::Replace => {
                tr!("macOS did not allow replacing Ánima. Allow it in System Settings › Privacy & Security › App Management, then try again.").into()
            }
            _ => tr!("The update was not installed. The installed version still works; download the new one from Releases.").into(),
        };
    }

    fn checked(&mut self, result: Result<Option<Release>, String>) {
        match result {
            Ok(Some(release)) => {
                self.message = tr!("{version} is available.", version = release.version);
                self.release = Some(release);
            }
            Ok(None) => {
                self.message = tr!("You're up to date.").into();
                self.release = None;
            }
            Err(error) => {
                self.message = tr!("Couldn't check for updates: {error}", error = error);
                self.release = None;
            }
        }
    }
    /// The app's exit path accepted closing. When a new version is starting, make sure this one
    /// is gone soon even if something keeps the window open.
    pub fn finish(&mut self) {
        if self.restart {
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_secs(5));
                std::process::exit(0);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draw(updates: &mut Updates, safe: bool, close: bool) -> egui::FullOutput {
        let ctx = egui::Context::default();
        let mut input =
            egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640.0, 400.0))), ..Default::default() };
        if close {
            input.viewports.entry(egui::ViewportId::ROOT).or_default().events.push(egui::ViewportEvent::Close);
        }
        let mut output = ctx.run_ui(input, |ui| {
            updates.show(ui.ctx(), safe);
        });
        output.textures_delta.clear();
        output
    }

    fn release(version: &str) -> Release {
        Release { tag: format!("v{version}"), version: version.into(), notes: String::new(), page: releases_url(), asset: None }
    }

    #[test]
    fn the_releases_are_animas_own() {
        assert_eq!(APP.repo, "getnodalia/anima");
        assert_eq!(APP.bundle, "Anima.app");
        assert_eq!(APP.version, env!("CARGO_PKG_VERSION"));
        assert!(!installs_in_place(&release("99.0.0")), "a release without a package for this system installs nothing");
    }

    /// The names the packaging scripts give the packages, and the systems whose updater must find
    /// each one. A rename in `packaging/` that the updater does not know about would leave those
    /// users on the old version without a word, so it fails here instead.
    #[test]
    fn every_published_package_is_found_by_the_updater_it_is_for() {
        let read = |path: &str| std::fs::read_to_string(format!("{}/packaging/{path}", env!("CARGO_MANIFEST_DIR"))).expect("packaging script");
        let macos = read("macos/package.sh");
        assert!(macos.contains(r#"APP="$WORK/Anima.app""#), "the bundle the updater swaps");
        let wxs = read("windows/anima.wxs");
        assert!(wxs.contains("InstallDir"), "the updater finds the MSI install through its InstallDir value");
        for (package, systems) in [
            ("anima-1.0.0-macos-universal.dmg", &[("macos", "aarch64"), ("macos", "x86_64")][..]),
            ("anima-1.0.0-linux-x86_64.AppImage", &[("linux", "x86_64")]),
            ("anima-1.0.0-linux-aarch64.AppImage", &[("linux", "aarch64")]),
            ("anima-1.0.0-windows-x64.msi", &[("windows", "x86_64")]),
            ("anima-1.0.0-windows-arm64.msi", &[("windows", "aarch64")]),
        ] {
            let found = nodalia_update::systems_for_package(package);
            for system in systems {
                assert!(found.contains(system), "{package} is not offered to {system:?}");
            }
        }
    }

    #[test]
    fn available_release_is_shown_but_never_installed_without_confirmation() {
        let (tx, rx) = channel();
        tx.send(Reply::Checked(Ok(Some(release("99.0.0"))), false)).expect("receiver is alive");
        let mut updates = Updates { open: true, worker: Some(rx), ..Updates::default() };
        let output = draw(&mut updates, false, false);
        assert!(updates.message.contains("99.0.0"));
        assert!(!output.shapes.is_empty(), "the result window must be rendered");
        assert!(updates.worker.is_none());
        assert!(!updates.installing);
        assert!(!updates.installable, "externally managed installs must only offer Releases");
        updates.installable = true;
        draw(&mut updates, true, false);
        assert!(!updates.installing, "an available update alone must never install");
    }

    #[test]
    fn checks_run_off_thread_and_repeated_requests_share_the_worker() {
        let ctx = egui::Context::default();
        let (started, wait) = channel();
        let mut updates = Updates::default();
        updates.check_using(&ctx, move || {
            wait.recv().expect("release test worker");
            (Ok(None), false)
        });
        assert!(updates.worker.is_some());
        updates.check_using(&ctx, || panic!("must not start a second check"));
        draw(&mut updates, false, false);
        started.send(()).expect("worker waiting");
        let reply = updates.worker.as_ref().expect("worker").recv_timeout(std::time::Duration::from_secs(5)).expect("worker finishes");
        let (tx, rx) = channel();
        tx.send(reply).expect("receiver");
        updates.worker = Some(rx);
        draw(&mut updates, true, false);
        assert_eq!(updates.message, "You're up to date.");
        assert!(updates.worker.is_none());
    }

    #[test]
    fn an_install_failure_unlocks_editing_and_close_is_cancelled_until_it_finishes() {
        let ctx = egui::Context::default();
        let (finish, wait) = channel();
        let mut updates = Updates { open: true, ..Updates::default() };
        updates.install_using(&ctx, move || {
            wait.recv().expect("release test worker");
            Err("The downloaded update failed its SHA-256 check.".into())
        });
        assert!(updates.installing);
        let output = draw(&mut updates, false, true);
        assert!(output.viewport_output.values().any(|v| v.commands.contains(&egui::ViewportCommand::CancelClose)));
        finish.send(()).expect("worker waiting");
        let reply = updates.worker.as_ref().expect("worker").recv_timeout(std::time::Duration::from_secs(5)).expect("worker finishes");
        let (tx, rx) = channel();
        tx.send(reply).expect("receiver");
        updates.worker = Some(rx);
        draw(&mut updates, false, false);
        assert!(!updates.installing);
        assert!(updates.worker.is_none());
        assert!(!updates.restart, "failed verification must never restart");
        assert!(updates.message.contains("SHA-256"));
        updates.finish();
        updates.apply_outcome(None);
        assert!(!updates.open || updates.message.contains("SHA-256"), "no outcome leaves the window as it was");
    }

    #[test]
    fn a_finished_install_closes_the_window_to_restart() {
        let (tx, rx) = channel();
        tx.send(Reply::Installed(Ok(()))).expect("receiver is alive");
        let mut updates = Updates { open: true, installing: true, worker: Some(rx), ..Updates::default() };
        let output = draw(&mut updates, true, false);
        assert!(updates.restart);
        assert!(output.viewport_output.values().any(|v| v.commands.contains(&egui::ViewportCommand::Close)));
    }

    #[test]
    fn uncommitted_stage_text_and_pen_paths_prevent_installation() {
        let ctx = egui::Context::default();
        let session = anima_engine::Session::new(anima_engine::doc::sample::demo());
        let mut app = crate::app::App::with_context(&ctx, session, Some(std::path::PathBuf::from("unused.png")));
        assert!(!app.s.is_dirty());
        assert!(safe_to_install(&app));
        let mut te = crate::app::TextEditing::new(egui::Pos2::ZERO, None, None, &app.s.text_style);
        te.text = "Uncommitted stage text".into();
        te.focus = false;
        app.text_edit = Some(te);
        assert!(!app.s.is_dirty(), "the edit has not reached Session yet");
        assert!(!safe_to_install(&app), "restarting would discard the text buffer");
        app.text_edit = None;
        app.pen = Some(crate::pathtools::PenPath { knots: vec![[anima_engine::geom::Vec2::ZERO; 3]], dragging: false });
        assert!(!app.s.is_dirty());
        assert!(!safe_to_install(&app), "an unfinished Pen path is also pending work");
        app.pen = None;
        assert!(safe_to_install(&app));
        app.s.begin_gesture("pending edit");
        assert!(!safe_to_install(&app), "an engine gesture must finish first");
    }

    #[test]
    fn a_failed_update_is_reported_when_the_old_app_starts_again() {
        let mut updates = Updates::default();
        updates.apply_outcome(Some(Failure::Cancelled));
        assert!(updates.open);
        assert!(updates.message.contains("cancelled"));
        assert!(updates.worker.is_none());
        assert!(!updates.installing);
        let mut other = Updates::default();
        other.apply_outcome(Some(Failure::Mount));
        assert!(other.message.contains("Releases"), "an unknown failure points at the release page");
    }

    #[test]
    fn a_manual_check_reports_success_and_failure() {
        let mut updates = Updates::default();
        updates.checked(Ok(None));
        assert_eq!(updates.message, "You're up to date.");
        updates.checked(Err("network unavailable".into()));
        assert!(updates.message.contains("network unavailable"));
        assert!(updates.release.is_none());
    }

    #[test]
    fn a_dropped_worker_reports_failure_and_unblocks_the_app() {
        let (tx, rx) = channel();
        drop(tx);
        let mut updates = Updates { worker: Some(rx), installing: true, ..Updates::default() };
        assert!(!updates.show(&egui::Context::default(), false));
        assert!(updates.message.contains("stopped unexpectedly"));
    }
}
