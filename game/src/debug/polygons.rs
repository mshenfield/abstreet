use crate::common::ContextMenu;
use crate::game::{State, Transition};
use crate::helpers::ID;
use crate::render::calculate_corners;
use crate::ui::UI;
use abstutil::Timer;
use ezgui::{EventCtx, GfxCtx, Key, Line, Text, WarpingItemSlider};
use geom::{Polygon, Pt2D, Triangle};

pub struct PolygonDebugger {
    slider: WarpingItemSlider<Item>,
    center: Option<Pt2D>,
}

enum Item {
    Point(Pt2D),
    Triangle(Triangle),
    Polygon(Polygon),
}

impl PolygonDebugger {
    pub fn new(ctx: &mut EventCtx, ui: &UI, ctx_menu: &mut ContextMenu) -> Option<PolygonDebugger> {
        match ctx_menu.current_focus() {
            Some(ID::Intersection(id)) => {
                let i = ui.primary.map.get_i(id);
                if ctx_menu.action(Key::X, "debug intersection geometry", ctx) {
                    let pts = i.polygon.points();
                    let mut pts_without_last = pts.clone();
                    pts_without_last.pop();
                    return Some(PolygonDebugger {
                        slider: WarpingItemSlider::new(
                            pts.iter()
                                .map(|pt| (*pt, Item::Point(*pt), Text::new()))
                                .collect(),
                            "Polygon Debugger",
                            "point",
                            ctx,
                        ),
                        center: Some(Pt2D::center(&pts_without_last)),
                    });
                } else if ctx_menu.action(Key::F2, "debug sidewalk corners", ctx) {
                    return Some(PolygonDebugger {
                        slider: WarpingItemSlider::new(
                            calculate_corners(
                                i,
                                &ui.primary.map,
                                &mut Timer::new("calculate corners"),
                            )
                            .into_iter()
                            .map(|poly| (poly.center(), Item::Polygon(poly), Text::new()))
                            .collect(),
                            "Polygon Debugger",
                            "corner",
                            ctx,
                        ),
                        center: None,
                    });
                }
            }
            Some(ID::Lane(id)) => {
                if ctx_menu.action(Key::X, "debug lane geometry", ctx) {
                    return Some(PolygonDebugger {
                        slider: WarpingItemSlider::new(
                            ui.primary
                                .map
                                .get_l(id)
                                .lane_center_pts
                                .points()
                                .iter()
                                .map(|pt| (*pt, Item::Point(*pt), Text::new()))
                                .collect(),
                            "Polygon Debugger",
                            "point",
                            ctx,
                        ),
                        center: None,
                    });
                } else if ctx_menu.action(Key::F2, "debug lane triangles", ctx) {
                    return Some(PolygonDebugger {
                        slider: WarpingItemSlider::new(
                            ui.primary
                                .draw_map
                                .get_l(id)
                                .polygon
                                .triangles()
                                .into_iter()
                                .map(|tri| {
                                    (
                                        Pt2D::center(&vec![tri.pt1, tri.pt2, tri.pt3]),
                                        Item::Triangle(tri),
                                        Text::new(),
                                    )
                                })
                                .collect(),
                            "Polygon Debugger",
                            "triangle",
                            ctx,
                        ),
                        center: None,
                    });
                }
            }
            Some(ID::Area(id)) => {
                if ctx_menu.action(Key::X, "debug area geometry", ctx) {
                    let pts = &ui.primary.map.get_a(id).polygon.points();
                    let center = if pts[0] == *pts.last().unwrap() {
                        // TODO The center looks really wrong for Volunteer Park and others, but I
                        // think it's because they have many points along some edges.
                        Pt2D::center(&pts.iter().skip(1).cloned().collect())
                    } else {
                        Pt2D::center(pts)
                    };
                    return Some(PolygonDebugger {
                        slider: WarpingItemSlider::new(
                            pts.iter()
                                .map(|pt| (*pt, Item::Point(*pt), Text::new()))
                                .collect(),
                            "Polygon Debugger",
                            "point",
                            ctx,
                        ),
                        center: Some(center),
                    });
                } else if ctx_menu.action(Key::F2, "debug area triangles", ctx) {
                    return Some(PolygonDebugger {
                        slider: WarpingItemSlider::new(
                            ui.primary
                                .map
                                .get_a(id)
                                .polygon
                                .triangles()
                                .into_iter()
                                .map(|tri| {
                                    (
                                        Pt2D::center(&vec![tri.pt1, tri.pt2, tri.pt3]),
                                        Item::Triangle(tri),
                                        Text::new(),
                                    )
                                })
                                .collect(),
                            "Polygon Debugger",
                            "triangle",
                            ctx,
                        ),
                        center: None,
                    });
                }
            }
            _ => {}
        }
        None
    }
}

impl State for PolygonDebugger {
    fn event(&mut self, ctx: &mut EventCtx, _: &mut UI) -> Transition {
        ctx.canvas.handle_event(ctx.input);

        if let Some((evmode, _)) = self.slider.event(ctx) {
            Transition::KeepWithMode(evmode)
        } else {
            Transition::Pop
        }
    }

    fn draw(&self, g: &mut GfxCtx, ui: &UI) {
        let (idx, item) = self.slider.get();

        match item {
            Item::Point(pt) => {
                g.draw_text_at(&Text::from(Line(idx.to_string())), *pt);
            }
            Item::Triangle(ref tri) => {
                for pt in &[tri.pt1, tri.pt2, tri.pt3] {
                    g.draw_text_at(&Text::from(Line(idx.to_string())), *pt);
                }
                g.draw_polygon(ui.cs.get("selected"), &Polygon::from_triangle(tri));
            }
            Item::Polygon(ref poly) => {
                g.draw_polygon(ui.cs.get("selected"), poly);
                g.draw_text_at(&Text::from(Line(idx.to_string())), poly.center());
            }
        }
        if let Some(pt) = self.center {
            g.draw_text_at(&Text::from(Line("c")), pt);
        }

        self.slider.draw(g);
    }
}
