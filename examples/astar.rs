use bevy::{color::palettes::css, math::VectorSpace, prelude::*};
use bevy_ascii_terminal::*;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal,
};
use sark_pathfinding::*;

pub const START_COLOR: LinearRgba = LinearRgba::BLUE;
pub const END_COLOR: LinearRgba = LinearRgba::GREEN;

const WALL_COLOR: f32 = 0.45;
const WALL_TILE: Tile = Tile {
    glyph: '#',
    fg_color: LinearRgba::rgb(WALL_COLOR, WALL_COLOR, WALL_COLOR),
    bg_color: LinearRgba::BLACK,
};
const FLOOR_TILE: Tile = Tile {
    glyph: ' ',
    fg_color: LinearRgba::WHITE,
    bg_color: LinearRgba::BLACK,
};

#[derive(Resource, Deref, DerefMut)]
pub struct PathMap(PathMap2d);

#[derive(Default, Resource)]
struct PathingState {
    start: Option<IVec2>,
    end: Option<IVec2>,
    time: f32,
    finder: Pathfinder,
}

impl PathingState {
    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
    }
}

fn setup(mut commands: Commands) {
    let size = [120, 60];
    commands.spawn(Terminal::new(size));
    commands.spawn(TerminalCamera::new());

    let mut map = PathMap(PathMap2d::new(size));

    build_walls(&mut map.0);

    commands.insert_resource(map);
    commands.insert_resource(PathingState::default());
}

fn input(
    input: Res<ButtonInput<MouseButton>>,
    q_cam: Query<&TerminalCamera>,
    q_term: Query<&TerminalTransform>,
    mut map: ResMut<PathMap>,
    mut path: ResMut<PathingState>,
) {
    let Some(cursor) = q_cam.get_single().ok().and_then(|c| c.cursor_world_pos()) else {
        return;
    };
    let Some(xy) = q_term
        .get_single()
        .ok()
        .and_then(|t| t.world_to_tile(cursor))
    else {
        return;
    };

    if input.just_pressed(MouseButton::Left) {
        map.toggle_obstacle(xy);
    }

    if input.just_pressed(MouseButton::Right) {
        // Set path marker
        if path.start.is_some() {
            if path.end.is_some() {
                path.clear();
                path.start = Some(xy);
            } else {
                path.end = Some(xy);
            }
        } else {
            path.clear();
            path.start = Some(xy)
        }
    }
}

fn update_path(map: Res<PathMap>, mut pstate: ResMut<PathingState>) {
    if !map.is_changed() && !pstate.is_changed() {
        return;
    }

    if let (Some(start), Some(end)) = (pstate.start, pstate.end) {
        let time = bevy::utils::Instant::now();
        pstate.finder.astar(&map.0, start, end);
        pstate.time = time.elapsed().as_secs_f32();
    }
}

fn draw(mut q_term: Query<&mut Terminal>, map: Res<PathMap>, pstate: Res<PathingState>) {
    if !map.is_changed() && !pstate.is_changed() {
        return;
    }

    let mut term = q_term.single_mut();

    for (i, tile) in (0..map.tile_count()).zip(term.tiles_mut()) {
        match map.obstacle_grid().get_index(i) {
            true => *tile = WALL_TILE,
            false => *tile = FLOOR_TILE,
        };
    }

    for p in pstate.finder.visited() {
        let glyph = match map.is_obstacle(*p) {
            true => WALL_TILE.glyph,
            false => '.',
        };
        term.put_char(*p, glyph)
            .fg(LinearRgba::RED)
            .bg(LinearRgba::BLACK);
    }

    let fg = css::YELLOW_GREEN;
    let path = pstate.finder.path();
    if !path.is_empty() {
        for (i, p) in path.iter().enumerate() {
            let t = i as f32 / (path.len() - 2) as f32;
            let col = color_lerp(START_COLOR, END_COLOR, t);
            term.put_char(*p, '█').fg(col);
        }
        term.put_string(
            [0, 2],
            format!(
                "Found path in {} ms. Length {}. Visited {} nodes.         ",
                pstate.time,
                path.len(),
                pstate.finder.visited().count()
            )
            .fg(fg),
        );
    } else {
        term.put_string([0, 2], "No valid path found                      ".fg(fg));
    }

    if let Some(start) = pstate.start {
        term.put_char(start, 'S');
        term.put_fg_color(start, LinearRgba::BLUE);
    }

    if let Some(end) = pstate.end {
        term.put_char(end, 'E');
    }

    term.put_string(
        [0, 0],
        "Left Click to toggle walls                  ".fg(fg),
    );
    term.put_string(
        [0, 1],
        "Right click to set path start/end points    ".fg(fg),
    );
    term.put_string(
        [0, 3],
        "                                            ".fg(fg),
    );
}

fn color_lerp(a: LinearRgba, b: LinearRgba, t: f32) -> LinearRgba {
    LinearRgba::lerp(a, b, t)
    // let t = f32::clamp(t, 0.0, 1.0);
    // a + (b - a) * t
    // // LinearRgba::new(
    // //     a.red + (b.red - a.red) * t,
    // //     a.green + (b.green - a.green) * t,
    // //     a.blue + (b.blue - a.blue) * t,
    // //     a.alpha + (b.alpha - a.alpha) * t,
    // // )
}

fn build_walls(walls: &mut PathMap2d) {
    let fbm = Fbm::new()
        .set_octaves(16)
        .set_frequency(1.5)
        .set_lacunarity(3.0)
        .set_persistence(0.9);
    let plane = PlaneMapBuilder::new(&fbm)
        .set_size(walls.width(), walls.height())
        .build();

    let threshold = 0.1;

    let w = walls.width();
    for i in 0..walls.tile_count() {
        let x = i % w;
        let y = i / w;

        let v = plane.get_value(x, y);
        walls.set_obstacle([x, y], v >= threshold);
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TerminalPlugins))
        .add_systems(Startup, setup)
        .add_systems(Update, (input, update_path, draw).chain())
        .run();
}
