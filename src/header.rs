//! The top bar (Nodalia's creator suite, 44 pt): the main menu (≡) with About, Preferences,
//! Keyboard Shortcuts and Quit, Home (the start screen), then the menus in the window when the
//! desktop shows none of its own, the document's name centred, and the workspace selector at the
//! right. On Linux with Nodalia's window controls ([`App::custom_titlebar`]) the bar is the
//! window's title bar too: its empty space drags the window and the round controls take its end.

use crate::app::App;
use crate::theme::{T, Type, font};
use egui::{Rect, Role, Sense, pos2, vec2};

/// The bar's height.
pub const HEIGHT: f32 = nodalia_theme::creator::layout::APP_BAR;
/// A bar button's side.
const BUTTON: f32 = 28.0;

/// A 28 pt icon button of the bar: Nodalia's icon `icon`, a rounded fill under the pointer.
fn icon_button(ui: &mut egui::Ui, rect: Rect, icon: &str, name: &str, selected: bool) -> egui::Response {
    let resp = ui.interact(rect, egui::Id::unique(("top_bar", icon)), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(Role::Button, true, selected, name));
    let fill = if selected {
        T.active
    } else if resp.hovered() {
        T.hover
    } else {
        egui::Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, nodalia_theme::component::CONTROL_RADIUS, fill);
    if let Some(glyph) = nodalia_theme::icons::get(icon) {
        let ink = if selected || resp.hovered() { T.icon_hot } else { T.icon };
        nodalia_theme::icons::paint(ui.painter(), rect.shrink(6.0), glyph, nodalia_theme::icons::Style::bold(ink));
    }
    crate::a11y::focus_ring(ui, &resp);
    resp
}

/// What the main menu offers.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Entry {
    About,
    Preferences,
    Shortcuts,
    Quit,
}

/// Nodalia's main menu: a hamburger at the start of the bar with the app's own entries.
fn main_menu(app: &mut App, ui: &mut egui::Ui, rect: Rect) {
    let button = icon_button(ui, rect, "menu", tr!("Main menu"), false);
    let mut chosen = None;
    egui::Popup::menu(&button).show(|ui| {
        ui.set_min_width(230.0);
        let entries = [
            (Entry::About, tr!("About Ánima"), ""),
            (Entry::Preferences, tr!("Preferences…"), if crate::platform::OS == crate::platform::Os::Mac { "Ctrl+," } else { "Ctrl+U" }),
            (Entry::Shortcuts, tr!("Keyboard Shortcuts"), "F1"),
            (Entry::Quit, tr!("Quit Ánima"), "Ctrl+Q"),
        ];
        for (entry, label, keys) in entries {
            if entry == Entry::Quit {
                ui.separator();
            }
            let keys = if keys.is_empty() { String::new() } else { crate::platform::shortcut_label(keys) };
            let row = ui.add(egui::Button::new(label).shortcut_text(keys).min_size(vec2(0.0, 26.0)));
            if row.clicked() {
                chosen = Some(entry);
                ui.close();
            }
        }
    });
    match chosen {
        Some(Entry::About) => app.show_about = true,
        Some(Entry::Preferences) => app.open_prefs(),
        Some(Entry::Shortcuts) => app.show_shortcuts = true,
        Some(Entry::Quit) => ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close),
        None => {}
    }
}

