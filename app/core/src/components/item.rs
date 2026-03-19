use bevy::prelude::*;

/// The kind of collectible item an entity represents.
///
/// Each variant has distinct gameplay effects applied on pickup by
/// [`crate::systems::item::item_collection_system`].
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    /// +1 to the player's power level (max 128).
    PowerSmall,
    /// +8 to the player's power level (max 128).
    PowerLarge,
    /// Score bonus whose value depends on the player's Y position at the moment
    /// of collection; see [`crate::systems::item::calc_point_item_value`].
    PointItem,
    /// One of five fragments needed to earn an extra life.
    LifeFragment,
    /// One of five fragments needed to earn an extra bomb stock.
    BombFragment,
    /// Sets the player's power level to the maximum (128) instantly.
    FullPower,
}

impl ItemKind {
    /// Placeholder sprite colour used until real sprite sheets are added in Phase 19.
    pub fn color(self) -> Color {
        match self {
            Self::PowerSmall | Self::PowerLarge | Self::FullPower => Color::srgb(1.0, 0.2, 0.2),
            Self::PointItem => Color::srgb(0.2, 0.5, 1.0),
            Self::LifeFragment => Color::srgb(0.2, 1.0, 0.4),
            Self::BombFragment => Color::srgb(1.0, 0.8, 0.2),
        }
    }

    /// Sprite size derived from the item kind (px).
    ///
    /// Larger items (`PowerLarge`, `FullPower`) are 12×12; all others are 8×8.
    pub fn sprite_size(self) -> Vec2 {
        match self {
            Self::PowerLarge | Self::FullPower => Vec2::splat(12.0),
            _ => Vec2::splat(8.0),
        }
    }
}

/// Physics state for a collectible item entity.
///
/// Every item falls downward under gravity and can be attracted toward the
/// player by the item-movement system ([`crate::systems::item::item_movement_system`]).
/// Once `attracted` is set to `true` it stays true for the item's lifetime.
#[derive(Component)]
pub struct ItemPhysics {
    /// Current velocity in pixels per second.
    pub velocity: Vec2,
    /// When `true` the item moves toward the player at high speed.
    ///
    /// Set automatically when the player is above the score line, or when the
    /// item enters the player's pickup attraction radius.
    pub attracted: bool,
    /// Downward acceleration in pixels per second squared (gravity equivalent).
    pub fall_speed: f32,
}

impl Default for ItemPhysics {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            attracted: false,
            fall_speed: 80.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_item_kinds_have_positive_sprite_size() {
        let kinds = [
            ItemKind::PowerSmall,
            ItemKind::PowerLarge,
            ItemKind::PointItem,
            ItemKind::LifeFragment,
            ItemKind::BombFragment,
            ItemKind::FullPower,
        ];
        for kind in kinds {
            let size = kind.sprite_size();
            assert!(
                size.x > 0.0 && size.y > 0.0,
                "{kind:?} must have positive size"
            );
        }
    }

    #[test]
    fn power_large_and_full_power_are_larger_than_small_items() {
        assert!(ItemKind::PowerLarge.sprite_size().x > ItemKind::PowerSmall.sprite_size().x);
        assert!(ItemKind::FullPower.sprite_size().x > ItemKind::PointItem.sprite_size().x);
    }

    #[test]
    fn item_physics_default_not_attracted_and_zero_velocity() {
        let p = ItemPhysics::default();
        assert!(!p.attracted);
        assert_eq!(p.velocity, Vec2::ZERO);
        assert!(p.fall_speed > 0.0);
    }
}
