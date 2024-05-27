use std::time::Duration;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_tweening::{lens::TransformPositionLens, Animator, EaseFunction, Tween};

use crate::Controls;
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AttackEvent>()
            .add_event::<PlayerStateChangeEvent>()
            .add_event::<ClashEvent>()
            .add_event::<ResetPlayers>()
            .register_type::<Player>()
            .register_type::<ClashCounter>()
            .insert_resource(ClashCounter::default())
            .add_systems(Startup, (spawn_players, reset_player))
            .add_systems(
                Update,
                (
                    player_timer_update,
                    reset_player_function,
                    move_player,
                    reset_player,
                    check_attack_hit,
                    update_player_color,
                    clash_players,
                    push_back_player_with_clash,
                ),
            );
    }
}

#[derive(Resource, Default, Reflect)]
pub struct ClashCounter(pub u32);

#[derive(Event)]
pub struct AttackEvent(Entity);

#[derive(Event)]
pub struct PlayerStateChangeEvent(pub Entity, pub PlayerState);

#[derive(Event)]
struct ClashEvent(Entity, Entity);

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Reflect)]
pub enum PlayerState {
    #[default]
    Alive,
    Dead,
    TakingDamage,
    Clashing,
    Wiff,
}

#[derive(Debug, Component, Reflect)]
pub struct Player {
    pub player_number: u8,
    pub state: PlayerState,
    pub color_mesh_handle: Handle<ColorMaterial>,
    pub attack_timer: Timer,
    //200ms timer
    pub parry_timer: Timer,
    pub clashing_timer: Timer,
    pub color: Color,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            player_number: 0,
            state: PlayerState::Alive,
            color_mesh_handle: Default::default(),
            attack_timer: Timer::from_seconds(1.0, TimerMode::Once),
            parry_timer: Timer::from_seconds(0.016 * 4.0, TimerMode::Once),
            clashing_timer: Timer::from_seconds(1.0, TimerMode::Once),
            color: Color::rgb(1.0, 0.7, 0.6),
        }
    }
}

fn reset_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ev_reset: EventWriter<ResetPlayers>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        ev_reset.send(ResetPlayers);
    }
}
#[derive(Event)]
pub struct ResetPlayers;

