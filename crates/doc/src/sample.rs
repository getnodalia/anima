//! Generated documents: the demo scene (first launch, tests, `anima-cli --sample`) and a
//! deterministic stress document for benchmarks. All artwork is simple generated geometry
//! (no third-party assets): the demo's flat palette and ridges are original to Ánima.

use crate::{
    ColorEffect, Document, Easing, Element, ElementId, ElementKind, Keyframe, Layer, LayerId, LoopMode, Paint, Rgba, Shape, Stage, Symbol, SymbolId,
    SymbolKind, Timeline, Tween,
};
use anima_geom::{Affine, Decomposed, Path, Vec2};
use std::sync::Arc;

fn el(doc: &mut Document, name: &str, transform: Affine, kind: ElementKind) -> Arc<Element> {
    Arc::new(Element { id: ElementId(doc.alloc_id()), name: name.into(), transform, color: ColorEffect::default(), kind })
}

fn instance(symbol: SymbolId) -> ElementKind {
    ElementKind::Symbol {
        symbol,
        loop_mode: LoopMode::Loop,
        first_frame: 0,
        blend: crate::BlendMode::Normal,
        filters: Vec::new(),
        events: Vec::new(),
        pivot: None,
    }
}

fn pose(tx: f32, ty: f32, sx: f32, sy: f32, rotation_deg: f32) -> Affine {
    Decomposed { tx, ty, sx, sy, rotation: rotation_deg.to_radians(), skew: 0.0 }.compose()
}

fn layer(id: u64, name: &str, color: u32, keyframes: Vec<Keyframe>) -> Arc<Layer> {
    let mut l = Layer::new(LayerId(id), name);
    l.color = Rgba::hex(color);
    l.keyframes = keyframes.into_iter().map(Arc::new).collect();
    Arc::new(l)
}

fn key(start: u32, duration: u32, elements: Vec<Arc<Element>>, tween: Option<Easing>) -> Keyframe {
    let mut k = Keyframe::new(start, duration);
    k.elements = elements;
    k.tween = tween.map(|easing| Tween { easing, ..Default::default() });
    k
}

/// A ridge line through `points` (Catmull-Rom, as cubics), closed down to `bottom`.
fn ridge(points: &[(f32, f32)], bottom: f32) -> Path {
    let pts: Vec<Vec2> = points.iter().map(|&(x, y)| Vec2::new(x, y)).collect();
    let mut p = Path::new();
    let (Some(&first), Some(&last)) = (pts.first(), pts.last()) else { return p };
    p.move_to(Vec2::new(first.x, bottom)).line_to(first);
    for i in 0..pts.len().saturating_sub(1) {
        let at = |j: usize| pts.get(j).copied().unwrap_or(last);
        let (p0, p1, p2, p3) = (at(i.saturating_sub(1)), at(i), at(i + 1), at(i + 2));
        let c1 = Vec2::new(p1.x + (p2.x - p0.x) / 6.0, p1.y + (p2.y - p0.y) / 6.0);
        let c2 = Vec2::new(p2.x - (p3.x - p1.x) / 6.0, p2.y - (p3.y - p1.y) / 6.0);
        p.cubic_to(c1, c2, p2);
    }
    p.line_to(Vec2::new(last.x, bottom)).close();
    p
}

