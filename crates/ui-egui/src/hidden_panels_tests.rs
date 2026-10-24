//! Auto-Show Hidden Panels (Preferences ▸ Workspace ▸ Auto-show hidden panels, #204).

use egui::{pos2, vec2};

use crate::PhotocraftApp;
use crate::auto_show_peek;
use crate::dock_strip_id;
use crate::record_panel_strip;

/// The app with its panels hidden by Tab, as the peek is meant to reveal them.
fn app() -> PhotocraftApp {
    let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
    app.ui.panels.toolbar = false;
    app.ui.panels.options_bar = false;
    app.ui.panels.dock = false;
    app.ui.panels.hidden_by_tab = Some([true, true, true]);
    app
}

/// A frame with the pointer at `p` on a 1200×800 window.
fn frame_at(ctx: &egui::Context, p: egui::Pos2) {
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1200.0, 800.0));
    let mut out = ctx.run_ui(egui::RawInput { screen_rect: Some(screen), events: vec![egui::Event::PointerMoved(p)], ..Default::default() }, |_ui| {});
    out.textures_delta.clear();
}

/// A frame with the pointer at `p` and the primary button held down (a drag).
fn frame_down_at(ctx: &egui::Context, p: egui::Pos2) {
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1200.0, 800.0));
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(screen),
            events: vec![
                egui::Event::PointerMoved(p),
                egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::default() },
            ],
            ..Default::default()
        },
        |_ui| {},
    );
    out.textures_delta.clear();
}

fn ctx_at(p: egui::Pos2) -> egui::Context {
    let ctx = egui::Context::default();
    frame_at(&ctx, p);
    ctx
}

fn ctx_down_at(p: egui::Pos2) -> egui::Context {
    let ctx = egui::Context::default();
    frame_down_at(&ctx, p);
    ctx
}

/// The strip a revealed 260-point dock occupies at the end edge of a 1200×800 window.
fn dock_strip(ctx: &egui::Context) {
    record_panel_strip(ctx, dock_strip_id(), egui::Rect::from_min_max(pos2(940.0, 0.0), pos2(1200.0, 800.0)));
}

/// The dock edge trigger sits just outside the 12-point vertical scrollbar: the scrollbar
/// strip itself is not a trigger, so the edge point is 15 points from the edge, not 6.
const DOCK_EDGE: f32 = 1185.0;

/// The window edges reveal the hidden toolbar (start edge) and dock (end edge); the middle
/// reveals nothing, and a shown panel is not peeked.
#[test]
fn the_window_edges_reveal_the_hidden_panels() {
    let mut app = app();
    let peek = |app: &PhotocraftApp, p: egui::Pos2| auto_show_peek(app, &ctx_at(p));
    assert!(peek(&app, pos2(6.0, 400.0)).toolbar, "the start edge reveals the toolbar");
    assert!(!peek(&app, pos2(6.0, 400.0)).dock);
    assert!(peek(&app, pos2(DOCK_EDGE, 400.0)).dock, "the end edge reveals the dock");
    assert_eq!(peek(&app, pos2(600.0, 400.0)), crate::Peek::default(), "the middle reveals nothing");
    app.ui.panels.toolbar = true;
    assert!(!peek(&app, pos2(6.0, 400.0)).toolbar, "a shown toolbar is not peeked");
    app.ui.panels.dock = true;
    assert_eq!(peek(&app, pos2(DOCK_EDGE, 400.0)), crate::Peek::default(), "a shown dock is not peeked");
}

/// The peek follows the preference, and holds while the pointer is over the strip it revealed
/// (so the pointer can reach into the panel it brought back).
#[test]
fn the_peek_follows_the_preference_and_keeps_over_the_revealed_panel() {
    let mut app = app();
    let ctx = ctx_at(pos2(1000.0, 400.0));
    // Nothing recorded yet: the pointer is in the canvas, far from the end edge.
    assert_eq!(auto_show_peek(&app, &ctx), crate::Peek::default());
    // The dock was drawn 260 points wide at the end edge: the pointer over it keeps the peek.
    dock_strip(&ctx);
    assert!(auto_show_peek(&app, &ctx).dock, "the pointer over the revealed dock keeps it");
    // The preference off: the edge hover does nothing.
    app.session.prefs.edit(|p| p.workspace.auto_show_hidden_panels = false);
    assert_eq!(auto_show_peek(&app, &ctx_at(pos2(6.0, 400.0))), crate::Peek::default(), "preference off");
}

