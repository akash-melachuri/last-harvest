use bevy::{prelude::*, window::PresentMode};
use rand::prelude::*;

const SCALE: f32 = 1.;
const WIDTH: f32 = 256.;
const HEIGHT: f32 = 240.;
const SCALED_WIDTH: f32 = WIDTH * SCALE;
const SCALED_HEIGHT: f32 = HEIGHT * SCALE;

const PUMPKIN_TIMER: f32 = 1.;
const GHOST_TIMER: f32 = 2.;
const GHOST_SPEED: f32 = 30.;
const PLAYER_SPEED: f32 = 50.;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    Playing,
    GameOver,
}
#[derive(PartialEq)]
enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN,
}

#[derive(Resource)]
struct PumpkinSprite(Handle<Image>);

#[derive(Resource)]
struct PumpkinSpawnTimer(Timer);

#[derive(Resource)]
struct GhostSprite(Handle<Image>);

#[derive(Resource)]
struct GhostSpawnTimer(Timer);

#[derive(Resource)]
struct PickupAudio(Handle<AudioSource>);

#[derive(Resource)]
struct GameOverAudio(Handle<AudioSource>);

#[derive(Resource)]
struct Score(i32);

#[derive(Resource)]
struct PixelFont(Handle<Font>);

#[derive(Component)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct Player {
    direction: Direction,
}

#[derive(Component)]
struct Pumpkin;

#[derive(Component)]
struct Ghost;

#[derive(Component)]
struct Chase;

#[derive(Component)]
struct ScoreText;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "The Last Harvest".to_string(),
                        width: SCALED_WIDTH,
                        height: SCALED_HEIGHT,
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_state(AppState::Playing)
        .add_system_set(SystemSet::on_enter(AppState::Playing).with_system(setup))
        .add_system_set(SystemSet::on_update(AppState::Playing).with_system(spawn_pumpkins))
        .add_system_set(SystemSet::on_update(AppState::Playing).with_system(spawn_ghost))
        .add_system_set(SystemSet::on_update(AppState::Playing).with_system(chase_ai))
        .add_system_set(SystemSet::on_update(AppState::Playing).with_system(player_logic))
        .add_system_set(SystemSet::on_update(AppState::Playing).with_system(pumpkin_collision))
        .add_system_set(SystemSet::on_update(AppState::Playing).with_system(update_score))
        .add_system_set(SystemSet::on_update(AppState::Playing).with_system(ghost_collision))
        .add_system_set(SystemSet::on_update(AppState::GameOver).with_system(game_over))
        .run();
}

