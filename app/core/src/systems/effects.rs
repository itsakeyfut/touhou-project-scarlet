use bevy::prelude::*;

use crate::components::effects::FadeOut;

// ---------------------------------------------------------------------------
// fade_out_system
// ---------------------------------------------------------------------------

/// Ticks every [`FadeOut`] timer, decreases alpha where applicable, and
/// despawns the entity when the timer finishes.
///
/// Two entity classes are handled:
///
/// - **Sprite entities** (`With<Sprite>`): the system writes
///   `alpha = 1.0 - timer.fraction()` to [`Sprite::color`] each frame,
///   producing a smooth visual fade-out before despawn.
/// - **Mesh2d entities** (`Without<Sprite>`): no visual alpha change is
///   applied (modifying a `Material2d` uniform per-entity would require
///   material-specific handling). The timer still ticks and the entity is
///   despawned when it finishes, giving a brief delay before removal rather
///   than an instant disappear.
///
/// # Ordering
///
/// Registered in [`crate::GameSystemSet::Effects`] so it runs after all
/// gameplay and collision logic but before cleanup, ensuring the alpha is
/// up-to-date when rendered.
pub fn fade_out_system(
    mut commands: Commands,
    mut sprite_query: Query<(Entity, &mut Sprite, &mut FadeOut)>,
    mut mesh_query: Query<(Entity, &mut FadeOut), Without<Sprite>>,
    time: Res<Time>,
) {
    // Sprite path: smooth alpha fade + despawn.
    for (entity, mut sprite, mut fade) in &mut sprite_query {
        fade.timer.tick(time.delta());
        sprite.color = sprite.color.with_alpha(1.0 - fade.timer.fraction());
        if fade.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }

    // Mesh2d path: tick timer + despawn (no per-material alpha change).
    for (entity, mut fade) in &mut mesh_query {
        fade.timer.tick(time.delta());
        if fade.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
