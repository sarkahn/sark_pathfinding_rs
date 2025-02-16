use bevy::{color::palettes::css, math::VectorSpace, prelude::*};
use bevy_ascii_terminal::*;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal,
};
use rand::seq::SliceRandom;
use sark_pathfinding::*;

pub const SIZE: UVec2 = UVec2::from_array([50, 40]);
pub const WALL_TILE: Tile = Tile::new('#', color::WHITE, color::BLACK);
pub const FLOOR_TILE: Tile = Tile::new('.', color::DARK_GRAY, color::BLACK);
pub const PLAYER_TILE: Tile = Tile::new('@', color::BLANCHED_ALMOND, color::BLACK);
pub const GOLD_TILE: Tile = Tile::new('$', color::YELLOW, color::BLACK);
pub const GOBLIN_TILE: Tile = Tile::new('g', color::DARK_GREEN, color::BLACK);
pub const OGRE_TILE: Tile = Tile::new('O', color::DARK_RED, color::BLACK);

#[derive(Resource, Clone, Copy, PartialEq, Eq)]
pub enum ShowGoals {
    None,
    Goblin,
    Ogre,
}

#[derive(Resource, Deref, DerefMut)]
pub struct PathMap(PathMap2d);

#[derive(Component)]
pub struct Goblin;

#[derive(Component)]
pub struct Ogre;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Gold;

#[derive(Component)]
pub struct Renderable(Tile);

#[derive(Component, Deref, DerefMut)]
pub struct Position(pub IVec2);

#[derive(Component, Deref, DerefMut)]
pub struct GoalMap(DijkstraMap);

#[derive(Resource)]
struct GoblinGoals {
    greed: DijkstraMap,
    cowardice: DijkstraMap,
    goal: DijkstraMap,
}

#[derive(Resource, Deref, DerefMut)]
struct OgreGoals(DijkstraMap);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TerminalPlugins))
        .insert_resource(PathMap(PathMap2d::new(SIZE)))
        .insert_resource(ShowGoals::None)
        .insert_resource(GoblinGoals {
            greed: DijkstraMap::new(SIZE),
            cowardice: DijkstraMap::new(SIZE),
            goal: DijkstraMap::new(SIZE),
        })
        .insert_resource(OgreGoals(DijkstraMap::new(SIZE)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input,
                update_goals.run_if(player_moved),
                move_monsters.run_if(player_moved),
                draw.run_if(player_moved.or(resource_changed::<ShowGoals>)),
            )
                .chain(),
        )
        .run();
}

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

    for _ in 0..5 {
        commands.spawn((
            Goblin,
            Renderable(GOBLIN_TILE),
            Position(random_point()),
            GoalMap(DijkstraMap::new(SIZE)),
        ));
    }

    for _ in 0..2 {
        commands.spawn((
            Ogre,
            Renderable(OGRE_TILE),
            Position(random_point()),
            GoalMap(DijkstraMap::new(SIZE)),
        ));
    }

    for _ in 0..3 {
        commands.spawn((Gold, Renderable(GOLD_TILE), Position(random_point())));
    }
}

