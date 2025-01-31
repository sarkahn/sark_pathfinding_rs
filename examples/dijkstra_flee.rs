use bevy::prelude::*;
use bevy_ascii_terminal::*;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal,
};
use rand::seq::{IteratorRandom, SliceRandom};
use sark_pathfinding::*;

pub const SIZE: UVec2 = UVec2::from_array([50, 40]);
pub const WALL_TILE: Tile = Tile::new('#', colors::WHITE, colors::BLACK);
pub const FLOOR_TILE: Tile = Tile::new('.', colors::DARK_GRAY, colors::BLACK);
pub const PLAYER_TILE: Tile = Tile::new('@', colors::ALICE_BLUE, colors::BLACK);
pub const GOLD_TILE: Tile = Tile::new('$', colors::YELLOW, colors::BLACK);
pub const GOBLIN_TILE: Tile = Tile::new('g', colors::RED, colors::BLACK);

/// Maintain a separate map for each behavior
#[derive(Component)]
pub struct FleeBehavior(DijkstraMap);

#[derive(Resource, Deref, DerefMut)]
pub struct PathMap(PathMap2d);

#[derive(Resource, Deref, DerefMut)]
pub struct DrawMap(bool);

#[derive(Component)]
pub struct Goblin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct ApproachMap(DijkstraMap);

#[derive(Component)]
pub struct Gold;

#[derive(Component)]
pub struct Renderable(Tile);

#[derive(Component, Deref, DerefMut)]
pub struct Position(pub IVec2);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TerminalPlugins))
        .insert_resource(PathMap(PathMap2d::new(SIZE)))
        .insert_resource(DrawMap(true))
        .add_systems(Startup, setup)
        .add_systems(Update, (input, update.run_if(player_moved), render).chain())
        .run();
}

fn setup(mut commands: Commands, mut map: ResMut<PathMap>) {
    commands.spawn(Terminal::new(SIZE));
    commands.spawn(TerminalCamera::new());
    build_level(&mut map.0);

    let mut open_points: Vec<_> = map
        .iter_grid_points()
        .filter(|p| !map.is_obstacle(*p))
        .collect();
    let mut rng = rand::thread_rng();
    let mut random_point = || {
        let i = (0..open_points.len()).choose(&mut rng).unwrap();
        open_points.remove(i)
    };
    commands.spawn((
        Player,
        Renderable(PLAYER_TILE),
        ApproachMap(DijkstraMap::new(SIZE)),
        Position(random_point()),
    ));
    for _ in 0..3 {
        commands.spawn((
            Goblin,
            Renderable(GOBLIN_TILE),
            Position(random_point()),
            FleeBehavior(DijkstraMap::new(SIZE)),
        ));
    }
}

fn build_level(map: &mut PathMap2d) {
    let fbm = Fbm::new()
        .set_octaves(16)
        .set_frequency(1.5)
        .set_lacunarity(3.0)
        .set_persistence(0.7);
    let plane = PlaneMapBuilder::new(&fbm)
        .set_size(map.width(), map.height())
        .build();

    let threshold = 0.1;

    let w = map.width();
    for i in 0..map.tile_count() {
        let x = i % w;
        let y = i / w;

        let v = plane.get_value(x, y);
        map.set_obstacle([x, y], v >= threshold);
    }
}

fn input(
    key: Res<ButtonInput<KeyCode>>,
    mut q_player: Query<&mut Position, With<Player>>,
    //mut show: ResMut<Flee>,
    pathmap: Res<PathMap>,
) {
    let Ok(mut player) = q_player.get_single_mut() else {
        return;
    };

    let left = -(key.any_just_pressed([
        KeyCode::Numpad1,
        KeyCode::Numpad4,
        KeyCode::Numpad7,
        KeyCode::KeyZ,
        KeyCode::KeyA,
        KeyCode::KeyQ,
    ]) as i32);
    let up = key.any_just_pressed([
        KeyCode::Numpad7,
        KeyCode::Numpad8,
        KeyCode::Numpad9,
        KeyCode::KeyQ,
        KeyCode::KeyW,
        KeyCode::KeyE,
    ]) as i32;
    let down = -(key.any_just_pressed([
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::KeyZ,
        KeyCode::KeyX,
        KeyCode::KeyC,
    ]) as i32);
    let right = key.any_just_pressed([
        KeyCode::Numpad3,
        KeyCode::Numpad6,
        KeyCode::Numpad9,
        KeyCode::KeyC,
        KeyCode::KeyD,
        KeyCode::KeyE,
    ]) as i32;
    let movement = IVec2::new(right + left, up + down);
    if movement.cmpeq(IVec2::ZERO).all() {
        return;
    }
    let next = player.0 + movement;
    if pathmap.is_obstacle(next) {
        return;
    }
    player.0 = next;
}

fn player_moved(q_player: Query<&Position, (With<Player>, Changed<Position>)>) -> bool {
    !q_player.is_empty()
}

fn update(mut q_approach: Query<(&mut ApproachMap, &Position)>, pathing: Res<PathMap>) {
    for (mut approach, pos) in &mut q_approach {
        let approach = &mut approach.0;
        approach.clear_all();
        approach.add_goal(pos.0, 1.0);
        approach.recalculate(&pathing.0);
        approach.apply_operation(|v| -v);
    }
}

fn render(
    mut q_term: Query<&mut Terminal>,
    draw_map: Res<DrawMap>,
    pathing: Res<PathMap>,
    q_renderables: Query<(&Renderable, &Position)>,
    q_approach_map: Query<&ApproachMap>,
) {
    let mut term = q_term.single_mut();
    let count = term.tile_count();
    let tiles = term.tiles_mut();

    (0..count).for_each(|i| {
        if pathing.obstacle_grid().get_index(i) {
            tiles[i] = WALL_TILE;
        } else {
            tiles[i] = FLOOR_TILE;
        }
    });

    for (r, p) in &q_renderables {
        term.put_tile(p.0, r.0);
    }

    if draw_map.0 {
        for approach in &q_approach_map {
            for (p, v) in approach.0.iter_xy() {
                let color = if v.is_finite() {
                    let v = (v / 255.0) as u8;
                    LinearRgba::from_u8_array([v, v, v, 255])
                } else {
                    colors::MAGENTA
                };
                if let Some(ch) = char::from_digit((v as i32).unsigned_abs(), 10) {
                    term.put_tile(p, Tile::new(ch, colors::WHITE, colors::BLACK));
                }
            }
        }
    }
}
