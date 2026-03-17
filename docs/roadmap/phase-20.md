# Phase 20: 咲夜プレイアブルキャラクター実装

## 目標

十六夜 咲夜を第3のプレイアブルキャラクターとして実装する。
咲夜はStage 5のボスとしても登場するため、プレイアブル版の性能はボス版と独立して設計する。

## 完了条件

- [ ] 咲夜 ショットタイプA（銀時計）実装
- [ ] 咲夜 ショットタイプB（幻時計）実装
- [ ] 咲夜ボム：タイプA「パーフェクトスクウェア」（全弾停止→ナイフ変換→一斉再開）
- [ ] 咲夜ボム：タイプB「タイムエクスプロージョン」（画面全体に時計の針）
- [ ] `bomb_sakuya.wgsl` シェーダー実装（時間停止の時空歪み演出）
- [ ] 咲夜の立ち絵（4表情差分）
- [ ] 咲夜のピクセルアートスプライト（32×48 px）
- [ ] キャラクター選択画面に咲夜を追加
- [ ] 咲夜固有のダイアログテキスト（日本語・英語）
- [ ] 咲夜を使ったユニットテストが `just test` で通る

---

## 実装詳細

### ショットタイプA：銀時計

```rust
// 通常ショット：前方集中ナイフ（高速・高頻度）
// フォーカスショット：扇状ナイフ投擲（5-way、広角）
// ボム：全弾停止 → ナイフ変換 → 3秒後に一斉再開（高威力）

pub fn sakuya_shot_type_a_params() -> ShotParams {
    ShotParams {
        normal_pattern: BulletPattern::Focused {
            count: 3,
            spread_deg: 5.0,
            speed: 600.0,
            kind: PlayerBulletKind::Knife,
        },
        focus_pattern: BulletPattern::NWaySpread {
            count: 5,
            spread_deg: 45.0,
            speed: 500.0,
            kind: PlayerBulletKind::Knife,
        },
        fire_rate: 0.04,  // 25発/秒
        damage_per_bullet: 18.0,
    }
}
```

### ショットタイプB：幻時計

```rust
// 通常ショット：分裂弾（前方に飛んで2分裂）
// フォーカスショット：前方集中の直線ナイフ（単発・高ダメージ）

pub fn sakuya_shot_type_b_params() -> ShotParams {
    ShotParams {
        normal_pattern: BulletPattern::Splitting {
            initial_speed: 400.0,
            split_after_secs: 0.15,
            split_count: 2,
            kind: PlayerBulletKind::Knife,
        },
        focus_pattern: BulletPattern::Focused {
            count: 1,
            spread_deg: 0.0,
            speed: 700.0,
            kind: PlayerBulletKind::LargeKnife,
        },
        fire_rate: 0.06,
        damage_per_bullet: 25.0,
    }
}
```

### bomb_sakuya.wgsl 概要

```wgsl
// 時間停止演出：
// 1. 画面全体に時計の針パターンをオーバーレイ
// 2. 停止している弾を青白いグレーに変色
// 3. 画面端から中央へ向かう時空歪み（UV歪み効果）
// 4. 3秒後：一斉再開のフラッシュ

@group(2) @binding(0) var<uniform> time: f32;
@group(2) @binding(1) var<uniform> phase: f32; // 0.0 = 停止中, 1.0 = 再開瞬間
```

---

## 参照

- `docs/specification.md` § 5.4 咲夜（プレイアブル）
- `docs/10_shaders_wgsl.md` § bomb_sakuya
- `app/core/src/systems/boss/bosses/sakuya.rs` — ボス版咲夜とは独立した実装