fn reset_player_function(
    mut ev_reset: EventReader<ResetPlayers>,
    mut query: Query<(&mut Player, Entity, &mut Transform)>,
    mut ev_player_state_change: EventWriter<PlayerStateChangeEvent>,
) {
    for _ in ev_reset.read() {
        let mut p1 = None;
        let mut p2 = None;
        for p in query.iter_mut() {
            if p.0.player_number == 1 {
                p1 = Some(p);
            } else {
                p2 = Some(p);
            }
        }
        if p1.is_none() || p2.is_none() {
            return;
        }
        let (mut player1, entity1, mut t1) = p1.unwrap();
        let (mut player2, entity2, mut t2) = p2.unwrap();
        player1.state = PlayerState::Alive;
        t1.translation.x = -300.0;
        t2.translation.x = 300.0;
        player2.state = PlayerState::Alive;
        let duration = player1.attack_timer.duration();
        player1.attack_timer.set_elapsed(duration);
        player2.attack_timer.set_elapsed(duration);
        ev_player_state_change.send(PlayerStateChangeEvent(entity1, PlayerState::Alive));
        ev_player_state_change.send(PlayerStateChangeEvent(entity2, PlayerState::Alive));
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controls: Res<super::Controls>,
    mut query: Query<(&mut Player, &mut Transform, Entity)>,
    mut ev_attack: EventWriter<AttackEvent>,
    clash_counter: Res<ClashCounter>,
    camera_query: Query<(&Camera, &OrthographicProjection), With<Transform>>,
    time: Res<Time>,
) {
    let move_amount: f32 = 650.0 * time.delta_seconds();

    if let Some((_, ortho_proj)) = camera_query.iter().find(|cam| cam.0.is_active).take() {
        let bounds = ortho_proj.area;

        for (mut player, mut transform, entity) in query.iter_mut() {
            if let Some(controls) = controls.control_map.get(&entity) {
                if matches!(player.state, PlayerState::Alive | PlayerState::TakingDamage) {
                    if keyboard_input.pressed(controls.right) {
                        transform.translation.x += move_amount + (clash_counter.0 as f32);
                    }
                    if keyboard_input.pressed(controls.left) {
                        transform.translation.x += -(move_amount + (clash_counter.0 as f32));
                    }
                }
                if matches!(player.state, PlayerState::Wiff) {
                    if keyboard_input.pressed(controls.right) {
                        transform.translation.x += move_amount * 0.1;
                    }
                    if keyboard_input.pressed(controls.left) {
                        transform.translation.x += -move_amount * 0.1;
                    }
                }
                if matches!(player.state, PlayerState::Alive | PlayerState::TakingDamage)
                    && keyboard_input.just_pressed(controls.attack)
                {
                    player_attack(&mut ev_attack, &mut player, &entity);
                }

                transform.translation.x = transform
                    .translation
                    .x
                    .clamp(bounds.min.x + 50., bounds.max.x - 50.);
            }
        }
    }
}

fn player_attack(ev_attack: &mut EventWriter<AttackEvent>, player: &mut Player, entity: &Entity) {
    println!(
        "Player {:?} attack timer: {:?}",
        player.player_number,
        player.attack_timer.elapsed_secs()
    );
    if player.attack_timer.finished() {
        println!("Player {:?} attacking!", player.player_number);
        player.attack_timer.reset();
        ev_attack.send(AttackEvent(*entity));
    }
}

fn check_attack_hit(
    mut ev_attack: EventReader<AttackEvent>,
    mut ev_clash: EventWriter<ClashEvent>,
    mut ev_player_state_change: EventWriter<PlayerStateChangeEvent>,
    mut query: Query<(&mut Player, &Transform, Entity)>,
) {
    for ev in ev_attack.read() {
        let mut attacking_player = None;
        let mut defending_player = None;

        for p in query.iter_mut() {
            if p.2 == ev.0 {
                attacking_player = Some(p);
            } else {
                defending_player = Some(p);
            }
        }

        if attacking_player.is_none() || defending_player.is_none() {
            continue;
        }

        let mut attacking_player = attacking_player.unwrap();
        let mut defending_player = defending_player.unwrap();

        if attacking_player.0.state == PlayerState::Dead {
            return;
        }

        if attacking_player
            .1
            .translation
            .distance(defending_player.1.translation)
            < 100.0
        {
            if attacking_player.0.state == PlayerState::TakingDamage {
                println!("Player {:?} parried!", attacking_player.0.player_number);
                attacking_player.0.state = PlayerState::Clashing;
                defending_player.0.state = PlayerState::Clashing;
                ev_player_state_change.send(PlayerStateChangeEvent(
                    attacking_player.2,
                    PlayerState::Clashing,
                ));
                ev_player_state_change.send(PlayerStateChangeEvent(
                    defending_player.2,
                    PlayerState::Clashing,
                ));
                ev_clash.send(ClashEvent(attacking_player.2, defending_player.2));
            } else {
                match defending_player.0.state {
                    PlayerState::Alive | PlayerState::Wiff => {
                        defending_player.0.state = PlayerState::TakingDamage;
                        defending_player.0.parry_timer.reset();
                        ev_player_state_change.send(PlayerStateChangeEvent(
                            defending_player.2,
                            PlayerState::TakingDamage,
                        ));
                    }
                    _ => {}
                }
            }
        } else {
            attacking_player.0.state = PlayerState::Wiff;
            ev_player_state_change.send(PlayerStateChangeEvent(
                attacking_player.2,
                PlayerState::Wiff,
            ));
        }
    }
}

#[derive(Component)]
struct ClashPushback {
    pub timer: Timer,
}

fn clash_players(
    mut ev_clash: EventReader<ClashEvent>,
    mut query: Query<(&mut Player, &mut Transform)>,
    mut commands: Commands,
    mut clash_counter: ResMut<ClashCounter>,
) {
    for ev in ev_clash.read() {
        let mut p1 = None;
        let mut p2 = None;
        for p in query.iter_mut() {
            if p.0.player_number == 1 {
                p1 = Some(p);
            } else {
                p2 = Some(p);
            }
        }

        if p1.is_none() || p2.is_none() {
            continue;
        }

        let (mut p1, t1) = p1.unwrap();
        let (mut p2, t2) = p2.unwrap();

        p1.state = PlayerState::Clashing;
        p2.state = PlayerState::Clashing;

        let p1_offset = if t1.translation.x < t2.translation.x {
            -1
        } else {
            1
        };
        let p2_offset = -p1_offset;

        // commands.entity(ev.0).insert(ClashPushback{offset: p1_offset, timer: Timer::from_seconds(0.8, TimerMode::Once)});
        // commands.entity(ev.1).insert(ClashPushback{offset: p2_offset, timer: Timer::from_seconds(0.8, TimerMode::Once)});
        clash_counter.0 += 1;
        commands.entity(ev.0).insert(Animator::new(Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(0.2),
            TransformPositionLens {
                start: t1.translation,
                end: Vec3::new(
                    t1.translation.x + (p1_offset as f32 * 300.0),
                    t1.translation.y,
                    t1.translation.z,
                ),
            },
        )));

        commands.entity(ev.1).insert(Animator::new(Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_secs_f32(0.2),
            TransformPositionLens {
                start: t2.translation,
                end: Vec3::new(
                    t2.translation.x + (p2_offset as f32 * 300.0),
                    t2.translation.y,
                    t2.translation.z,
                ),
            },
        )));
    }
}