/// A strip only keeps an *active* peek alive: once the pointer leaves it the strip is dropped,
/// so returning to the panel's former area does not reveal it without the window edge.
#[test]
fn the_strip_only_keeps_an_active_peek_alive() {
    let app = app();
    let ctx = egui::Context::default();
    dock_strip(&ctx);
    frame_at(&ctx, pos2(1000.0, 400.0));
    assert!(auto_show_peek(&app, &ctx).dock, "inside the revealed strip the peek holds");
    frame_at(&ctx, pos2(600.0, 400.0));
    assert_eq!(auto_show_peek(&app, &ctx), crate::Peek::default(), "leaving the panel ends the peek");
    frame_at(&ctx, pos2(1000.0, 400.0));
    assert_eq!(auto_show_peek(&app, &ctx), crate::Peek::default(), "the stale strip does not reopen it");
    frame_at(&ctx, pos2(DOCK_EDGE, 400.0));
    assert!(auto_show_peek(&app, &ctx).dock, "the edge still reveals it");
}

/// A normally visible panel must not leave a strip behind: Tab-hiding it afterwards cannot turn
/// its former area into a hover target.
#[test]
fn a_strip_left_from_a_visible_panel_cannot_reopen_it() {
    let mut app = app();
    let ctx = ctx_at(pos2(1000.0, 400.0));
    dock_strip(&ctx);
    app.ui.panels.dock = true;
    assert_eq!(auto_show_peek(&app, &ctx), crate::Peek::default(), "a visible dock is not peeked");
    app.ui.panels.dock = false;
    assert_eq!(auto_show_peek(&app, &ctx), crate::Peek::default(), "the stale strip was dropped");
    assert!(auto_show_peek(&app, &ctx_at(pos2(DOCK_EDGE, 400.0))).dock, "the edge still reveals it");
}

/// The pointer over the vertical scrollbar reaches the scrollbar, not the dock: the 18-point
/// edge zone must not cover the 12-point bar at the window's right edge.
#[test]
fn pointer_on_vertical_scrollbar_does_not_peek() {
    let app = app();
    let win = egui::Rect::from_min_size(egui::Pos2::ZERO, vec2(1200.0, 800.0));
    let bar = crate::scrollbars::bar_rects(win, true, true).1.expect("vertical bar");
    assert_eq!(auto_show_peek(&app, &ctx_at(bar.center())), crate::Peek::default());
}

/// The pointer over the vertical ruler reaches the ruler, not the toolbar; the dock edge is
/// unaffected by the ruler.
#[test]
fn pointer_on_vertical_ruler_does_not_peek() {
    let mut app = app();
    app.ui.extras.rulers = true;
    assert_eq!(auto_show_peek(&app, &ctx_at(pos2(6.0, 400.0))), crate::Peek::default(), "the ruler strip is not a toolbar trigger");
    assert!(auto_show_peek(&app, &ctx_at(pos2(DOCK_EDGE, 400.0))).dock, "the dock edge still reveals the dock");
}

/// A drag never starts a peek, so a stroke or marquee reaching the edge does not move the
/// image under the pointer; but a click inside a peek the pointer is already in keeps it.
#[test]
fn a_drag_does_not_start_a_peek() {
    let app = app();
    assert_eq!(auto_show_peek(&app, &ctx_down_at(pos2(6.0, 400.0))), crate::Peek::default(), "a drag over the start edge starts no peek");
    assert_eq!(auto_show_peek(&app, &ctx_down_at(pos2(DOCK_EDGE, 400.0))), crate::Peek::default(), "a drag over the end edge starts no peek");
    let ctx = ctx_down_at(pos2(1000.0, 400.0));
    dock_strip(&ctx);
    assert!(auto_show_peek(&app, &ctx).dock, "a click inside the revealed dock keeps it");
}

/// A panel closed from the Window menu — not hidden by Tab — stays closed on hover.
#[test]
fn a_panel_closed_from_the_window_menu_does_not_peek() {
    let mut app = app();
    app.ui.panels.hidden_by_tab = None;
    assert_eq!(auto_show_peek(&app, &ctx_at(pos2(6.0, 400.0))), crate::Peek::default(), "a Window-closed toolbar stays closed");
    assert_eq!(auto_show_peek(&app, &ctx_at(pos2(DOCK_EDGE, 400.0))), crate::Peek::default(), "a Window-closed dock stays closed");
}