/// Builds the demo document: an evening landscape in flat colours, with a ball bouncing across
/// the meadow, the evening star turning and the setting sun breathing over the ridges.
pub fn demo() -> Document {
    let mut d = Document {
        stage: Stage { width: 1280, height: 720, fps: 24.0, background: Rgba::hex(PAPER) },
        scenes: vec![Arc::new(crate::Scene::new("Scene 1"))],
        library: Vec::new(),
        bitmaps: Vec::new(),
        sounds: Vec::new(),
        library_tree: Default::default(),
        document_class: String::new(),
        publish: Default::default(),
        aids: Default::default(),
        swatches: Arc::new(crate::swatch::default_set()),
        eases: Default::default(),
        next_id: 1,
    };
    let (w, h) = (1280.0, 720.0);
    let solid = |c: u32| Paint::Solid(Rgba::hex(c));

    // --- library -------------------------------------------------------------------------
    let ball_id = SymbolId(d.alloc_id());
    let ball_shape = Shape::filled(Path::ellipse(0.0, 0.0, 48.0, 48.0), solid(OCHRE));
    let ball_el = el(&mut d, "ball body", Affine::IDENTITY, ElementKind::Shape(ball_shape));
    let ball_layer = layer(d.alloc_id(), "body", 0x4fa3ff, vec![key(0, 1, vec![ball_el], None)]);
    d.library.push(Arc::new(Symbol {
        id: ball_id,
        name: "Ball".into(),
        kind: SymbolKind::Graphic,
        linkage: None,
        scale_grid_kept: None,
        scale_grid: None,
        timeline: Timeline { layers: vec![ball_layer] },
    }));

    let star_id = SymbolId(d.alloc_id());
    let star_shape = Shape::filled(Path::star(0.0, 0.0, 60.0, 14.0, 4), solid(INK));
    let star_el_id = ElementId(d.alloc_id()); // same id across keyframes: one instance over time
    let star_keys = [0.0f32, 120.0, 240.0, 360.0]
        .into_iter()
        .enumerate()
        .map(|(i, deg)| {
            let e = Element {
                id: star_el_id,
                name: "star".into(),
                transform: pose(0.0, 0.0, 1.0, 1.0, deg),
                color: ColorEffect::default(),
                kind: ElementKind::Shape(star_shape.clone()),
            };
            key(i as u32 * 8, if i == 3 { 1 } else { 8 }, vec![Arc::new(e)], (i < 3).then_some(Easing::Linear))
        })
        .collect();
    let star_layer = layer(d.alloc_id(), "spin", 0x4fa3ff, star_keys);
    d.library.push(Arc::new(Symbol {
        id: star_id,
        name: "Star".into(),
        kind: SymbolKind::Graphic,
        linkage: None,
        scale_grid_kept: None,
        scale_grid: None,
        timeline: Timeline { layers: vec![star_layer] },
    }));

    // --- main timeline (index 0 = top) ---------------------------------------------------
    const LEN: u32 = 48;

    // Ball: bounces across the meadow with squash at each impact.
    let ball_layer_id = d.alloc_id();
    let ball_inst = ElementId(d.alloc_id());
    let poses = [
        (0u32, (200.0, 260.0, 1.0, 1.0), Easing::QuadIn),
        (12, (480.0, 572.0, 1.25, 0.78), Easing::QuadOut),
        (24, (760.0, 270.0, 0.95, 1.05), Easing::QuadIn),
        (36, (1010.0, 572.0, 1.25, 0.78), Easing::QuadOut),
        (47, (1160.0, 330.0, 1.0, 1.0), Easing::Linear),
    ];
    let ball_keys = poses
        .iter()
        .enumerate()
        .map(|(i, (start, (x, y, sx, sy), easing))| {
            let (start, (x, y, sx, sy), easing) = (*start, (*x, *y, *sx, *sy), easing.clone());
            let next = poses.get(i + 1).map_or(LEN, |p| p.0);
            let e = Element {
                id: ball_inst,
                name: "ball".into(),
                transform: pose(x, y, sx, sy, 0.0),
                color: ColorEffect::default(),
                kind: instance(ball_id),
            };
            key(start, next - start, vec![Arc::new(e)], (i + 1 < poses.len()).then_some(easing))
        })
        .collect();
    let ball = layer(ball_layer_id, "Ball", 0xff6b4a, ball_keys);

    // Star: a nested, self-animating graphic symbol.
    let star_layer_id = d.alloc_id();
    let star_el = el(&mut d, "star", pose(330.0, 130.0, 0.6, 0.6, 0.0), instance(star_id));
    let star = layer(star_layer_id, "Star", 0xffd54a, vec![key(0, LEN, vec![star_el], None)]);

    // Sun: low over the ridges, breathing with an ease in-out.
    let sun_layer_id = d.alloc_id();
    let sun_inst = ElementId(d.alloc_id());
    let sun_shape = Shape::filled(Path::ellipse(0.0, 0.0, 104.0, 104.0), solid(VERMILION));
    let sun_keys = [(0u32, 24u32, 1.0f32), (24, 23, 1.06), (47, 1, 1.0)]
        .into_iter()
        .enumerate()
        .map(|(i, (start, dur, s))| {
            let e = Element {
                id: sun_inst,
                name: "sun".into(),
                transform: pose(960.0, 225.0, s, s, 0.0),
                color: ColorEffect::default(),
                kind: ElementKind::Shape(sun_shape.clone()),
            };
            key(start, dur, vec![Arc::new(e)], (i < 2).then_some(Easing::QuadInOut))
        })
        .collect();
    let sun = layer(sun_layer_id, "Sun", 0xffa726, sun_keys);

    // Hills: two ridges and the meadow the ball lands on.
    let hills_id = d.alloc_id();
    let mut hill_els = Vec::new();
    for (name, path, c) in [
        (
            "far hill",
            ridge(&[(0.0, 470.0), (180.0, 418.0), (400.0, 452.0), (640.0, 392.0), (900.0, 446.0), (1110.0, 404.0), (1280.0, 436.0)], h),
            SAGE,
        ),
        ("near hill", ridge(&[(0.0, 548.0), (260.0, 500.0), (560.0, 540.0), (850.0, 492.0), (1080.0, 530.0), (1280.0, 506.0)], h), MOSS),
        ("ground", Path::rect(0.0, 608.0, w, h - 608.0), INK),
    ] {
        hill_els.push(el(&mut d, name, Affine::IDENTITY, ElementKind::Shape(Shape::filled(path, solid(c)))));
    }
    let hills = layer(hills_id, "Hills", 0x66bb6a, vec![key(0, LEN, hill_els, None)]);

    // Sky.
    let sky_id = d.alloc_id();
    let sky_el = el(&mut d, "sky", Affine::IDENTITY, ElementKind::Shape(Shape::filled(Path::rect(0.0, 0.0, w, h), solid(PAPER))));
    let mut sky = layer(sky_id, "Sky", 0x7e57c2, vec![key(0, LEN, vec![sky_el], None)]);
    Arc::make_mut(&mut sky).locked = true;

    d.timeline_mut(0).layers = vec![ball, star, sun, hills, sky];
    d
}

