use crate::colors::ColorScheme;
use crate::objects::{DrawCtx, ID};
use crate::render::{RenderOptions, Renderable};
use ezgui::{Color, GfxCtx};
use geom::{Bounds, Polygon, Pt2D};
use map_model::{Area, AreaID, AreaType, Map};

pub struct DrawArea {
    pub id: AreaID,
}

impl DrawArea {
    pub fn new(area: &Area, cs: &ColorScheme) -> (DrawArea, Color, Polygon) {
        let color = match area.area_type {
            AreaType::Park => cs.get_def("park area", Color::rgb(200, 250, 204)),
            AreaType::Water => cs.get_def("water area", Color::rgb(170, 211, 223)),
        };

        (DrawArea { id: area.id }, color, area.polygon.clone())
    }
}

impl Renderable for DrawArea {
    fn get_id(&self) -> ID {
        ID::Area(self.id)
    }

    fn draw(&self, g: &mut GfxCtx, opts: RenderOptions, ctx: &DrawCtx) {
        if let Some(color) = opts.color {
            g.draw_polygon(color, &ctx.map.get_a(self.id).polygon);
        }
    }

    fn get_bounds(&self, map: &Map) -> Bounds {
        map.get_a(self.id).polygon.get_bounds()
    }

    fn contains_pt(&self, pt: Pt2D, map: &Map) -> bool {
        map.get_a(self.id).polygon.contains_pt(pt)
    }
}