fn input(
    key: Res<ButtonInput<KeyCode>>,
    mut q_player: Query<&mut Position, With<Player>>,
    mut show: ResMut<ShowGoals>,
    mut pathmap: ResMut<PathMap>,
) {
    let Ok(mut player) = q_player.get_single_mut() else {
        return;
    };

    if key.just_pressed(KeyCode::Tab) {
        *show = match *show {
            ShowGoals::None => ShowGoals::Goblin,
            ShowGoals::Goblin => ShowGoals::Ogre,
            ShowGoals::Ogre => ShowGoals::None,
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
    if !pathmap.in_bounds(next) || pathmap.is_obstacle(next) {
        return;
    }
    pathmap.0.move_obstacle(player.0, next);
    player.0 = next;
}

fn player_moved(q_player: Query<&Position, (With<Player>, Changed<Position>)>) -> bool {
    !q_player.is_empty()
}

#[allow(clippy::type_complexity)]
fn update_goals(
    q_ogres: Query<&Position, With<Ogre>>,
    q_goblins: Query<&Position, (With<Goblin>, Without<Ogre>)>,
    q_gold: Query<&Position, (With<Gold>, Without<Ogre>, Without<Goblin>)>,
    q_player: Query<&Position, (With<Player>, Without<Goblin>, Without<Ogre>, Without<Gold>)>,
    mut goblin_goals: ResMut<GoblinGoals>,
    mut ogre_goals: ResMut<OgreGoals>,
    pathing: Res<PathMap>,
) {
    let player_pos = q_player.single().0;

    ogre_goals.clear_all();
    for gob_pos in &q_goblins {
        ogre_goals.add_goal(gob_pos.0, 5.0);
    }
    ogre_goals.add_goal(player_pos, 1.0);
    ogre_goals.recalculate(&pathing.0);

    goblin_goals.greed.clear_all();
    goblin_goals.cowardice.clear_all();

    for gold_pos in &q_gold {
        goblin_goals.greed.add_goal(gold_pos.0, 5.0);
    }
    goblin_goals.greed.recalculate(&pathing.0);

    // Goblins really hate ogres
    for ogre_pos in &q_ogres {
        goblin_goals.cowardice.add_goal(ogre_pos.0, -2.0);
    }
    // And kinda hate players
    goblin_goals.cowardice.add_goal(player_pos, -1.0);

    //println!("GOALS preflip {:?}", goblin_goals.cowardice.values());
    goblin_goals.cowardice.recalculate(&pathing.0);
    goblin_goals.cowardice.apply_operation(|f| f * -1.2);

    // for gold_pos in &q_gold {
    //     goblin_goals.cowardice.set_goal(gold_pos.0, -15.0);
    // }

    goblin_goals.cowardice.recalculate(&pathing.0);

    // println!("GOALS postflip {:?}", goblin_goals.cowardice.values());
}

fn move_monsters(
    mut q_goblins: Query<&mut Position, With<Goblin>>,
    mut q_ogres: Query<&mut Position, (With<Ogre>, Without<Goblin>)>,
    mut pathing: ResMut<PathMap>,
    goblin_goals: Res<GoblinGoals>,
    ogres_goals: Res<OgreGoals>,
) {
    for mut p in &mut q_goblins {
        if let Some(next) = goblin_goals.cowardice.next_lowest(p.0, &pathing.0) {
            pathing.move_obstacle(p.0, next);
            *p = Position(next);
        }
        // let mut exits = goblin_goals.cowardice.exits(p.0, &pathing.0);
        // if let Some(next) = exits.next() {
        //     pathing.move_obstacle(p.0, next);
        //     *p = Position(next);
        // }
    }

    // for mut p in &mut q_ogres {
    //     let mut exits = ogres_goals.exits(p.0);
    //     if let Some(next) = exits.next() {
    //         pathing.move_obstacle(p.0, next);
    //         *p = Position(next);
    //     }
    // }
}

fn draw(
    mut q_term: Query<&mut Terminal>,
    show: Res<ShowGoals>,
    pathing: Res<PathMap>,
    q_renderables: Query<(&Renderable, &Position)>,
    goblin_goals: Res<GoblinGoals>,
    ogre_goals: Res<OgreGoals>,
) {
    let mut term = q_term.single_mut();
    term.clear();
    let count = term.tile_count();
    let tiles = term.tiles_mut();

    (0..count).for_each(|i| {
        if pathing.obstacle_grid().value_from_index(i) {
            tiles[i] = WALL_TILE;
        } else {
            tiles[i] = FLOOR_TILE;
        }
    });

    if *show != ShowGoals::None {
        let (map, string) = match *show {
            ShowGoals::Goblin => (&goblin_goals.cowardice, "Showing Goblin goals"),
            ShowGoals::Ogre => (&ogre_goals.0, "Showing Ogre goals"),
            _ => unreachable!(),
        };
        for (p, v) in map.iter_xy() {
            let tile_value = v as i32;
            let digit_value = tile_value.abs() % 62; // 0-61 for 0-9, a-z, A-Z
            let t = tile_value.abs() as f32 / 30.0;

            let distance_char = match digit_value {
                0..=9 => (b'0' + digit_value as u8) as char, // 0-9
                10..=35 => (b'a' + (digit_value - 10) as u8) as char, // Far
                36..=61 => (b'A' + (digit_value - 36) as u8) as char, // Pretty far
                _ => '-',                                    // Very far
            };

            let (bg, fg) = if tile_value >= 0 {
                (
                    Srgba::lerp(css::LIGHT_BLUE, css::DARK_BLUE, t),
                    LinearRgba::BLACK,
                )
            } else {
                (
                    Srgba::lerp(css::LIGHT_BLUE, css::DARK_RED, t),
                    LinearRgba::WHITE,
                )
            };

            let tile = Tile::new(distance_char, fg, bg.into());
            term.put_tile(p, tile);
        }
        // for (p, v) in map.iter_xy() {
        //     let tile_value = v.rem_euclid(16.0);
        //     let t = tile_value / 16.0;

        //     // Select tile character based on integer part
        //     let digit = (tile_value as i32).unsigned_abs();
        //     if let Some(digit_char) = char::from_digit(digit, 16) {
        //         // Cycle colors based on tile value
        //         let bg = match digit {
        //             0..=7 => Srgba::lerp(css::YELLOW, css::PURPLE, t),
        //             _ => Srgba::lerp(css::PURPLE, css::ORANGE, t),
        //         };

        //         let fg = LinearRgba::BLACK;
        //         let tile = Tile::new(digit_char, fg, bg.into());
        //         term.put_tile(p, tile);
        //     }
        // }
        // for (p, v) in map.iter_xy() {
        //     if (-15.9..=15.9).contains(&v) {
        //         let t = (v + 16.0) / 32.0;
        //         let digit = v as i32;
        //         let digit = char::from_digit(digit.unsigned_abs(), 16).unwrap();
        //         let bg = Srgba::lerp(css::YELLOW, css::PURPLE, t);
        //         let fg = LinearRgba::BLACK;
        //         let tile = Tile::new(digit, fg, bg.into());
        //         term.put_tile(p, tile);
        //     } else {
        //         let digit = v as i32 - 16;
        //         let t = (v + 16.0) / 32.0;
        //         if let Some(digit) = char::from_digit(digit.unsigned_abs(), 16) {
        //             let bg = Srgba::lerp(css::PURPLE, css::ORANGE, t);
        //             let fg = LinearRgba::BLACK;
        //             let tile = Tile::new(digit, fg, bg.into());
        //             term.put_tile(p, tile);
        //         }
        //     }
        // }
        term.put_string([0, 0], string.clear_colors());
    }

    // if let Some(goals) = match *show {
    //     ShowGoalMap::None => None,
    //     ShowGoalMap::Chase => Some(&goals.maps[0]),
    //     ShowGoalMap::Flee => Some(&goals.maps[1]),
    //     ShowGoalMap::GoldOrChase => Some(&goals.maps[2]),
    // } {
    //     for ((p, tile), v) in term.iter_xy_mut().zip(goals.values().iter()) {
    //         if pathing.is_obstacle(p) {
    //             continue;
    //         }
    //         if (-9.9..=9.9).contains(v) {
    //             let ch = char::from_digit((*v as i32).unsigned_abs(), 10).unwrap();
    //             let t = (v + 10.0) / 20.0;
    //             let bg = LinearRgba::lerp(LinearRgba::GREEN, LinearRgba::RED, t).with_alpha(0.5);
    //             tile.glyph = ch;
    //             tile.bg_color = bg;
    //             tile.fg_color = LinearRgba::BLACK;
    //         };
    //     }
    // }

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