// The demo's palette: flat, print-like colours, original to Ánima.
const PAPER: u32 = 0xede3d1;
const VERMILION: u32 = 0xd6553b;
const OCHRE: u32 = 0xe8a73a;
const SAGE: u32 = 0xbfc8b6;
const MOSS: u32 = 0x6e8c7c;
const INK: u32 = 0x2c4549;

/// A small interactive movie (ADR 0024): a menu that stops on its first frame with a Play
/// button; clicking it plays a short show (a spinning movie clip and a ball rolling across),
/// whose last frame traces a line and goes back to the menu.
pub fn interactive() -> Document {
    use crate::actions::{Action, ButtonEvent, EventHandler, FrameRef, FrameScript, Target};
    let mut d = Document {
        stage: Stage { width: 1280, height: 720, fps: 24.0, background: Rgba::hex(0x1d2333) },
        scenes: vec![Arc::new(crate::Scene::new("Scene 1"))],
        library: Vec::new(),
        bitmaps: Vec::new(),
        sounds: Vec::new(),
        library_tree: Default::default(),
        document_class: String::new(),
        publish: Default::default(),
        aids: Default::default(),
        swatches: Arc::new(crate::swatch::default_set()),
        eases: Default::default(),
        next_id: 1,
    };
    const LEN: u32 = 36;
    let solid = |c: u32| Paint::Solid(Rgba::hex(c));

    // The Play button: Up, Over and Down looks, and a Hit area a little larger than the art.
    let button_id = SymbolId(d.alloc_id());
    let face = |d: &mut Document, c: u32| {
        let body = el(d, "face", Affine::IDENTITY, ElementKind::Shape(Shape::filled(Path::rounded_rect(-110.0, -40.0, 220.0, 80.0, 18.0), solid(c))));
        let arrow = el(
            d,
            "arrow",
            Affine::IDENTITY,
            ElementKind::Shape(Shape::filled(
                {
                    let mut p = Path::new();
                    p.move_to(Vec2::new(-14.0, -20.0)).line_to(Vec2::new(22.0, 0.0)).line_to(Vec2::new(-14.0, 20.0)).close();
                    p
                },
                solid(0xffffff),
            )),
        );
        vec![body, arrow]
    };
    let (up, over, down) = (face(&mut d, 0x3d8bfd), face(&mut d, 0x6aa8ff), face(&mut d, 0x2a64c0));
    let hit_area = el(&mut d, "hit", Affine::IDENTITY, ElementKind::Shape(Shape::filled(Path::rect(-120.0, -50.0, 240.0, 100.0), solid(0x000000))));
    let states = layer(
        d.alloc_id(),
        "states",
        0x4fa3ff,
        vec![key(0, 1, up, None), key(1, 1, over, None), key(2, 1, down, None), key(3, 1, vec![hit_area], None)],
    );
    d.library.push(Arc::new(Symbol {
        id: button_id,
        name: "Play Button".into(),
        kind: SymbolKind::Button,
        linkage: None,
        scale_grid_kept: None,
        scale_grid: None,
        timeline: Timeline { layers: vec![states] },
    }));

    // A movie clip that spins on its own timeline.
    let spinner_id = SymbolId(d.alloc_id());
    let star_shape = Shape::filled(Path::star(0.0, 0.0, 70.0, 30.0, 5), solid(0xffd54a)).with_stroke(Rgba::hex(0xe0a400), 3.0);
    let star_el = ElementId(d.alloc_id());
    let spin_keys = [(0u32, 0.0f32), (12, 360.0)]
        .into_iter()
        .map(|(start, deg)| {
            let e = Element {
                id: star_el,
                name: String::new(),
                transform: pose(0.0, 0.0, 1.0, 1.0, deg),
                color: ColorEffect::default(),
                kind: ElementKind::Shape(star_shape.clone()),
            };
            key(start, if start == 0 { 12 } else { 1 }, vec![Arc::new(e)], (start == 0).then_some(Easing::Linear))
        })
        .collect();
    let spin = layer(d.alloc_id(), "spin", 0xffd54a, spin_keys);
    d.library.push(Arc::new(Symbol {
        id: spinner_id,
        name: "Spinner".into(),
        kind: SymbolKind::MovieClip,
        linkage: None,
        scale_grid_kept: None,
        scale_grid: None,
        timeline: Timeline { layers: vec![spin] },
    }));

    // Main timeline: actions on top, then the button (menu frame), the show, the backdrop.
    let mut menu = key(0, 1, Vec::new(), None);
    menu.label = "menu".into();
    menu.script = Some(FrameScript::Actions(vec![Action::Stop { target: Target::this() }]));
    let mut show = key(1, LEN - 2, Vec::new(), None);
    show.label = "show".into();
    show.script = Some(FrameScript::Actions(vec![Action::Trace { message: "The show starts".into() }]));
    let mut back = key(LEN - 1, 1, Vec::new(), None);
    back.script = Some(FrameScript::Actions(vec![
        Action::Trace { message: "Back to the menu".into() },
        Action::GotoAndStop { target: Target::this(), frame: FrameRef::Label("menu".into()), scene: None },
    ]));
    let actions = layer(d.alloc_id(), "Actions", 0xb39ddb, vec![menu, show, back]);

    let mut play_btn = (*el(&mut d, "play_btn", pose(640.0, 400.0, 1.0, 1.0, 0.0), instance(button_id))).clone();
    if let ElementKind::Symbol { events, .. } = &mut play_btn.kind {
        events.push(EventHandler {
            event: ButtonEvent::Click,
            actions: vec![Action::GotoAndPlay { target: Target::this(), frame: FrameRef::Label("show".into()), scene: None }],
        });
    }
    let title = el(
        &mut d,
        "title bar",
        Affine::IDENTITY,
        ElementKind::Shape(Shape::filled(Path::rounded_rect(440.0, 220.0, 400.0, 24.0, 12.0), solid(0x8a93a8))),
    );
    let buttons = layer(d.alloc_id(), "Menu", 0x4fa3ff, vec![key(0, 1, vec![title, Arc::new(play_btn)], None), key(1, LEN - 1, Vec::new(), None)]);

    let spinner = el(&mut d, "spinner", pose(640.0, 300.0, 1.0, 1.0, 0.0), instance(spinner_id));
    let ball_id = ElementId(d.alloc_id());
    let ball = |x: f32| {
        Arc::new(Element {
            id: ball_id,
            name: "ball".into(),
            transform: pose(x, 560.0, 1.0, 1.0, 0.0),
            color: ColorEffect::default(),
            kind: ElementKind::Shape(Shape::filled(Path::ellipse(0.0, 0.0, 36.0, 36.0), solid(0xff6b4a))),
        })
    };
    let show_layer = layer(
        d.alloc_id(),
        "Show",
        0xff6b4a,
        vec![
            key(0, 1, Vec::new(), None),
            key(1, LEN - 2, vec![spinner.clone(), ball(160.0)], Some(Easing::QuadInOut)),
            key(LEN - 1, 1, vec![spinner, ball(1120.0)], None),
        ],
    );

    let floor = el(&mut d, "floor", Affine::IDENTITY, ElementKind::Shape(Shape::filled(Path::rect(0.0, 600.0, 1280.0, 120.0), solid(0x2a3247))));
    let backdrop = layer(d.alloc_id(), "Backdrop", 0x66bb6a, vec![key(0, LEN, vec![floor], None)]);

    d.timeline_mut(0).layers = vec![actions, buttons, show_layer, backdrop];
    d
}