fn push_back_player_with_clash(
    mut query: Query<(&mut Player, &mut ClashPushback)>,
    time: Res<Time>,
) {
    for (mut player, mut clash_pushback) in query.iter_mut() {
        clash_pushback.timer.tick(time.delta());
        if clash_pushback.timer.finished() {
            player.state = PlayerState::Alive;
        }
    }
}

fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut controls: ResMut<Controls>,
) {
    // Player 1
    let p_1_color = materials.add(Color::rgb(1.0, 0.7, 0.6));
    let mut binding = commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(100.0, 100.0))),
        material: p_1_color.clone(),
        transform: Transform::from_xyz(-300.0, 0.0, 0.0),
        ..default()
    });
    let p1_e = binding.insert(Player {
        player_number: 1,
        color_mesh_handle: p_1_color,
        color: Color::rgb(1.0, 0.7, 0.6),
        ..default()
    });

    // player 1 default controls
    controls.control_map.insert(
        p1_e.id(),
        crate::ControlPerPlayer {
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            attack: KeyCode::KeyS,
        },
    );

    // Player 2
    let p_2_color = materials.add(Color::rgb(0.8, 1.0, 0.6));
    let mut binding = commands.spawn(MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Rectangle::new(100.0, 100.0))),
        material: p_2_color.clone(),
        transform: Transform::from_xyz(300.0, 0.0, 0.0),
        ..default()
    });
    let p2_e = binding.insert(Player {
        player_number: 2,
        color_mesh_handle: p_2_color,
        color: Color::rgb(0.8, 1.0, 0.6),
        ..default()
    });

    // player 2 default controls
    controls.control_map.insert(
        p2_e.id(),
        crate::ControlPerPlayer {
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
            attack: KeyCode::ArrowDown,
        },
    );
}

fn player_timer_update(
    time: Res<Time>,
    mut query: Query<(&mut Player, Entity)>,
    mut ev_player_state_change: EventWriter<PlayerStateChangeEvent>,
) {
    for (mut player, entity) in query.iter_mut() {
        player.parry_timer.tick(time.delta());
        player.attack_timer.tick(time.delta());
        player.clashing_timer.tick(time.delta());
        if player.parry_timer.just_finished() && player.state == PlayerState::TakingDamage {
            player.state = PlayerState::Dead;
            ev_player_state_change.send(PlayerStateChangeEvent(entity, PlayerState::Dead));
        }
        if player.attack_timer.finished() && player.state == PlayerState::Wiff {
            player.state = PlayerState::Alive;
            ev_player_state_change.send(PlayerStateChangeEvent(entity, PlayerState::Alive));
        }
        if player.clashing_timer.finished() && player.state == PlayerState::Clashing {
            player.state = PlayerState::Alive;
            ev_player_state_change.send(PlayerStateChangeEvent(entity, PlayerState::Alive));
        }
    }
}

fn update_player_color(
    mut query: Query<(&mut Player, &mut Transform)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (player, mut transform) in query.iter_mut() {
        if !player.attack_timer.finished() && player.state != PlayerState::Dead {
            materials.get_mut(&player.color_mesh_handle).unwrap().color = Color::rgb(
                player.color.r() * 0.5,
                player.color.g() * 0.5,
                player.color.b() * 0.5,
            );
        } else {
            match player.state {
                PlayerState::Dead => {
                    materials.get_mut(&player.color_mesh_handle).unwrap().color =
                        Color::rgb(0.0, 0.0, 0.0);
                    transform.translation.z = -(player.player_number as f32);
                }
                PlayerState::Alive => {
                    materials.get_mut(&player.color_mesh_handle).unwrap().color = player.color;
                    transform.translation.z = player.player_number as f32;
                }
                PlayerState::Clashing => {
                    println!("Clashing!");
                    materials.get_mut(&player.color_mesh_handle).unwrap().color =
                        Color::rgb(1.0, 1.0, 1.0);
                }
                PlayerState::Wiff => {}
                PlayerState::TakingDamage => {}
            }
        }
    }
}
