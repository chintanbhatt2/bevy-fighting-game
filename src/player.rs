use bevy::{ecs::query, prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};

use crate::{player, Controls};
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<AttackEvent>()
            .add_event::<PlayerStateChangeEvent>()
            .add_event::<ClashEvent>()
            .add_systems(Startup, (spawn_players))
            .add_systems(Update, (move_player, player_taking_damage, reset_player, check_attack_hit, update_player_color, clash_players));
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum PlayerState{
    #[default]
    Alive,
    Dead,
    TakingDamage,   
    Clashing,
    Wiff,
}

#[derive(Debug, Component)]
pub struct Player{
    pub player_number: u8,
    pub state: PlayerState,
    pub color_mesh_handle: Handle<ColorMaterial>,
    pub attack_timer: Timer,
    //200ms timer
    pub parry_timer: Timer,
}

impl Default for Player{
    fn default() -> Self{
        Player{
            player_number: 0,
            state: PlayerState::Alive,
            color_mesh_handle: Default::default(),
            attack_timer: Timer::from_seconds(2.0, TimerMode::Once),
            parry_timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }

}

#[derive(Event)]
pub struct AttackEvent(Entity);


fn reset_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, Entity)>,
    mut ev_player_state_change: EventWriter<PlayerStateChangeEvent>,
){
    if keyboard_input.just_pressed(KeyCode::KeyR){
        for mut p in query.iter_mut(){
            p.0.state = PlayerState::Alive;
            ev_player_state_change.send(PlayerStateChangeEvent(p.1, PlayerState::Alive));
        }
    }
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    controls: Res<super::Controls>,
    mut query: Query<(&Player, &mut Transform, Entity)>,
    mut ev_attack: EventWriter<AttackEvent>,
){

    const MOVE_AMOUNT: f32 = 10.0;

    for (player, mut transform, entity) in query.iter_mut(){
        if let Some(controls) = controls.control_map.get(&entity){
            if matches!(player.state, PlayerState::Alive | PlayerState::TakingDamage | PlayerState::Wiff | PlayerState::Clashing) {
                if keyboard_input.pressed(controls.right) {
                    transform.translation.x += MOVE_AMOUNT;
                }
                if keyboard_input.pressed(controls.left){
                    transform.translation.x += -MOVE_AMOUNT;
                }
            }
            if matches!(player.state, PlayerState::Alive | PlayerState::TakingDamage) {
                if player.attack_timer.finished(){
                    if keyboard_input.pressed(controls.attack){
                        ev_attack.send(AttackEvent(entity));
                    }
                }
            }
        }
    }
    
}


#[derive(Event)]
pub struct PlayerStateChangeEvent(pub Entity, pub PlayerState);

#[derive(Event)]
struct ClashEvent;

fn check_attack_hit(
    mut ev_attack: EventReader<AttackEvent>,
    mut ev_player_state_change: EventWriter<PlayerStateChangeEvent>,
    mut query: Query<( &mut Player, &Transform, Entity )>,
){
    
    let mut player_1 = None;
    let mut player_2 = None;
    for q in query.iter_mut(){
        if q.0.player_number == 1 && q.0.state == PlayerState::Alive{
            player_1 = Some(q);
        }
        else if q.0.player_number == 2 && q.0.state == PlayerState::Alive{
            player_2 = Some(q);
        }
    }
    if player_1.is_none() || player_2.is_none(){
        return;
    }

    let mut player_1 = player_1.unwrap();
    let mut player_2 = player_2.unwrap();
    
    for ev in ev_attack.read(){
        if ev.0 == player_1.2{
            if (player_1.1.translation.x - player_2.1.translation.x).abs() < 100.0{
                println!("Player 1 hit Player 2");
                player_2.0.state = PlayerState::TakingDamage;
                ev_player_state_change.send(PlayerStateChangeEvent(player_2.2, PlayerState::TakingDamage));
            }
        }
        if ev.0 == player_2.2{
            if (player_2.1.translation.x - player_1.1.translation.x).abs() < 100.0{
                println!("Player 2 hit Player 1");
                player_1.0.state = PlayerState::TakingDamage;
                ev_player_state_change.send(PlayerStateChangeEvent(player_1.2, PlayerState::TakingDamage));
            }
        }
    }
}