fn player_logic(
    mut query: Query<(
        &mut Transform,
        &mut TextureAtlasSprite,
        &mut AnimationTimer,
        &mut Player,
    )>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let (mut transform, mut sprite, mut anim_timer, mut player) = query.single_mut();

    let mut velocity = Vec3::ZERO;

    if keys.pressed(KeyCode::D) {
        velocity += Vec3::new(PLAYER_SPEED, 0., 0.);
    } else if keys.pressed(KeyCode::A) {
        velocity -= Vec3::new(PLAYER_SPEED, 0., 0.);
    } else if keys.pressed(KeyCode::W) {
        velocity += Vec3::new(0., PLAYER_SPEED, 0.);
    } else if keys.pressed(KeyCode::S) {
        velocity -= Vec3::new(0., PLAYER_SPEED, 0.);
    }

    transform.translation += velocity * time.delta_seconds();

    if transform.translation.x > WIDTH / 2. {
        transform.translation.x -= WIDTH;
    } else if transform.translation.x < -WIDTH / 2. {
        transform.translation.x += WIDTH;
    }

    if transform.translation.y > HEIGHT / 2. {
        transform.translation.y -= HEIGHT;
    } else if transform.translation.y < -HEIGHT / 2. {
        transform.translation.y += HEIGHT;
    }

    if velocity != Vec3::ZERO {
        anim_timer.0.tick(time.delta());
        if velocity.y > 0. {
            if player.direction != Direction::UP {
                sprite.index = 6;
            }
            player.direction = Direction::UP;

            if anim_timer.0.just_finished() {
                if sprite.index == 7 {
                    sprite.index = 8;
                } else {
                    sprite.index = 7;
                }
            }
        } else if velocity.x < 0. {
            if player.direction != Direction::RIGHT {
                sprite.index = 3;
            }
            player.direction = Direction::RIGHT;

            if anim_timer.0.just_finished() {
                if sprite.index == 4 {
                    sprite.index = 5;
                } else {
                    sprite.index = 4;
                }
            }
        } else if velocity.x > 0. {
            if player.direction != Direction::LEFT {
                sprite.index = 0;
            }
            player.direction = Direction::LEFT;

            if anim_timer.0.just_finished() {
                if sprite.index == 1 {
                    sprite.index = 2;
                } else {
                    sprite.index = 1;
                }
            }
        } else {
            if player.direction != Direction::DOWN {
                sprite.index = 9;
            }
            player.direction = Direction::DOWN;

            if anim_timer.0.just_finished() {
                if sprite.index == 10 {
                    sprite.index = 11;
                } else {
                    sprite.index = 10;
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        texture: asset_server.load("bg.png"),
        transform: Transform::from_scale(Vec3::splat(SCALE)),
        ..default()
    });

    let pickup_audio_handle: Handle<AudioSource> = asset_server.load("pickup.wav");
    commands.insert_resource(PickupAudio(pickup_audio_handle));

    let game_over_audio_handle: Handle<AudioSource> = asset_server.load("gameover.wav");
    commands.insert_resource(GameOverAudio(game_over_audio_handle));

    let pumpkin_handle = asset_server.load("pumpkin.png");
    commands.insert_resource(PumpkinSprite(pumpkin_handle.clone()));

    commands.insert_resource(PumpkinSpawnTimer(Timer::from_seconds(
        PUMPKIN_TIMER,
        TimerMode::Repeating,
    )));

    let ghost_handle = asset_server.load("ghost.png");
    commands.insert_resource(GhostSprite(ghost_handle.clone()));

    commands.insert_resource(GhostSpawnTimer(Timer::from_seconds(
        GHOST_TIMER,
        TimerMode::Repeating,
    )));

    let player_sprite_sheet = asset_server.load("player.png");
    let player_atlas =
        TextureAtlas::from_grid(player_sprite_sheet, Vec2::splat(8.), 12, 1, None, None);
    let player_atlas_handle = texture_atlases.add(player_atlas);
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 0,
                ..default()
            },
            texture_atlas: player_atlas_handle,
            transform: Transform::IDENTITY.with_translation(Vec3::new(0., 0., 1.)),
            ..default()
        },
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        Player {
            direction: Direction::DOWN,
        },
    ));

    let font_handle: Handle<Font> = asset_server.load("Pixel_NES.otf");
    commands.insert_resource(PixelFont(font_handle.clone()));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Score:",
                TextStyle {
                    font: font_handle.clone(),
                    font_size: 10.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font_handle.clone(),
                font_size: 10.0,
                color: Color::WHITE,
            }),
        ])
        .with_text_alignment(TextAlignment::TOP_LEFT)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                left: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        ScoreText,
    ));

    commands.insert_resource(Score(0));
}

