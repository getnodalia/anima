//! The window's own title bar on Linux, where Ánima draws Nodalia's window controls and the
//! window has no title bar of the system's ([`App::custom_titlebar`], NDL): empty space in the top
//! bar drags the window and a double-click there maximizes it, the round controls at its right end
//! minimize, maximize or restore, and close, and thin invisible zones along the window's edges
//! resize it. macOS and Windows keep their native controls.

use crate::app::App;
use egui::{CursorIcon, Id, LayerId, Order, PointerButton, Rect, ResizeDirection, Sense, Ui, ViewportCommand, pos2};
use nodalia_theme::titlebar::{Placement, Side};

/// The room the round controls take at the top bar's right end, with their gap to the edge.
pub const WIDTH: f32 = nodalia_theme::window_controls::WIDTH + EDGE_GAP;
/// The controls' distance from the window's right edge.
const EDGE_GAP: f32 = 8.0;
/// Resize zone thickness along an edge, and the side of a corner zone.
const EDGE: f32 = 5.0;
const CORNER: f32 = 12.0;

fn maximized(ctx: &egui::Context) -> bool {
    ctx.input(|i| i.viewport().maximized.unwrap_or(false))
}

fn toggle_maximize(ctx: &egui::Context) {
    ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized(ctx)));
}

/// The bar's background: a press-and-drag on empty space moves the window, a double-click toggles
/// maximize. Call it before laying out the bar so its widgets, registered later, take the clicks.
pub fn drag_area(ui: &Ui, rect: Rect) {
    let ctx = ui.ctx();
    // Not focusable: Tab goes through the bar's widgets, not its background.
    let resp = ui.interact(rect, Id::unique("titlebar-drag"), Sense::CLICK | Sense::DRAG);
    // egui hands a drag that starts on a click-only widget (a menu title) to the drag-sensing
    // background under it, so remember whether the press itself landed on bare background.
    if ctx.input(|i| i.pointer.any_pressed()) {
        let bare = ctx.viewport(|v| v.hits.click.is_some_and(|w| w.id == resp.id));
        ctx.data_mut(|d| d.insert_temp(resp.id, bare));
    }
    if resp.double_clicked() {
        toggle_maximize(ctx);
    } else if resp.drag_started_by(PointerButton::Primary) && ctx.data(|d| d.get_temp(resp.id).unwrap_or(false)) {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        handed_to_os(ctx);
    }
}

/// The window manager runs the move or resize from here and, on X11 and Wayland, keeps the
/// button's release: egui would hold the drag until the next press. End both now.
fn handed_to_os(ctx: &egui::Context) {
    ctx.stop_dragging();
    ctx.input_mut(|i| i.pointer = Default::default());
}

/// The end of the bar Nodalia's controls sit at: the side the desktop keeps its own on (the right
/// unless the desktop's button layout puts them at the left). `None` without them.
pub fn side(app: &App) -> Option<Side> {
    app.custom_titlebar.then(nodalia_theme::titlebar::system_side)
}

/// Nodalia's minimize, maximize/restore and close at the desktop's side of `bar`. Close asks the
/// window to close, which the unsaved-work guard sees as the system's close would be.
pub fn window_controls(ui: &mut Ui, bar: Rect, side: Side) {
    let ctx = ui.ctx().clone();
    let rect = Placement::nodalia(side).controls_rect(bar);
    let focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
    let labels =
        nodalia_theme::window_controls::Labels { minimize: tr!("Minimize"), maximize: tr!("Maximize"), restore: tr!("Restore"), close: tr!("Close") };
    use nodalia_theme::window_controls::Action;
    let palette = crate::theme::ndl_palette();
    match nodalia_theme::window_controls::show_on(ui, rect, side, &palette, crate::theme::ndl_mode(), focused, maximized(&ctx), labels) {
        Some(Action::Minimize) => ctx.send_viewport_cmd(ViewportCommand::Minimized(true)),
        Some(Action::Maximize | Action::Restore) => toggle_maximize(&ctx),
        Some(Action::Close) => ctx.send_viewport_cmd(ViewportCommand::Close),
        None => {}
    }
}

/// The edge and corner zones of a window with content rect `w`: (zone, direction, cursor). Corners
/// come last so they win where they overlap the edges.
fn zones(w: Rect) -> [(Rect, ResizeDirection, CursorIcon); 8] {
    use ResizeDirection::*;
    use egui::vec2;
    let (e, c) = (EDGE, CORNER);
    [
        (Rect::from_min_max(w.left_top(), pos2(w.right(), w.top() + e)), North, CursorIcon::ResizeVertical),
        (Rect::from_min_max(pos2(w.left(), w.bottom() - e), w.right_bottom()), South, CursorIcon::ResizeVertical),
        (Rect::from_min_max(w.left_top(), pos2(w.left() + e, w.bottom())), West, CursorIcon::ResizeHorizontal),
        (Rect::from_min_max(pos2(w.right() - e, w.top()), w.right_bottom()), East, CursorIcon::ResizeHorizontal),
        (Rect::from_min_size(w.left_top(), vec2(c, c)), NorthWest, CursorIcon::ResizeNwSe),
        (Rect::from_min_size(w.right_top() - vec2(c, 0.0), vec2(c, c)), NorthEast, CursorIcon::ResizeNeSw),
        (Rect::from_min_size(w.left_bottom() - vec2(0.0, c), vec2(c, c)), SouthWest, CursorIcon::ResizeNeSw),
        (Rect::from_min_size(w.right_bottom() - vec2(c, c), vec2(c, c)), SouthEast, CursorIcon::ResizeNwSe),
    ]
}

/// Invisible resize zones along the window's edges, above everything else; a press in one starts
/// a resize by the window manager. None while the window is maximized or fullscreen.
pub fn resize_zones(app: &App, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    if !app.custom_titlebar || maximized(&ctx) || ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)) {
        return;
    }
    let w = ctx.content_rect();
    let layer = LayerId::new(Order::Foreground, Id::unique("titlebar-resize"));
    let zui = ui.new_child(egui::UiBuilder::new().layer_id(layer).max_rect(w));
    for (i, (r, dir, cursor)) in zones(w).into_iter().enumerate() {
        let resp = zui.interact(r, Id::unique(("titlebar-resize", i)), Sense::DRAG).on_hover_cursor(cursor);
        if resp.drag_started_by(PointerButton::Primary) {
            ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
            handed_to_os(&ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_zones_win_over_edges_and_stay_inside_the_window() {
        let w = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(800.0, 600.0));
        let z = zones(w);
        assert!(z.iter().all(|(r, _, _)| w.contains_rect(*r)));
        assert_eq!(z[7].1, ResizeDirection::SouthEast);
    }
}
