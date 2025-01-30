use bevy::{color::palettes::css, math::VectorSpace, prelude::*};
use bevy_ascii_terminal::*;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal,
};
use rand::seq::SliceRandom;
use sark_pathfinding::*;

// Most named colors cannot be used in a const context since recent color changes
// in bevy.
// Guess we have to do it ourselves.
const fn u8_color(r: u8, g: u8, b: u8) -> LinearRgba {
    LinearRgba::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}
const ALICE_BLUE: LinearRgba = u8_color(240, 248, 255);
const DARK_GRAY: LinearRgba = u8_color(169, 169, 169);
const YELLOW: LinearRgba = u8_color(255, 255, 0);

pub const SIZE: UVec2 = UVec2::from_array([50, 40]);
pub const WALL_TILE: Tile = Tile::new('#', LinearRgba::WHITE, LinearRgba::BLACK);
pub const FLOOR_TILE: Tile = Tile::new('.', DARK_GRAY, LinearRgba::BLACK);
pub const PLAYER_TILE: Tile = Tile::new('@', ALICE_BLUE, LinearRgba::BLACK);
pub const GOLD_TILE: Tile = Tile::new('$', YELLOW, LinearRgba::BLACK);
pub const GOBLIN_TILE: Tile = Tile::new('g', LinearRgba::RED, LinearRgba::BLACK);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TerminalPlugins))
        .insert_resource(GoalMaps {
            maps: [
                DijkstraMap::new(SIZE),
                DijkstraMap::new(SIZE),
                DijkstraMap::new(SIZE),
            ],
        })
        .insert_resource(ShowGoalMap::Chase)
        .insert_resource(PathMap(PathMap2d::new(SIZE)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input,
                update_dijkstra.run_if(player_moved),
                move_goblins.run_if(player_moved),
                draw.run_if(player_moved.or(resource_changed::<ShowGoalMap>)),
            )
                .chain(),
        )
        .run();
}

/// Maintain a separate map for each behavior
#[derive(Resource)]
pub struct GoalMaps {
    maps: [DijkstraMap; 3],
}

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum ShowGoalMap {
    None,
    Chase,
    Flee,
    GoldOrChase,
}

#[derive(Resource, Deref, DerefMut)]
pub struct PathMap(PathMap2d);

#[derive(Component, Clone, Copy)]
pub enum Behavior {
    /// Chase the player
    Chase,
    /// Run from the player
    Flee,
    /// Chase the player, unless there's gold nearby
    GoldOrChase,
}

#[derive(Component)]
pub struct Goblin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Gold;

#[derive(Component)]
pub struct Renderable(Tile);

#[derive(Component, Deref, DerefMut)]
pub struct Position(pub IVec2);

fn setup(mut commands: Commands, mut map: ResMut<PathMap>) {
    commands.spawn(Terminal::new(SIZE));
    commands.spawn(TerminalCamera::new());
    build_walls(&mut map.0);
    let open_points: Vec<_> = map
        .iter_grid_points()
        .filter(|p| !map.is_obstacle(*p))
        .collect();
    let mut rng = rand::thread_rng();
    let mut random_point = || *open_points.choose(&mut rng).unwrap();
    commands.spawn((Player, Renderable(PLAYER_TILE), Position(random_point())));

    // Give each goblin a unique behavior.
    let behaviors = [Behavior::Chase, Behavior::Flee, Behavior::GoldOrChase];
    (0..3).for_each(|i| {
        commands.spawn((
            Goblin,
            Renderable(GOBLIN_TILE),
            //Behavior::Flee,
            behaviors[i],
            Position(random_point()),
        ));
    });

    for _ in 0..3 {
        commands.spawn((Gold, Renderable(GOLD_TILE), Position(random_point())));
    }
}

