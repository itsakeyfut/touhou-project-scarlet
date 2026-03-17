# Phase 05: 衝突・グレイズ判定 + グレイズシェーダー

## 目標

当たり判定が機能し、プレイヤーが弾に当たると残機が減る。グレイズすると +500点。グレイズ圏を `GrazeMaterial` シェーダーで可視化する。

## 完了条件

- [ ] プレイヤーヒットボックス (r=2.5px) で被弾判定
- [ ] グレイズ圏 (r=16px) でグレイズ判定
- [ ] 被弾時に残機減少 → 無敵時間 → ゲームオーバー
- [ ] グレイズ時にスコア +500・カウント +1
- [ ] プレイヤー弾が敵に当たる基本判定
- [ ] `GrazeMaterial` (`shaders/graze_field.wgsl`) の実装
- [ ] 低速移動中 (Shift) にグレイズフィールドが可視化
- [ ] グレイズ発生時にフィールドがスパーク
- [ ] ユニットテスト: `check_circle_collision`

---

## タスク詳細

### 1. コリジョンユーティリティ

```rust
// app/core/src/systems/collision.rs

/// 2つの円の重なり判定
#[inline]
pub fn check_circle_collision(pos_a: Vec2, r_a: f32, pos_b: Vec2, r_b: f32) -> bool {
    let dist_sq = (pos_a - pos_b).length_squared();
    let sum_r = r_a + r_b;
    dist_sq < sum_r * sum_r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collision_overlap() {
        assert!(check_circle_collision(Vec2::ZERO, 5.0, Vec2::new(3.0, 0.0), 5.0));
    }

    #[test]
    fn test_collision_no_overlap() {
        assert!(!check_circle_collision(Vec2::ZERO, 5.0, Vec2::new(11.0, 0.0), 5.0));
    }

    #[test]
    fn test_graze_zone() {
        let player = Vec2::ZERO;
        let bullet_r = 4.0;

        // グレイズ圏内 (dist=18, r_total=20) → 当たり
        assert!(check_circle_collision(player, 16.0, Vec2::new(18.0, 0.0), bullet_r));
        // ヒットボックス外 (dist=18, r_total=6.5) → 当たらない
        assert!(!check_circle_collision(player, 2.5, Vec2::new(18.0, 0.0), bullet_r));
    }
}
```

### 2. PlayerHitEvent とゲームオーバー

```rust
// app/core/src/events.rs
#[derive(Event)]
pub struct PlayerHitEvent;
```

```rust
// app/core/src/systems/collision.rs
pub fn player_hit_detection(
    mut commands: Commands,
    player: Query<(Entity, &Transform, &PlayerStats, Option<&InvincibilityTimer>), With<Player>>,
    bullets: Query<(&Transform, &EnemyBulletKind), With<EnemyBullet>>,
    mut hit_events: EventWriter<PlayerHitEvent>,
) {
    let Ok((player_entity, player_tf, stats, invincible)) = player.single() else { return };

    // 無敵中はスキップ
    if invincible.is_some() { return; }

    let player_pos = player_tf.translation.truncate();

    for (bullet_tf, kind) in &bullets {
        let bullet_pos = bullet_tf.translation.truncate();
        if check_circle_collision(player_pos, stats.hitbox_radius, bullet_pos, kind.collision_radius()) {
            hit_events.write(PlayerHitEvent);
            return; // 1フレームに1ヒットのみ
        }
    }
}
```

### 3. 被弾処理

```rust
// app/core/src/systems/player.rs
const INVINCIBLE_DURATION: f32 = 3.0;
const COUNTER_BOMB_WINDOW: f32 = 0.1;

pub fn handle_player_hit(
    mut commands: Commands,
    mut hit_events: EventReader<PlayerHitEvent>,
    mut game_data: ResMut<GameData>,
    mut bomb_state: ResMut<BombState>,
    player: Query<Entity, With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _ in hit_events.read() {
        // カウンターボムウィンドウを開く
        bomb_state.counter_bomb_window = COUNTER_BOMB_WINDOW;

        if game_data.lives == 0 {
            next_state.set(AppState::GameOver);
            return;
        }

        game_data.lives -= 1;
        game_data.power = game_data.power.saturating_sub(16); // パワーダウン
        game_data.bombs = game_data.bombs.max(3); // ボム補充（原作仕様）

        // 無敵時間付与
        if let Ok(entity) = player.single() {
            commands.entity(entity).insert(InvincibilityTimer {
                timer: Timer::from_seconds(INVINCIBLE_DURATION, TimerMode::Once),
            });
        }
    }
}

pub fn update_invincibility(
    mut commands: Commands,
    mut players: Query<(Entity, &mut Sprite, &mut InvincibilityTimer), With<Player>>,
    time: Res<Time>,
) {
    for (entity, mut sprite, mut inv) in &mut players {
        inv.timer.tick(time.delta());

        // 点滅エフェクト
        let blink = (time.elapsed_secs() * 10.0).sin() > 0.0;
        sprite.color.set_alpha(if blink { 0.3 } else { 1.0 });

        if inv.timer.just_finished() {
            sprite.color.set_alpha(1.0);
            commands.entity(entity).remove::<InvincibilityTimer>();
        }
    }
}
```