/// Deterministic stress document for benchmarks and scaling tests: `layers` layers with
/// `shapes` shapes each, keyed every 12 frames over `frames` frames with classic tweens.
/// Every element keeps one `Arc<Path>` across its keyframes, as a real tweened shape does.
pub fn stress(layers: u32, shapes: u32, frames: u32) -> Document {
    let mut d = Document {
        stage: Stage { width: 1920, height: 1080, fps: 24.0, background: Rgba::WHITE },
        scenes: vec![Arc::new(crate::Scene::new("Scene 1"))],
        library: Vec::new(),
        bitmaps: Vec::new(),
        sounds: Vec::new(),
        library_tree: Default::default(),
        document_class: String::new(),
        publish: Default::default(),
        aids: Default::default(),
        swatches: Arc::new(crate::swatch::default_set()),
        eases: Default::default(),
        next_id: 1,
    };
    let (layers, shapes, frames) = (layers.clamp(1, 1000), shapes.clamp(1, 100_000), frames.clamp(1, 10_000));
    let mut seed = 0x2545_f491_u32;
    let mut rnd = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed as f32) / (u32::MAX as f32)
    };
    for li in 0..layers {
        let lid = d.alloc_id();
        let els: Vec<(ElementId, Arc<Path>, Paint, f32, f32)> = (0..shapes)
            .map(|si| {
                let r = 6.0 + rnd() * 40.0;
                let path = match si % 3 {
                    0 => Path::ellipse(0.0, 0.0, r, r * (0.5 + rnd())),
                    1 => Path::star(0.0, 0.0, r, r * 0.45, 5),
                    _ => Path::rounded_rect(-r, -r * 0.6, r * 2.0, r * 1.2, r * 0.2),
                };
                let c = Rgba::new(rnd(), rnd(), rnd(), 0.6 + 0.4 * rnd());
                (ElementId(d.alloc_id()), Arc::new(path), Paint::Solid(c), rnd() * 1920.0, rnd() * 1080.0)
            })
            .collect();
        let mut keys = Vec::new();
        let mut start = 0;
        while start < frames {
            let dur = 12.min(frames - start);
            let phase = start as f32 * 0.05;
            let elements = els
                .iter()
                .map(|(id, path, fill, x, y)| {
                    let dx = (phase + x * 0.01).sin() * 60.0;
                    let dy = (phase + y * 0.01).cos() * 40.0;
                    Arc::new(Element {
                        id: *id,
                        name: String::new(),
                        transform: pose(x + dx, y + dy, 1.0, 1.0, phase * 20.0),
                        color: ColorEffect::default(),
                        kind: ElementKind::Shape(Shape {
                            path: path.clone(),
                            fill: Some(fill.clone()),
                            stroke: None,
                            fill_rule: Default::default(),
                            merge: false,
                            warp: None,
                        }),
                    })
                })
                .collect();
            keys.push(key(start, dur, elements, (start + dur < frames).then_some(Easing::QuadInOut)));
            start += dur;
        }
        d.timeline_mut(0).layers.push(layer(lid, &format!("Layer {}", li + 1), 0x4fa3ff, keys));
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_is_consistent() {
        let mut d = demo();
        assert_eq!(d.frame_count(), 48);
        let before = d.clone();
        d.sanitize();
        assert_eq!(d.scenes, before.scenes, "demo must already satisfy invariants");
        let items = crate::evaluate(&d, 10).items.len();
        assert!(items >= 7, "{items}");
    }

    #[test]
    fn stress_shape() {
        let d = stress(3, 50, 30);
        assert_eq!(d.timeline(0).layers.len(), 3);
        assert_eq!(d.frame_count(), 30);
        assert_eq!(crate::evaluate(&d, 7).items.len(), 150);
        // paths are shared across keyframes
        let l = &d.timeline(0).layers[0];
        let (ElementKind::Shape(a), ElementKind::Shape(b)) = (&l.keyframes[0].elements[0].kind, &l.keyframes[1].elements[0].kind) else { panic!() };
        assert!(Arc::ptr_eq(&a.path, &b.path));
    }
}
