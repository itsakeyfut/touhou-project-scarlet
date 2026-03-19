pub mod bullet;
pub mod enemy;
pub mod item;
pub mod player;

pub use bullet::{
    BulletEmitter, BulletPattern, BulletTrail, BulletVelocity, DespawnOutOfBounds, EnemyBullet,
    EnemyBulletKind, PlayerBullet, ShootTimer,
};
pub use enemy::{Enemy, EnemyKind, EnemyMovement};
pub use item::{ItemKind, ItemPhysics};
pub use player::{GrazeVisual, InvincibilityTimer, Player, PlayerStats};

use bevy::prelude::*;

/// Marker component attached to every entity that lives only during an active
/// game session (i.e. while [`crate::states::AppState::Playing`] is active).
///
/// Used by [`crate::systems::cleanup::cleanup_game_session`], registered on
/// [`bevy::app::OnExit`]`(`[`crate::states::AppState::Playing`]`)`, to despawn
/// the entire gameplay scene in one pass — player, enemies, bullets, and items —
/// so that re-entering `Playing` starts from a clean world state.
///
/// # Usage
///
/// Add this component to every entity spawned during gameplay:
///
/// ```rust,ignore
/// commands.spawn((
///     MyComponent,
///     GameSessionEntity,
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GameSessionEntity;
