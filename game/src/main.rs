mod abtest;
mod challenges;
mod common;
mod debug;
mod edit;
mod game;
mod helpers;
mod mission;
mod render;
mod sandbox;
mod splash_screen;
mod tutorial;
mod ui;

use crate::ui::Flags;
use abstutil::CmdArgs;
use sim::SimFlags;

fn main() {
    let mut args = CmdArgs::new();

    // TODO Lift this out of the game crate entirely.
    if args.enabled("--prebake") {
        challenges::prebake();
        return;
    }

    let mut flags = Flags {
        sim_flags: SimFlags::from_args(&mut args),
        kml: args.optional("--kml"),
        draw_lane_markings: !args.enabled("--dont_draw_lane_markings"),
        num_agents: args.optional_parse("--num_agents", |s| s.parse()),
        textures: args.enabled("--textures"),
        dev: args.enabled("--dev"),
    };
    if flags.dev {
        flags.sim_flags.rng_seed = Some(42);
    }
    let mut settings = ezgui::Settings::new("A/B Street", (1800.0, 800.0));
    if args.enabled("--enable_profiler") {
        settings.enable_profiling();
    }
    if let Some(n) = args.optional_parse("--font_size", |s| s.parse::<usize>()) {
        settings.default_font_size(n);
    }
    args.done();

    ezgui::run(settings, |ctx| game::Game::new(flags, ctx));
}
