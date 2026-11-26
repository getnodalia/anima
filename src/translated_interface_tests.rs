//! Every screen draws its text in the interface language. The dialog or panel behind every menu
//! command is drawn in Spanish; a text that is still the English key of a Spanish catalog row was
//! drawn without going through the catalog.

use std::collections::{BTreeMap, BTreeSet};

use crate::app::App;
use egui::epaint::Shape;

/// Names of what the app ships or the walk makes, drawn as stored: a document, a layer, a symbol, a
/// swatch or an action is data the person can rename, so it is not interface text.
const DATA: &[&str] = &[
    // Layers of the sample document and the layer a command adds.
    "Camera", "Star",
];

/// Texts whose draw site is still to be found. Each is a bug, never a place to add to: remove one
/// when its site goes through `tr!`.
const KNOWN: &[&str] = &[];

fn collect_shape(shape: &Shape, out: &mut BTreeMap<String, String>) {
    match shape {
        Shape::Text(t) => {
            for line in t.galley.text().split('\n') {
                let line = line.trim();
                if !line.is_empty() {
                    out.entry(line.to_owned()).or_insert_with(|| "painted".to_owned());
                }
            }
        }
        Shape::Vec(v) => v.iter().for_each(|s| collect_shape(s, out)),
        _ => {}
    }
}

/// `n` frames; what the last one painted as text. `events` go in the first.
fn frames(ctx: &egui::Context, app: &mut App, n: usize, events: Vec<egui::Event>) -> BTreeMap<String, String> {
    let mut shown = BTreeMap::new();
    for i in 0..n {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.0, 940.0))),
            events: if i == 0 { events.clone() } else { Vec::new() },
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| app.frame(ui));
        out.textures_delta.clear();
        shown.clear();
        for clipped in &out.shapes {
            collect_shape(&clipped.shape, &mut shown);
        }
    }
    shown
}

fn escape(ctx: &egui::Context, app: &mut App) {
    let key = |pressed| egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: Some(egui::Key::Escape),
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    };
    for _ in 0..3 {
        frames(ctx, app, 2, vec![key(true), key(false)]);
    }
}

/// Texts of the frame that are English keys of a Spanish row with another reading.
fn untranslated(frame: &BTreeMap<String, String>) -> Vec<String> {
    frame
        .iter()
        .filter(|(t, _)| !DATA.contains(&t.as_str()) && !KNOWN.contains(&t.as_str()))
        .filter(|(t, _)| crate::i18n::t_dyn(t) != t.as_str())
        .map(|(t, kind)| format!("{t:?} ({kind})"))
        .collect()
}

/// Commands that leave the app, close the document the walk draws on, or open a system file dialog.
fn skipped(id: &str) -> bool {
    id.contains("quit") || id.contains("exit") || id.starts_with("file.close") || id.starts_with("window.")
}

#[test]
fn spanish_interface_has_no_untranslated_text() {
    let ctx = egui::Context::default();
    let session = anima_engine::Session::new(anima_engine::doc::sample::demo());
    let mut app = App::with_context(&ctx, session, Some(std::path::PathBuf::from("unused.png")));
    app.prefs.language = "es".into();
    app.apply_prefs(&ctx);
    // The catalogs are the optional build input NODALIA_I18N_DIR (nodalia-i18n): without them there
    // is no Spanish to check, as with the tests that need nodalia-fonts.
    if crate::i18n::code() != "es" {
        eprintln!("skipped: this build embeds no Spanish catalog (set NODALIA_I18N_DIR)");
        return;
    }
    // The welcome line was written at construction, in the language the person had saved; the app
    // under test was switched to Spanish after it.
    app.status_ok("");
    let mut failures: Vec<String> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut note = |what: &str, frame: &BTreeMap<String, String>| {
        for line in untranslated(frame) {
            if seen.insert(line.clone()) {
                failures.push(format!("{what}: {line}"));
            }
        }
    };
    let main = frames(&ctx, &mut app, 4, Vec::new());
    note("main window", &main);
    let ids: Vec<&'static str> =
        anima_engine::commands().iter().filter(|c| !c.menu.is_empty()).map(|c| c.id).collect::<BTreeSet<_>>().into_iter().collect();
    for id in ids {
        if skipped(id) {
            continue;
        }
        app.invoke(id);
        let frame = frames(&ctx, &mut app, 3, Vec::new());
        note(&format!("command {id}"), &frame);
        escape(&ctx, &mut app);
    }
    assert!(failures.is_empty(), "texts drawn in English although Spanish has them:\n{}", failures.join("\n"));
}
