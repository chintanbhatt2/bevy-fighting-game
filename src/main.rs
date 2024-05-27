use std::time::Duration;

use bevy::{prelude::*, render::camera::ScalingMode, utils::HashMap, window::PresentMode};
use bevy_tweening::TweeningPlugin;
mod player;

fn main() {
    let window = Window {
        canvas: Some("#bevy".to_string()),
        resizable: true,
        present_mode: PresentMode::AutoNoVsync,
        resize_constraints: WindowResizeConstraints {
            min_width: 800.0,
            min_height: 600.0,
            max_height: 1313.0,
            max_width: 2560.0,
        },
        ..default()
    };

    App::default()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .add_plugins(player::PlayerPlugin)
        .add_plugins(DevelopmentPlugin)
        .add_plugins(TweeningPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (reset_points, score_point, update_ui))
        .register_type::<Controls>()
        .register_type::<Points>()
        .insert_resource(Controls::default())
        .insert_resource(Points::default())
        .run();
}

struct DevelopmentPlugin;
impl Plugin for DevelopmentPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "editor")]
        _app.add_plugins(EditorPlugin::default());
    }
}

#[derive(Debug, Reflect)]
struct ControlPerPlayer {
    right: KeyCode,
    left: KeyCode,
    attack: KeyCode,
}

impl Default for ControlPerPlayer {
    fn default() -> Self {
        ControlPerPlayer {
            right: KeyCode::ArrowRight,
            left: KeyCode::ArrowLeft,
            attack: KeyCode::ArrowDown,
        }
    }
}

#[derive(Debug, Resource, Default, Reflect)]
struct Controls {
    control_map: HashMap<Entity, ControlPerPlayer>,
}

#[derive(Debug, Resource, Reflect, Clone)]
#[reflect(Resource)]
struct Points {
    player_1: u32,
    player_2: u32,
    reset_timer: Timer,
}

impl Default for Points {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(3.0, TimerMode::Once);
        timer.set_elapsed(Duration::from_secs(3));

        Points {
            player_1: 0,
            player_2: 0,
            reset_timer: timer,
        }
    }
}

#[derive(Debug)]
pub enum UIComponent {
    PlayerOneScore,
    ClashCounter,
    PlayerTwoScore,
}

#[derive(Debug, Component)]
pub struct EditableUIComponent(pub UIComponent);

fn update_ui(
    mut query: Query<(&EditableUIComponent, &mut Text)>,
    points: Res<Points>,
    clash_counter: Res<player::ClashCounter>,
) {
    for (component, mut text) in query.iter_mut() {
        match component.0 {
            UIComponent::PlayerOneScore => {
                text.sections[0].value = format!("Player 1: {}", points.player_1);
            }
            UIComponent::ClashCounter => {
                text.sections[0].value = format!("Clash Counter: {}", clash_counter.0);
            }
            UIComponent::PlayerTwoScore => {
                text.sections[0].value = format!("Player 2: {}", points.player_2);
            }
        }
    }
}

fn reset_points(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ev_reset: EventWriter<player::ResetPlayers>,
    mut points: ResMut<Points>,
    mut clash_counter: ResMut<player::ClashCounter>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        points.player_1 = 0;
        points.player_2 = 0;
        clash_counter.0 = 0;
        ev_reset.send(player::ResetPlayers);
    }
}

fn setup(mut commands: Commands) {
    let camera_bundle = Camera2dBundle {
        projection: OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal(1000.),
            scale: 2.5,
            ..default()
        },
        ..default()
    };
    let camera = commands.spawn(camera_bundle).id();

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(80.0),
                height: Val::Percent(80.0),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(50.),
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: "Player 1: 0".to_string(),
                                    style: TextStyle {
                                        font_size: 40.0,
                                        color: Color::WHITE,
                                        font: Default::default(),
                                    },
                                }],
                                justify: JustifyText::Left,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(EditableUIComponent(UIComponent::PlayerOneScore));

                    parent
                        .spawn(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: "Clash Counter: 0".to_string(),
                                    style: TextStyle {
                                        font_size: 40.0,
                                        color: Color::WHITE,
                                        font: Default::default(),
                                    },
                                }],
                                justify: JustifyText::Center,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(EditableUIComponent(UIComponent::ClashCounter));

                    parent
                        .spawn(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: "Player 2: 0".to_string(),
                                    style: TextStyle {
                                        font_size: 40.0,
                                        color: Color::WHITE,
                                        font: Default::default(),
                                    },
                                }],
                                justify: JustifyText::Right,
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .insert(EditableUIComponent(UIComponent::PlayerTwoScore));
                });

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(50.),
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Baseline,
                        flex_direction: FlexDirection::ColumnReverse,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: "Press U to reset points".to_string(),
                                style: TextStyle {
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                    font: Default::default(),
                                },
                            }],
                            justify: JustifyText::Center,
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });
        })
        .insert(TargetCamera(camera));
}

fn score_point(
    mut ev_player_state_change: EventReader<player::PlayerStateChangeEvent>,
    mut points: ResMut<Points>,
    query: Query<&mut player::Player>,
    time: Res<Time>,
    mut ev_reset: EventWriter<player::ResetPlayers>,
) {
    points.reset_timer.tick(time.delta());
    if points.reset_timer.just_finished() {
        println!("Resetting game");
        ev_reset.send(player::ResetPlayers);
    }
    for event in ev_player_state_change.read() {
        if event.1 == player::PlayerState::Dead {
            points.reset_timer.reset();
            if let Ok(player) = query.get(event.0) {
                match player.player_number {
                    1 => points.player_2 += 1,
                    2 => points.player_1 += 1,
                    _ => unreachable!("Invalid player number"),
                }
            }
        }
    }
}
