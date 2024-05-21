
use std::default;

use bevy::{prelude::*, utils::HashMap};
use bevy_editor_pls::prelude::*;
use bevy_tweening::TweeningPlugin;
mod player;

fn main() {
    App::default()
        .add_plugins((DefaultPlugins))
        .add_plugins(DevelopmentPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(TweeningPlugin)
        .add_systems(Startup, (setup))
        .add_systems(Update, (score_point))
        .insert_resource(Controls::default())
        .insert_resource(Points::default())
        .run();
}


struct DevelopmentPlugin;
impl Plugin for DevelopmentPlugin{
    fn build(&self, app: &mut App){
        app.add_plugins(EditorPlugin::default());
    }
}


#[derive(Debug)]
struct ControlPerPlayer{
    right: KeyCode,
    left: KeyCode,
    attack: KeyCode,
}

impl Default for ControlPerPlayer{
    fn default() -> Self{
        ControlPerPlayer{
            right: KeyCode::ArrowRight,
            left: KeyCode::ArrowLeft,
            attack: KeyCode::ArrowDown,
        }
    }
}

#[derive(Debug, Resource, Default)]
struct Controls{
    control_map: HashMap<Entity, ControlPerPlayer>,
}


#[derive(Debug, Resource, Default)]
struct Points{
    player_1: u32,
    player_2: u32,
}




fn setup(mut commands: Commands){
    commands.spawn(Camera2dBundle::default());

    commands.spawn(NodeBundle{
        style: Style{
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_content: AlignContent::FlexStart,
            ..default()
        },
        ..default()
    }).with_children(
            |parent| {
                parent.spawn(Text2dBundle{
                    text: Text{
                        sections: vec![TextSection{
                            value: "Player 1: 0".to_string(),
                            style: TextStyle{
                                font_size: 40.0,
                                color: Color::WHITE,
                                font: Default::default(),
                            },
                        }],
                        justify: JustifyText::Left,
                        ..Default::default()
                    },
                    ..Default::default()
                
                });
                parent.spawn(Text2dBundle{
                    text: Text{
                        sections: vec![TextSection{
                            value: "Player 2: 0".to_string(),
                            style: TextStyle{
                                font_size: 40.0,
                                color: Color::WHITE,
                                font: Default::default(),
                            },
                        }],
                        justify: JustifyText::Right,
                        ..Default::default()
                    },
                    ..Default::default()
                
                });
            }
    );
}

fn score_point(
    mut ev_player_state_change: EventReader<player::PlayerStateChangeEvent>,
    mut points: ResMut<Points>,
    query: Query<(&player::Player)>,
){
    for event in ev_player_state_change.read(){
        if event.1 == player::PlayerState::Dead{
            if let Ok(player) = query.get(event.0){
                match player.player_number{
                    1 => points.player_2 += 1,
                    2 => points.player_1 += 1,
                    _ => (),
                }
            }
        }
    }
}