### 4. グレイズシステム

```rust
// app/core/src/systems/graze.rs
use std::collections::HashSet;

const GRAZE_SCORE: u64 = 500;

pub fn graze_detection_system(
    player: Query<(&Transform, &PlayerStats), With<Player>>,
    bullets: Query<(Entity, &Transform, &EnemyBulletKind), With<EnemyBullet>>,
    mut grazed: Local<HashSet<Entity>>,
    mut graze_events: EventWriter<GrazeEvent>,
    mut game_data: ResMut<GameData>,
) {
    let Ok((player_tf, stats)) = player.single() else { return };
    let player_pos = player_tf.translation.truncate();

    let current_bullets: HashSet<Entity> = bullets.iter().map(|(e, _, _)| e).collect();
    grazed.retain(|e| current_bullets.contains(e));

    for (entity, bullet_tf, kind) in &bullets {
        if grazed.contains(&entity) { continue; }

        let bullet_pos = bullet_tf.translation.truncate();
        let r = kind.collision_radius();

        let in_graze = check_circle_collision(player_pos, stats.graze_radius, bullet_pos, r);
        let not_hit  = !check_circle_collision(player_pos, stats.hitbox_radius, bullet_pos, r);

        if in_graze && not_hit {
            grazed.insert(entity);
            game_data.graze_count += 1;
            game_data.score += GRAZE_SCORE;
            graze_events.write(GrazeEvent { bullet_entity: entity });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graze_logic() {
        // グレイズ圏 r=16、弾r=4、ヒットボックス r=2.5
        let player = Vec2::ZERO;
        let bullet_r = 4.0;

        // dist=18: graze圏内(20)、ヒット圏外(6.5) → グレイズ
        let dist = 18.0f32;
        let bullet = Vec2::new(dist, 0.0);
        assert!(check_circle_collision(player, 16.0, bullet, bullet_r));   // graze
        assert!(!check_circle_collision(player, 2.5, bullet, bullet_r));   // not hit
    }
}
```

### 5. プレイヤー弾 vs 敵の衝突

```rust
// app/core/src/systems/collision.rs
pub fn player_bullet_hit_enemy(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<PlayerBullet>>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
    mut defeated_events: EventWriter<EnemyDefeatedEvent>,
) {
    const BULLET_RADIUS: f32 = 4.0;
    const ENEMY_RADIUS: f32 = 12.0; // Phase 7 でコンポーネント化

    for (bullet_entity, bullet_tf) in &bullets {
        let bpos = bullet_tf.translation.truncate();
        for (enemy_entity, enemy_tf, mut enemy) in &mut enemies {
            let epos = enemy_tf.translation.truncate();
            if check_circle_collision(bpos, BULLET_RADIUS, epos, ENEMY_RADIUS) {
                commands.entity(bullet_entity).despawn();
                enemy.hp -= 12.0; // PlayerBullet.damage は後でクエリ
                if enemy.hp <= 0.0 {
                    defeated_events.write(EnemyDefeatedEvent {
                        enemy_entity,
                        score: enemy.score_value,
                    });
                    commands.entity(enemy_entity).despawn_recursive();
                }
                break;
            }
        }
    }
}
```

### 6. SystemSet への配置

```
Collision セット:
  - player_hit_detection
  - graze_detection_system
  - player_bullet_hit_enemy

GameLogic セット:
  - handle_player_hit
  - update_invincibility
```

### 7. デバッグ: ヒットボックス可視化

```rust
#[cfg(debug_assertions)]
pub fn debug_hitbox_system(
    mut gizmos: Gizmos,
    player: Query<(&Transform, &PlayerStats), With<Player>>,
) {
    if let Ok((tf, stats)) = player.single() {
        let pos = tf.translation.truncate();
        gizmos.circle_2d(pos, stats.hitbox_radius, Color::srgb(1.0, 0.0, 0.0));
        gizmos.circle_2d(pos, stats.graze_radius,  Color::srgb(1.0, 1.0, 0.0));
    }
}
```

### シェーダー: グレイズフィールド追加

```rust
// setup_graze_visual をプレイヤースポーン時に呼び出す
// (docs/10_shaders_wgsl.md § 5 参照)

// ScarletShadersPlugin に追加
app.add_plugins(Material2dPlugin::<GrazeMaterial>::default())
   .add_systems(Update, (
       setup_graze_visual.run_if(in_state(AppState::Playing)),
       update_graze_material.run_if(in_state(AppState::Playing)),
   ));
```

---

## 参照

- `docs/03_danmaku_systems.md` § 3 (グレイズシステム)
- `docs/02_architecture.md` § コリジョン検出
- `docs/10_shaders_wgsl.md` § 5 (グレイズ電気フィールド)
