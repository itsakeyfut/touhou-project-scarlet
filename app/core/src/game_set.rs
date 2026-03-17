use bevy::prelude::*;

/// Ordered system sets for in-game logic.
///
/// All sets run only while [`crate::AppState::Playing`] is active and execute
/// in strict sequential order via `.chain()`.
///
/// ```text
/// Input → PlayerLogic → BulletEmit → Movement → Collision
///       → GameLogic → StageControl → Effects → Cleanup
/// ```
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    /// Raw keyboard / gamepad input reading.
    Input,
    /// Player-specific logic (shooting timer, bomb state).
    PlayerLogic,
    /// Enemy bullet emission and pattern updates.
    BulletEmit,
    /// Velocity integration — moves all entities.
    Movement,
    /// Circle-based collision detection (hit, graze, item pickup).
    Collision,
    /// Score, lives, extend checks, stage-clear conditions.
    GameLogic,
    /// Stage script timeline and boss-phase transitions.
    StageControl,
    /// Visual effects, particles, screen shake.
    Effects,
    /// Despawn out-of-bounds entities and expired timers.
    Cleanup,
}
