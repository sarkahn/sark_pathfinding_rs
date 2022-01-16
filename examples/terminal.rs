use bevy::{prelude::*, utils::Instant};
use bevy_ascii_terminal::*;
use bevy_tiled_camera::*;
use noise::{Fbm, utils::{PlaneMapBuilder, NoiseMapBuilder}, MultiFractal};
use sark_pathfinding::{AStar, pathing_map::{PathMap2d}};

pub const START_COLOR: Color = Color::BLUE;
pub const END_COLOR: Color = Color::GREEN;

const WALL_VALUE: f32 = 0.45;
const WALL_TILE: Tile = Tile {
    glyph: '#',
    fg_color: Color::Rgba { red: WALL_VALUE, green: WALL_VALUE, blue: WALL_VALUE, alpha: 1.0 },
    bg_color: Color::BLACK
};
const FLOOR_TILE: Tile = Tile {
    glyph: ' ',
    fg_color: Color::WHITE,
    bg_color: Color::BLACK
};
const TEXT_FMT: StringFormat = StringFormat {
    fg_color: Color::YELLOW_GREEN,
    bg_color: Color::BLACK,
    pivot: Pivot::TopLeft,
};

enum InputCommand {
    ToggleWall((IVec2,usize)),
    SetPath((IVec2,usize)),
}

#[derive(Default)]
struct PathingState {
    start: Option<IVec2>,
    end: Option<IVec2>,
    astar: AStar<[i32;2]>,
    time: f32,
}

impl PathingState {
    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
        self.astar.clear();
    }
}

fn setup(
    mut commands: Commands
) {
    let size = [120,60];
    commands.spawn_bundle(TerminalBundle::new()
    .with_size(size));

    commands.spawn_bundle(TiledCameraBundle::new()
    .with_tile_count(size));

    let mut map = PathMap2d::new([120,60]);

    build_walls(&mut map);

    commands.insert_resource(map);
    commands.insert_resource(PathingState::default());
}

fn input_to_commands(
    input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    q_cam: Query<(&Camera, &GlobalTransform, &TiledProjection)>,
    map: Res<PathMap2d>,
    mut input_writer: EventWriter<InputCommand>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(cursor_pos) = window.cursor_position() {

        let (cam, t, proj) = q_cam.single();

        if let Some(pos) = proj.screen_to_world(cam, &windows, t, cursor_pos) {
            let pos = world_to_map(&map, pos);
            if !map.in_bounds(pos.into()) {
                return;
            }
            let i = map.to_index(pos.into());
            if input.just_pressed(MouseButton::Left) {
                input_writer.send(InputCommand::ToggleWall((pos,i)));
            }

            if input.just_pressed(MouseButton::Right) {
                input_writer.send(InputCommand::SetPath((pos,i)));
            }
        }
    }
}

fn read_inputs(
    mut evt: EventReader<InputCommand>,
    mut map: ResMut<PathMap2d>,
    mut path: ResMut<PathingState>,
) {
    for evt in evt.iter() {
        match evt {
            InputCommand::ToggleWall((_,i)) => {
                map.toggle_obstacle_index(*i);
            },
            InputCommand::SetPath((pos,_)) => {
                if let Some(_) = path.start {
                    if let Some(_) = path.end {
                        path.clear();
                        path.start = Some(*pos);
                    } else {
                        path.end = Some(*pos);
                    }
                } else {
                    path.clear();
                    path.start = Some(*pos)
                }
            },
        }
    }
}

fn update_path(
    map: Res<PathMap2d>,
    mut path: ResMut<PathingState>,
) {
    if !map.is_changed() && !path.is_changed() {
        return;
    }

    if let Some(start) = path.start {
        if let Some(end) = path.end {
            path.astar.clear();
            let time = Instant::now();
            path.astar.find_path(&*map, start.into(), end.into());
            path.time = time.elapsed().as_secs_f32();

        }
    }
}

fn world_to_map(map: &PathMap2d, pos: Vec3) -> IVec2 {
    let pos = pos.truncate().floor().as_ivec2();
    let size = map.size().as_ivec2();
    pos + size / 2
}

fn draw(
    mut q_term: Query<&mut Terminal>,
    map: Res<PathMap2d>,
    path_state: Res<PathingState>,
) {
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

    for p in path_state.astar.visited() {
        let fmt = CharFormat::new(Color::RED, Color::BLACK);

        let c = match map.is_obstacle(p) {
            true => WALL_TILE.glyph,
            false => '.',
        };
        term.put_char_formatted(p, c, fmt);
    }

    if let Some(path) = path_state.astar.path() {
        for (i, p) in path.iter().enumerate() {
            let t = i as f32 / (path.len() - 2) as f32;
            let col = color_lerp(START_COLOR, END_COLOR, t);
            let fmt = CharFormat::new(col, Color::BLACK);
            term.put_char_formatted(*p, '█', fmt);
        }
        term.put_string_formatted([0,2], format!("Found path in {} ms. Length {}. Visited {} nodes.         ", 
            path_state.time, path.len(), path_state.astar.visited().count()).as_str(), TEXT_FMT);
    } else {
        term.put_string_formatted([0,2], "No valid path found                      ", TEXT_FMT);
    }

    if let Some(start) = path_state.start {
        let fmt = CharFormat::new(Color::BLUE, Color::BLACK);
        term.put_char_formatted(start.into(), 'S', fmt);
    }

    if let Some(end) = path_state.end {
        term.put_char(end.into(), 'E');
    }

    term.put_string_formatted([0,0], "Left Click to toggle walls                  ", TEXT_FMT);
    term.put_string_formatted([0,1], "Right click to set path start/end points    ", TEXT_FMT);
    term.put_string_formatted([0,3], "                                            ", TEXT_FMT);
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

fn build_walls(
    map: &mut PathMap2d,
) {
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
    .add_plugin(TiledCameraPlugin)
    .add_startup_system(setup)
    .add_system(input_to_commands)
    .add_system(read_inputs)
    .add_system(update_path)
    .add_system(draw)
    .add_event::<InputCommand>()
    .run();
}