fn game_over(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut state: ResMut<State<AppState>>,
    query: Query<Entity>,
) {
    if keys.pressed(KeyCode::Space) {
        state.set(AppState::Playing).unwrap();
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_score(mut query: Query<&mut Text, With<ScoreText>>, score: Res<Score>) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{}", score.0);
}

fn spawn_pumpkins(
    mut commands: Commands,
    pumpkin_handle: Res<PumpkinSprite>,
    mut timer: ResMut<PumpkinSpawnTimer>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();
    if timer.0.tick(time.delta()).just_finished() {
        commands.spawn((
            SpriteBundle {
                texture: pumpkin_handle.0.clone(),
                transform: Transform::from_scale(Vec3::splat(SCALE)).with_translation(Vec3::new(
                    rng.gen_range(-SCALED_WIDTH / 2f32..SCALED_WIDTH / 2f32),
                    rng.gen_range(-SCALED_HEIGHT / 2f32..SCALED_HEIGHT / 2f32),
                    1.,
                )),
                ..default()
            },
            Pumpkin,
        ));
    }
}

fn spawn_ghost(
    mut commands: Commands,
    ghost_handle: Res<GhostSprite>,
    time: Res<Time>,
    mut timer: ResMut<GhostSpawnTimer>,
) {
    let mut rng = rand::thread_rng();
    if timer.0.tick(time.delta()).just_finished() {
        commands.spawn((
            SpriteBundle {
                texture: ghost_handle.0.clone(),
                transform: Transform::from_scale(Vec3::splat(SCALE)).with_translation(Vec3::new(
                    rng.gen_range(-SCALED_WIDTH / 2f32..SCALED_WIDTH / 2f32),
                    rng.gen_range(-SCALED_HEIGHT / 2f32..SCALED_HEIGHT / 2f32),
                    1.,
                )),
                ..default()
            },
            Chase,
            Ghost,
        ));
    }
}

fn chase_ai(
    mut query: Query<&mut Transform, With<Chase>>,
    player_query: Query<&Transform, (With<Player>, Without<Chase>)>,
    time: Res<Time>,
) {
    let player = player_query.single();
    for mut transform in query.iter_mut() {
        let dir = (player.translation - transform.translation).normalize()
            * GHOST_SPEED
            * time.delta_seconds();
        transform.translation += dir;
    }
}

fn ghost_collision(
    ghost_query: Query<&Transform, With<Ghost>>,
    player_query: Query<&Transform, With<Player>>,
    font_handle: Res<PixelFont>,
    mut state: ResMut<State<AppState>>,
    mut commands: Commands,
    game_over_audio_handle: Res<GameOverAudio>,
    audio: Res<Audio>,
) {
    let player = player_query.single();
    let prectx = player.translation.x - 4.;
    let precty = player.translation.y - 4.;

    for transform in ghost_query.iter() {
        let rectx = transform.translation.x - 4.;
        let recty = transform.translation.y - 4.;

        if rectx < prectx + 8. && rectx + 8. > prectx && recty < precty + 8. && recty + 8. > precty
        {
            commands.spawn(
                TextBundle::from_sections([TextSection::new(
                    "Game Over! Press Space to try again.",
                    TextStyle {
                        font: font_handle.0.clone(),
                        font_size: 10.0,
                        color: Color::WHITE,
                    },
                )])
                .with_text_alignment(TextAlignment::CENTER)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        bottom: Val::Px(HEIGHT / 2.),
                        left: Val::Px(WIDTH / 16.),
                        ..default()
                    },
                    ..default()
                }),
            );
            state.set(AppState::GameOver).unwrap();
            audio.play(game_over_audio_handle.0.clone());
        }
    }
}

fn pumpkin_collision(
    pumpkin_query: Query<(Entity, &Transform), With<Pumpkin>>,
    player_query: Query<&Transform, With<Player>>,
    mut score: ResMut<Score>,
    mut commands: Commands,
    pickup_audio: Res<PickupAudio>,
    audio: Res<Audio>,
) {
    let player = player_query.single();
    let prectx = player.translation.x - 4.;
    let precty = player.translation.y - 4.;

    for (entity, transform) in pumpkin_query.iter() {
        let rectx = transform.translation.x - 4.;
        let recty = transform.translation.y - 4.;

        if rectx < prectx + 8. && rectx + 8. > prectx && recty < precty + 8. && recty + 8. > precty
        {
            score.0 += 1;
            commands.entity(entity).despawn();
            audio.play(pickup_audio.0.clone());
        }
    }
}
