use bevy::prelude::*;

use crate::{
    AppState,
    components::{
        bullet::{BulletEmitter, BulletPattern, EnemyBullet, EnemyBulletKind},
        enemy::Enemy,
    },
    constants::{PLAY_AREA_HALF_H, PLAY_AREA_HALF_W, PLAY_AREA_HEIGHT, PLAY_AREA_WIDTH},
    systems::danmaku::emitter::SpiralState,
};

/// Immediately transitions to [`AppState::Playing`] on startup.
///
/// Bypasses both the future title/character-select screens *and* the
/// [`crate::systems::loading::wait_for_configs`] readiness check, so the
/// game enters gameplay instantly using `DEFAULT_*` fallback values while
/// configs finish loading in the background. Only active with `debug-hitbox`.
pub fn debug_skip_to_playing(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Playing);
}

/// Draws the play-area boundary and center cross using Bevy Gizmos.
///
/// Enabled only when the `debug-hitbox` feature is active:
/// ```bash
/// cargo run -p touhou-project-scarlet --features scarlet-core/debug-hitbox
/// ```
pub fn debug_play_area_system(mut gizmos: Gizmos) {
    let color = Color::srgba(1.0, 1.0, 0.0, 0.8);

    // Outer boundary rectangle.
    gizmos.rect_2d(
        Isometry2d::IDENTITY,
        Vec2::new(PLAY_AREA_WIDTH, PLAY_AREA_HEIGHT),
        color,
    );

    // Center cross for orientation reference.
    gizmos.line_2d(
        Vec2::new(-PLAY_AREA_HALF_W, 0.0),
        Vec2::new(PLAY_AREA_HALF_W, 0.0),
        Color::srgba(1.0, 1.0, 0.0, 0.3),
    );
    gizmos.line_2d(
        Vec2::new(0.0, -PLAY_AREA_HALF_H),
        Vec2::new(0.0, PLAY_AREA_HALF_H),
        Color::srgba(1.0, 1.0, 0.0, 0.3),
    );
}

/// Spawns a set of dummy enemy emitters for in-development pattern testing.
///
/// Spawns four emitters at fixed positions, one per [`BulletPattern`] variant,
/// so all patterns can be visually verified with `just dev`.
pub fn spawn_debug_enemies(mut commands: Commands) {
    // Ring — top-left
    commands.spawn((
        Enemy::new(100.0, 16.0),
        BulletEmitter {
            pattern: BulletPattern::Ring {
                count: 8,
                speed: 120.0,
            },
            bullet_kind: EnemyBulletKind::SmallRound,
            timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            active: true,
        },
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(-80.0, 120.0, 1.0),
    ));

    // Aimed — top-right
    commands.spawn((
        Enemy::new(100.0, 16.0),
        BulletEmitter {
            pattern: BulletPattern::Aimed {
                count: 3,
                spread_deg: 20.0,
                speed: 150.0,
            },
            bullet_kind: EnemyBulletKind::MediumRound,
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            active: true,
        },
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(80.0, 120.0, 1.0),
    ));

    // Spread — centre-top
    commands.spawn((
        Enemy::new(100.0, 16.0),
        BulletEmitter {
            pattern: BulletPattern::Spread {
                count: 5,
                spread_deg: 60.0,
                speed: 130.0,
                angle_offset: 0.0,
            },
            bullet_kind: EnemyBulletKind::Rice,
            timer: Timer::from_seconds(1.2, TimerMode::Repeating),
            active: true,
        },
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 160.0, 1.0),
    ));

    // Spiral — centre
    commands.spawn((
        Enemy::new(200.0, 16.0),
        BulletEmitter {
            pattern: BulletPattern::Spiral {
                arms: 3,
                speed: 100.0,
                rotation_speed_deg: 120.0,
            },
            bullet_kind: EnemyBulletKind::Star,
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
            active: true,
        },
        SpiralState::default(),
        Sprite {
            color: Color::srgb(0.6, 0.3, 0.6),
            custom_size: Some(Vec2::splat(24.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 60.0, 1.0),
    ));
}

/// Draws collision circles for all active enemy bullets using Gizmos.
///
/// Only active with `debug-hitbox` feature.
pub fn debug_bullet_hitbox(
    mut gizmos: Gizmos,
    bullets: Query<(&Transform, &EnemyBulletKind), With<EnemyBullet>>,
) {
    for (transform, kind) in &bullets {
        gizmos.circle_2d(
            Isometry2d::from_translation(transform.translation.truncate()),
            kind.collision_radius(),
            Color::srgba(0.0, 1.0, 0.0, 0.4),
        );
    }
}