/// Draws the top bar in `rect` and handles its clicks.
pub fn show(app: &mut App, ui: &mut egui::Ui, rect: Rect) {
    let p = ui.painter().clone();
    // The bar is the window's frame around the islands: the window colour, no line under it.
    p.rect_filled(rect, 0.0, T.chrome);
    if app.custom_titlebar || crate::platform::OS == crate::platform::Os::Mac {
        crate::titlebar::drag_area(ui, rect);
    }
    let cy = rect.center().y;
    // macOS: the traffic lights sit over the bar's start.
    // Nodalia's window controls at the left (the desktop's choice on Linux) take the bar's first
    // stretch and the right end is free.
    let side = crate::titlebar::side(app);
    let controls_left = side == Some(nodalia_theme::titlebar::Side::Left);
    let mut x = rect.min.x
        + if controls_left {
            crate::titlebar::WIDTH + nodalia_theme::titlebar::CONTENT_GAP
        } else {
            crate::platform::menu_inset().max(crate::theme::ISLAND_GAP + 2.0)
        };
    let square = |x: f32| Rect::from_min_size(pos2(x, cy - BUTTON / 2.0), vec2(BUTTON, BUTTON));
    main_menu(app, ui, square(x));
    x += BUTTON + 4.0;
    let home = square(x);
    if icon_button(ui, home, "home", tr!("Home"), app.start.is_some()).on_hover_text(tr!("Home")).clicked() {
        if app.start.is_some() {
            app.start = None;
        } else {
            crate::start::open(app);
        }
    }
    x = home.max.x + 8.0;
    // The menus, unless the desktop shows them in its own bar.
    let mut left_end = x;
    if !crate::global_menu::shown() {
        let menu_rect = Rect::from_min_max(pos2(x, rect.min.y + 8.0), pos2(rect.max.x, rect.max.y - 8.0));
        let mut mu = ui.new_child(egui::UiBuilder::new().max_rect(menu_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
        {
            let v = mu.visuals_mut();
            v.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
            v.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
            v.widgets.inactive.bg_stroke = egui::Stroke::NONE;
            v.widgets.inactive.fg_stroke = egui::Stroke::new(1.5, T.text_dim);
        }
        left_end = crate::menus::bar(app, &mut mu);
    }
    // Right group, from the right: window controls, then the workspace selector.
    let mut rx =
        rect.max.x - if side == Some(nodalia_theme::titlebar::Side::Right) { crate::titlebar::WIDTH + 4.0 } else { crate::theme::ISLAND_GAP + 2.0 };
    if let Some(side) = side {
        crate::titlebar::window_controls(ui, rect, side);
    }
    workspace_selector(app, ui, &mut rx, cy);
    // The document's name, centred while the groups either side leave it room.
    let name = app.s.path.as_ref().and_then(|p| p.file_name()).map_or_else(|| tr!("Untitled").to_string(), |n| n.to_string_lossy().into_owned());
    let title = if app.s.is_dirty() { tr!("{name} — Edited", name = name) } else { name };
    let (left, right) = (left_end + 12.0, rx - 12.0);
    if right > left + 40.0 {
        let mut job = egui::text::LayoutJob::simple_singleline(title, font(Type::Body), T.text);
        job.wrap = egui::text::TextWrapping::truncate_at_width(right - left);
        let galley = p.layout_job(job);
        let half = galley.size().x / 2.0;
        let center = if rect.center().x - half >= left && rect.center().x + half <= right { rect.center().x } else { (left + right) / 2.0 };
        p.with_clip_rect(Rect::from_min_max(pos2(left, rect.min.y), pos2(right, rect.max.y))).galley(
            pos2(center - half, cy - galley.size().y / 2.0),
            galley,
            T.text,
        );
    }
}

/// The creator suite's workspace selector, right-aligned at `rx` (moved to its left edge):
/// Essentials, Animator, Designer and yours, then Reset, Save Workspace… and Manage Workspaces….
fn workspace_selector(app: &mut App, ui: &mut egui::Ui, rx: &mut f32, cy: f32) {
    let keys = app.prefs.workspaces.keys();
    let names: Vec<String> = keys.iter().map(|k| crate::i18n::t_dyn(crate::dock::Workspaces::name(k)).into_owned()).collect();
    let current_key = app.prefs.workspaces.current.clone();
    let current = crate::i18n::t_dyn(crate::dock::Workspaces::name(&current_key)).into_owned();
    let w = (ui.painter().layout_no_wrap(current.clone(), egui::FontId::proportional(13.0), T.text).size().x + 44.0).clamp(130.0, 200.0);
    let r = Rect::from_min_size(pos2(*rx - w, cy - 14.0), vec2(w, 28.0));
    *rx = r.min.x;
    let mut wu = ui.new_child(egui::UiBuilder::new().max_rect(r).layout(egui::Layout::left_to_right(egui::Align::Center)));
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let saved_from = crate::dock::BUILT_IN.len().min(keys.len());
    let actions = [tr!("Reset Workspace"), tr!("Save Workspace…"), tr!("Manage Workspaces…")];
    use nodalia_theme::creator::WorkspacePick;
    match nodalia_theme::creator::workspace_selector(&mut wu, &crate::theme::creator(), "anima", &current, &refs, saved_from, &actions, w) {
        Some(WorkspacePick::Workspace(i)) => {
            if let Some(k) = keys.get(i)
                && *k != current_key
            {
                app.switch_workspace(k);
            }
        }
        Some(WorkspacePick::Action(0)) => app.reset_workspace(),
        Some(WorkspacePick::Action(1)) => {
            app.prompt = Some(crate::app::Prompt {
                title: tr_noop!("Save Workspace"),
                cmd: "view.save_workspace",
                key: "name",
                text: String::new(),
                focus: true,
            });
        }
        Some(WorkspacePick::Action(_)) => crate::workspaces_window::open(app),
        None => {}
    }
}
