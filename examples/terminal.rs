#![feature(let_chains)]
use bevy::{prelude::*, utils::Instant};
use bevy_ascii_terminal::{prelude::*, TiledCamera};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal,
};
use sark_pathfinding::{pathmap::PathMap2d, Pathfinder};

pub const START_COLOR: Color = Color::BLUE;
pub const END_COLOR: Color = Color::GREEN;

const WALL_VALUE: f32 = 0.45;
const WALL_TILE: Tile = Tile {
    glyph: '#',
    fg_color: Color::Rgba {
        red: WALL_VALUE,
        green: WALL_VALUE,
        blue: WALL_VALUE,
        alpha: 1.0,
    },
    bg_color: Color::BLACK,
};
const FLOOR_TILE: Tile = Tile {
    glyph: ' ',
    fg_color: Color::WHITE,
    bg_color: Color::BLACK,
};
// const TEXT_FMT: StringFormat = StringFormat {
//     fg_color: Color::YELLOW_GREEN,
//     bg_color: Color::BLACK,
//     pivot: Pivot::TopLeft,
// };

enum InputCommand {
    ToggleWall((IVec2, usize)),
    SetPath((IVec2, usize)),
}

#[derive(Resource)]
pub struct PathMap(PathMap2d);

#[derive(Default, Resource)]
struct PathingState {
    start: Option<IVec2>,
    end: Option<IVec2>,
    pathfinder: Pathfinder,
    time: f32,
}

impl PathingState {
    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
        self.pathfinder.clear();
    }
}

fn setup(mut commands: Commands) {
    let size = [120, 60];
    commands.spawn((TerminalBundle::new().with_size(size), AutoCamera));

    let mut map = PathMap(PathMap2d::new(size));

    build_walls(&mut map.0);

    commands.insert_resource(map);
    commands.insert_resource(PathingState::default());
}

impl std::ops::DerefMut for PathMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for PathMap {
    type Target = PathMap2d;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn input_to_commands(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_cam: Query<(&Camera, &GlobalTransform, &TiledCamera)>,
    map: Res<PathMap>,
    mut input_writer: EventWriter<InputCommand>,
) {
    if let Some(window) = windows.get_primary()
    && let Some(cursor_pos) = window.cursor_position()
    && let Ok((cam, t, tcam)) = q_cam.get_single()
    && let Some(pos) = tcam.screen_to_world(cursor_pos, cam, t) {
        let pos = world_to_map(&map, pos.extend(0.0));
        if !map.in_bounds(pos) {
            return;
        }
        let i = map.transform_lti(pos);
        if input.just_pressed(MouseButton::Left) {
            input_writer.send(InputCommand::ToggleWall((pos, i)));
        }

        if input.just_pressed(MouseButton::Right) {
            input_writer.send(InputCommand::SetPath((pos, i)));
        }
    }
}

fn read_inputs(
    mut evt: EventReader<InputCommand>,
    mut map: ResMut<PathMap>,
    mut path: ResMut<PathingState>,
) {
    for evt in evt.iter() {
        match evt {
            InputCommand::ToggleWall((_, i)) => {
                map[*i] = !map[*i];
            }
            InputCommand::SetPath((pos, _)) => {
                if path.start.is_some() {
                    if path.end.is_some() {
                        path.clear();
                        path.start = Some(*pos);
                    } else {
                        path.end = Some(*pos);
                    }
                } else {
                    path.clear();
                    path.start = Some(*pos)
                }
            }
        }
    }
}

fn update_path(map: Res<PathMap>, mut path: ResMut<PathingState>) {
    if !map.is_changed() && !path.is_changed() {
        return;
    }

    if let Some(start) = path.start {
        if let Some(end) = path.end {
            path.pathfinder.clear();
            let time = Instant::now();
            path.pathfinder.astar(&map.0, start, end);
            path.time = time.elapsed().as_secs_f32();
        }
    }
}

fn world_to_map(map: &PathMap2d, pos: Vec3) -> IVec2 {
    let pos = pos.truncate().floor().as_ivec2();
    let size = map.size().as_ivec2();
    pos + size / 2
}

fn draw(mut q_term: Query<&mut Terminal>, map: Res<PathMap>, path_state: Res<PathingState>) {
    if !map.is_changed() && !path_state.is_changed() {
        return;
    }

    let mut term = q_term.single_mut();

    for (is_wall, tile) in map.iter().zip(term.iter_mut()) {
        match is_wall {
            true => *tile = WALL_TILE,
            false => *tile = FLOOR_TILE,
        }
    }

    for p in path_state.pathfinder.visited() {
        let c = match map[*p] {
            true => WALL_TILE.glyph,
            false => '.',
        };
        term.put_char(*p, c.fg(Color::RED).bg(Color::BLACK));
    }

    let fg = Color::YELLOW_GREEN;
    let path = path_state.pathfinder.path();
    if !path.is_empty() {
        for (i, p) in path.iter().enumerate() {
            let t = i as f32 / (path.len() - 2) as f32;
            let col = color_lerp(START_COLOR, END_COLOR, t);
            term.put_char(*p, '█'.fg(col));
        }
        term.put_string(
            [0, 2].pivot(Pivot::TopLeft),
            format!(
                "Found path in {} ms. Length {}. Visited {} nodes.         ",
                path_state.time,
                path.len(),
                path_state.pathfinder.visited().count()
            )
            .as_str()
            .fg(fg),
        );
    } else {
        term.put_string(
            [0, 2].pivot(Pivot::TopLeft),
            "No valid path found                      ".fg(fg),
        );
    }

    if let Some(start) = path_state.start {
        term.put_char(start, 'S'.fg(Color::BLUE));
    }

    if let Some(end) = path_state.end {
        term.put_char(end, 'E');
    }

    term.put_string(
        [0, 0].pivot(Pivot::TopLeft),
        "Left Click to toggle walls                  ".fg(fg),
    );
    term.put_string(
        [0, 1].pivot(Pivot::TopLeft),
        "Right click to set path start/end points    ".fg(fg),
    );
    term.put_string(
        [0, 3].pivot(Pivot::TopLeft),
        "                                            ".fg(fg),
    );
}

fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let t = f32::clamp(t, 0.0, 1.0);
    Color::rgba(
        a.r() + (b.r() - a.r()) * t,
        a.g() + (b.g() - a.g()) * t,
        a.b() + (b.b() - a.b()) * t,
        a.a() + (b.a() - a.a()) * t,
    )
}

fn build_walls(map: &mut PathMap2d) {
    let fbm = Fbm::new()
        .set_octaves(16)
        .set_frequency(1.5)
        .set_lacunarity(3.0)
        .set_persistence(0.9);
    let plane = PlaneMapBuilder::new(&fbm)
        .set_size(map.width(), map.height())
        .build();

    let threshold = 0.1;

    let w = map.width();
    for (i, b) in map.iter_mut().enumerate() {
        let x = i % w;
        let y = i / w;

        let v = plane.get_value(x, y);

        if v >= threshold {
            *b = true;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TerminalPlugin)
        .add_startup_system(setup)
        .add_system(input_to_commands)
        .add_system(read_inputs)
        .add_system(update_path)
        .add_system(draw)
        .add_event::<InputCommand>()
        .run();
}