fn input(
    key: Res<ButtonInput<KeyCode>>,
    mut q_player: Query<&mut Position, With<Player>>,
    mut show: ResMut<ShowGoalMap>,
    pathmap: Res<PathMap>,
) {
    let Ok(mut player) = q_player.get_single_mut() else {
        return;
    };

    if key.just_pressed(KeyCode::Tab) {
        *show = match *show {
            ShowGoalMap::None => ShowGoalMap::Chase,
            ShowGoalMap::Chase => ShowGoalMap::Flee,
            ShowGoalMap::Flee => ShowGoalMap::GoldOrChase,
            ShowGoalMap::GoldOrChase => ShowGoalMap::None,
        };
    }

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

fn update_dijkstra(
    q_player: Query<&Position, With<Player>>,
    q_gold: Query<&Position, With<Gold>>,
    pathing: Res<PathMap>,
    mut goals: ResMut<GoalMaps>,
) {
    let chase = &mut goals.maps[0];
    chase.clear_all();
    let player_xy = q_player.single().0;
    chase.add_goal(player_xy, 1.0);
    chase.recalculate(&pathing.0);

    // let flee = &mut goals.maps[1];
    // flee.clear_all();
    // flee.add_goal(player_xy, 1.0);
    // flee.recalculate(&pathing.0);
    // flee.apply_operation(|v| v * -2.5);
    // flee.recalculate(&pathing.0);

    // let greed = &mut goals.maps[2];
    // greed.clear_all();
    // greed.add_goal(player_xy, 1.0);
    // for p in q_gold.iter().map(|p| p.0) {
    //     greed.add_goal(p, 4.0);
    // }
    // greed.recalculate(&pathing.0);
}

fn move_goblins(
    mut q_goblin: Query<(&mut Position, &Behavior), With<Goblin>>,
    goals: Res<GoalMaps>,
) {
    for (mut p, behavior) in &mut q_goblin {
        let map = match behavior {
            Behavior::Chase => &goals.maps[0],
            Behavior::Flee => &goals.maps[1],
            Behavior::GoldOrChase => &goals.maps[2],
        };
        let Some(next) = map.exits(p.0).next() else {
            continue;
        };
        p.0 = next;
    }
}

fn draw(
    mut q_term: Query<&mut Terminal>,
    show: Res<ShowGoalMap>,
    goals: Res<GoalMaps>,
    pathing: Res<PathMap>,
    q_renderables: Query<(&Renderable, &Position)>,
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

    if let Some(goals) = match *show {
        ShowGoalMap::None => None,
        ShowGoalMap::Chase => Some(&goals.maps[0]),
        ShowGoalMap::Flee => Some(&goals.maps[1]),
        ShowGoalMap::GoldOrChase => Some(&goals.maps[2]),
    } {
        for ((p, tile), v) in term.iter_xy_mut().zip(goals.values().iter()) {
            if pathing.is_obstacle(p) {
                continue;
            }
            if (-9.9..=9.9).contains(v) {
                let ch = char::from_digit((*v as i32).unsigned_abs(), 10).unwrap();
                let t = (v + 10.0) / 20.0;
                let bg = LinearRgba::lerp(LinearRgba::GREEN, LinearRgba::RED, t).with_alpha(0.5);
                tile.glyph = ch;
                tile.bg_color = bg;
                tile.fg_color = LinearRgba::BLACK;
            };
        }
    }

    // for (i, (tile, v)) in term
    //     .tiles_mut()
    //     .iter_mut()
    //     .zip(goals.0.float_grid().values().iter())
    //     .enumerate()
    // {
    //     if map.obstacle_grid().get_index(i) {
    //         tile.glyph = '#';
    //         continue;
    //     }
    //     let v = *v / pathmap::CARDINAL_COST as f32;
    //     let ch = if (-9.9..=9.9).contains(&v) {
    //         char::from_digit((v as i32).unsigned_abs(), 10).unwrap()
    //     } else {
    //         'F'
    //     };

    //     tile.glyph = ch;

    //     let t = (v + 10.0) / 20.0;
    //     let bg = color_lerp(Color::GREEN, Color::RED, t);
    //     tile.fg_color = Color::BLACK;
    //     tile.bg_color = bg;
    // }

    for (r, p) in &q_renderables {
        term.put_tile(p.0, r.0);
    }

    // let fg = Color::YELLOW_GREEN;
    // let bg = Color::BLACK;
    // term.put_string([0, 0], "Left click to add 'value' to a goal".fg(fg).bg(bg));
    // term.put_string(
    //     [0, 1],
    //     "Right click to clear a goal from a tile.".fg(fg).bg(bg),
    // );
}

fn build_walls(walls: &mut PathMap2d) {
    let fbm = Fbm::new()
        .set_octaves(16)
        .set_frequency(1.5)
        .set_lacunarity(3.0)
        .set_persistence(0.7);
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