fn clash_players(
    mut ev_clash: EventReader<ClashEvent>,
    mut query: Query<(&mut Player, &mut Transform)>,
){
    for _ in ev_clash.read(){
        let mut player_1 = None;
        let mut player_2 = None;
        for q in query.iter_mut(){
            if q.0.player_number == 1 && q.0.state == PlayerState::Alive{
                player_1 = Some(q);
            }
            else if q.0.player_number == 2 && q.0.state == PlayerState::Alive{
                player_2 = Some(q);
            }
        }
        if player_1.is_none() || player_2.is_none(){
            return;
        }
        let mut player_1 = player_1.unwrap();
        let mut player_2 = player_2.unwrap();

        player_1.1.translation.x = -10.0;
        player_2.1.translation.x = 10.0;
    }
}


fn spawn_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut controls: ResMut<Controls>,
){
    // Player 1
    let p_1_color = materials.add(Color::rgb(1.0, 0.7, 0.6));
    let mut binding = commands.spawn(
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(100.0, 100.0))),
            material: p_1_color.clone(),
            transform: Transform::from_xyz(-200.0, 0.0, 0.0),
            ..default()
        }
    );
    let p1_e = binding.insert(Player{player_number: 1, color_mesh_handle: p_1_color, ..default()});

    // player 1 default controls
    controls.control_map.insert(p1_e.id(), crate::ControlPerPlayer {
        left: KeyCode::KeyA,
        right: KeyCode::KeyD,
        attack: KeyCode::KeyS,
    });

    // Player 2
    let p_2_color = materials.add(Color::rgb(0.8, 1.0, 0.6));
    let mut binding = commands.spawn(
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(100.0, 100.0))),
            material: p_2_color.clone(),
            transform: Transform::from_xyz(200.0, 0.0, 0.0),
            ..default()
        }
    );
    let p2_e = binding.insert(Player{player_number: 2, color_mesh_handle: p_2_color, ..default()});

        // player 2 default controls
    controls.control_map.insert(p2_e.id(), crate::ControlPerPlayer {
        left: KeyCode::ArrowLeft,
        right: KeyCode::ArrowRight,
        attack: KeyCode::ArrowDown,
    });
}


fn player_taking_damage(
    mut query: Query<(&mut Player, Entity)>,
    mut ev_player_state_change: EventWriter<PlayerStateChangeEvent>,
){
    for (mut player, entity) in query.iter_mut(){
        if player.state == PlayerState::TakingDamage && player.parry_timer.finished(){
            player.state = PlayerState::Dead;
            ev_player_state_change.send(PlayerStateChangeEvent(entity, PlayerState::Dead));
        }
    }
}



fn update_player_color(
    mut query: Query<(&mut Player, &mut Transform)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_state_event: EventReader<PlayerStateChangeEvent>,
){
    for event in player_state_event.read(){
        if let Ok((mut player, mut transform)) = query.get_mut(event.0){
            player.state = event.1;
            match player.state{
                PlayerState::Dead => {
                    materials.get_mut(&player.color_mesh_handle).unwrap().color = Color::rgb(0.0, 0.0, 0.0);
                    transform.translation.z = -(player.player_number as f32);
                }
                PlayerState::Alive => {
                    if player.player_number == 1{
                        materials.get_mut(&player.color_mesh_handle).unwrap().color = Color::rgb(1.0, 0.7, 0.6);
                    } else {
                        materials.get_mut(&player.color_mesh_handle).unwrap().color = Color::rgb(0.8, 1.0, 0.6);
                    }
                    transform.translation.z = player.player_number as f32;
                }
                PlayerState::Clashing => {
                    println!("Clashing!");
                    materials.get_mut(&player.color_mesh_handle).unwrap().color = Color::rgb(1.0, 1.0, 1.0);
                }
                PlayerState::Wiff => {
                    if player.player_number == 1{
                        materials.get_mut(&player.color_mesh_handle).unwrap().color = Color::rgb(1.0 * 0.8, 0.7* 0.8, 0.6* 0.8);
                    } else {
                        materials.get_mut(&player.color_mesh_handle).unwrap().color = Color::rgb(0.8* 0.8, 1.0* 0.8, 0.6* 0.8);
                    }
                }
                PlayerState::TakingDamage => {}
            }
        }
    }

}