use bevy::prelude::*;

use crate::components::effects::FadeOut;

// ---------------------------------------------------------------------------
// fade_out_system
// ---------------------------------------------------------------------------

/// Ticks every [`FadeOut`] timer and linearly decreases the [`Sprite`] alpha.
///
/// Each frame the system computes `alpha = 1.0 - timer.fraction()` and writes
/// it to [`Sprite::color`]. When the timer finishes the entity is despawned.
///
/// # Requirements
///
/// The entity must have both a [`FadeOut`] and a [`Sprite`] component.
/// Entities with [`FadeOut`] but no [`Sprite`] are silently ignored.
///
/// # Ordering
///
/// Registered in [`crate::GameSystemSet::Effects`] so it runs after all
/// gameplay and collision logic but before cleanup, ensuring the alpha is
/// up-to-date when rendered.
pub fn fade_out_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut FadeOut)>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut fade) in &mut query {
        fade.timer.tick(time.delta());

        let alpha = 1.0 - fade.timer.fraction();
        sprite.color = sprite.color.with_alpha(alpha);

        if fade.timer.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
