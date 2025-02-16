use bevy::{color::palettes::css, math::VectorSpace, prelude::*};
use bevy_ascii_terminal::*;
use sark_pathfinding::*;

const MAP_STRING: &str = "
########################################
########################################
########################################
#                #############         #
#                #############         #
#                #############         #
#                #############         #
#                           ##         #
#                ##########            #
#                #############         #
#                ################## ####
######  ########################### ####
######  #######################     ####
######  #################       ########
######  ############      ##############
######  ############ ###################
####   ############# ###################
####   ############# ###################
####   ###########     #################
####   #############    ################
##         ######    #    ##############
####   ### ###### ######   #############
##########        #######   ############
########################################";

#[derive(Resource, Deref, DerefMut)]
pub struct PathMap(PathMap2d);

#[derive(Resource, Deref, DerefMut)]
pub struct BehaviorMap(DijkstraMap);

#[derive(Component)]
pub struct Renderable(Tile);

#[derive(Component, Deref, DerefMut)]
pub struct Position(pub IVec2);

#[derive(Component)]
pub struct Goblin;

#[derive(Component)]
pub struct Player;

#[derive(Resource, Deref, DerefMut)]
pub struct ShowMap(bool);

pub const WALL_TILE: Tile = Tile::new('#', color::WHITE, color::BLACK);
pub const FLOOR_TILE: Tile = Tile::new('.', color::DARK_GRAY, color::BLACK);
pub const PLAYER_TILE: Tile = Tile::new('@', color::BLANCHED_ALMOND, color::BLACK);
pub const GOB_TILE: Tile = Tile::new('g', color::DARK_GREEN, color::BLACK);
pub const PLAYER_SPAWN_POS: IVec2 = IVec2::new(9, 17);
pub const GOB_SPAWN_POS: IVec2 = IVec2::new(15, 14);

fn main() {
    let pathmap = PathMap2d::from_string(MAP_STRING, '#').unwrap();
    let fearmap = DijkstraMap::new(pathmap.size());
    App::new()
        .add_plugins((DefaultPlugins, TerminalPlugins))
        .insert_resource(BehaviorMap(fearmap))
        .insert_resource(PathMap(pathmap))
        .insert_resource(ShowMap(true))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input,
                update_fearmap.run_if(player_moved),
                move_goblin.run_if(player_moved),
                draw.run_if(player_moved.or(resource_changed::<ShowMap>)),
            )
                .chain(),
        )
        .run();
}

fn setup(mut commands: Commands, mut pathmap: ResMut<PathMap>) {
    commands.spawn(Terminal::new(pathmap.size()));
    commands.spawn(TerminalCamera::new());
    commands.spawn((Player, Position(PLAYER_SPAWN_POS), Renderable(PLAYER_TILE)));
    pathmap.0.add_obstacle(PLAYER_SPAWN_POS);
    commands.spawn((Goblin, Position(GOB_SPAWN_POS), Renderable(GOB_TILE)));
    pathmap.0.add_obstacle(GOB_SPAWN_POS);
}

fn input(
    key: Res<ButtonInput<KeyCode>>,
    mut q_player: Query<&mut Position, With<Player>>,
    mut show: ResMut<ShowMap>,
    mut pathmap: ResMut<PathMap>,
) {
    let Ok(mut player) = q_player.get_single_mut() else {
        return;
    };

    if key.just_pressed(KeyCode::Tab) {
        show.0 = !show.0;
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

fn update_fearmap(
    q_player: Query<&Position, With<Player>>,
    pathmap: Res<PathMap>,
    mut fearmap: ResMut<BehaviorMap>,
) {
    //pathmap.print_grid();
    let player = q_player.single();
    fearmap.0.clear_all();
    fearmap.0.add_goal(player.0, 0.0);
    //fearmap.print_grid_values();
    fearmap.recalculate(&pathmap.0);
    fearmap.print_grid_values();
    fearmap.apply_operation(|f| f * -1.2);
    fearmap.recalculate(&pathmap.0);
}

fn move_goblin(
    fearmap: Res<BehaviorMap>,
    mut pathing: ResMut<PathMap>,
    mut q_goblin: Query<&mut Position, With<Goblin>>,
) {
    let mut goblin = q_goblin.single_mut();
    if let Some(next) = fearmap.next_lowest(goblin.0, &pathing.0) {
        pathing.0.move_obstacle(goblin.0, next);
        goblin.0 = next;
    }
}

fn draw(
    mut q_term: Query<&mut Terminal>,
    pathmap: Res<PathMap>,
    fearmap: Res<BehaviorMap>,
    show: Res<ShowMap>,
    q_renderables: Query<(&Renderable, &Position)>,
) {
    let mut term = q_term.single_mut();

    for x in 0..pathmap.width() {
        for y in 0..pathmap.height() {
            let t = if pathmap.is_obstacle([x, y]) {
                WALL_TILE
            } else {
                FLOOR_TILE
            };
            term.put_tile([x, y], t);
        }
    }

    if show.0 {
        for (p, v) in fearmap.iter_xy() {
            let tile_value = v as i32;
            let digit_value = tile_value.abs() % 62; // 0-61 for 0-9, a-z, A-Z
            let t = tile_value.abs() as f32 / 128.0;

            let distance_char = match digit_value {
                0..=9 => (b'0' + digit_value as u8) as char, // 0-9
                10..=35 => (b'a' + (digit_value - 10) as u8) as char, // Far
                36..=61 => (b'A' + (digit_value - 36) as u8) as char, // Pretty far
                _ => '-',                                    // Very far
            };

            let (bg, fg) = if tile_value >= 0 {
                (
                    // Positive values are blue
                    Srgba::lerp(css::LIGHT_BLUE, css::DARK_BLUE, t),
                    LinearRgba::BLACK,
                )
            } else {
                (
                    // Negative values are red
                    Srgba::lerp(css::LIGHT_BLUE, css::DARK_RED, t),
                    LinearRgba::WHITE,
                )
            };

            let tile = Tile::new(distance_char, fg, bg.into());
            term.put_tile(p, tile);
        }
    }

    for (r, p) in &q_renderables {
        term.put_tile(p.0, r.0);
    }

    term.put_string([0, 0], "Tab: Toggle Map Visualization".clear_colors());
}
