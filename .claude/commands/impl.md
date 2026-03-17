---
description: Start implementing a GitHub issue
allowed-tools: ["bash", "read", "write", "edit", "glob", "grep", "task"]
argument-hint: "<issue-number>"
---

First, fetch the issue details:

```bash
gh issue view $1
```

Now proceed with implementing this issue.

**Development Guidelines:**
- All comments and documentation must be written in English
- Follow Rust best practices and idiomatic patterns
- Keep game logic modular and maintainable
- Consider performance implications (60fps target, 500+ bullets on screen)
- Use Bevy's ECS patterns (Entity-Component-System)

**Before starting:**
1. Review the issue requirements carefully
2. Check acceptance criteria (if PBI/feature)
3. Identify affected components:
   - Danmaku Systems (bullet emitter, patterns, BulletGlowMaterial)
   - Player (movement, shooting, hitbox, graze)
   - Boss / Spell Card (phase management, BossPhaseData)
   - Collision Detection (manual circle-based, SpatialGrid, graze)
   - Score System (graze bonus, spell card bonus, POI)
   - Bomb System (BombState, counter-bomb, BombReimuMaterial/BombMarisaMaterial)
   - Item System (ItemKind, ItemPhysics, FragmentTracker)
   - UI & HUD (scarlet-ui)
   - Audio (BGM/SFX via bevy_kira_audio)
   - WGSL Shaders (Material2d + AsBindGroup)
   - Game State Management (AppState)
4. Review related documentation (docs/01-10)
5. Plan the implementation approach

**Crate placement:**
- `scarlet-core` (`app/core/`) — all ECS game logic
- `scarlet-ui` (`app/ui/`) — camera, UI screens, HUD
- `scarlet-audio` (`app/audio/`) — BGM/SFX
- `scarlet-assets` (`app/assets/`) — asset loading
- `touhou-project-scarlet` (`app/touhou-project-scarlet/`) — binary entry point

**Implementation checklist:**
- [ ] Follow Rust naming conventions and coding style
- [ ] Place code in appropriate crate (scarlet-core/ui/audio/assets)
- [ ] Add unit tests in `#[cfg(test)]` sections where appropriate
- [ ] Document public APIs with rustdoc comments
- [ ] Implement proper error handling with Result types
- [ ] Use Bevy systems and queries efficiently
- [ ] For shaders: implement `Material2d` + WGSL file in `assets/shaders/`
- [ ] Run `just fmt` before committing
- [ ] Run `just clippy` to check warnings
- [ ] Run `just test` to verify all tests pass
- [ ] Run `just build` to ensure it compiles
- [ ] Test in-game (run with `just dev`)

**Bevy Best Practices:**
- Use `Query` filters (`With`, `Without`, `Changed`) to optimize systems
- Prefer event-driven communication over direct system dependencies
- Use `Commands` for deferred entity operations
- Use `ResMut` sparingly, prefer `Res` when possible
- Add systems to appropriate `SystemSet` for execution order:
  `Input → PlayerLogic → BulletEmit → Movement → Collision → GameLogic → StageControl → Effects → Cleanup`
- Use `SpatialGrid` for collision optimization when many bullets/enemies exist
- Use `DespawnOnExit(AppState::Playing)` for in-game entities

**Shader Guidelines (WGSL):**
- Bullets use `Mesh2d` + `MeshMaterial2d<BulletGlowMaterial>` (not `Sprite`)
- Camera must be in HDR mode (`Camera { hdr: true, ..default() }`)
- Use `LinearRgba::new(r, g, b, 1.0)` with values > 1.0 for glow/bloom
- Normal sprites use values ≤ 1.0 (no bloom)
- Hot reload works via `just dev` (file_watcher feature enabled)
- Reference: `docs/10_shaders_wgsl.md`

**Commit Scopes (for later):**
Use these scopes for conventional commits:
- `core`: Core game logic (app/core)
- `ui`: UI systems (app/ui)
- `audio`: Audio systems (app/audio)
- `assets`: Asset management (app/assets)
- `danmaku`: Bullet/emitter/pattern systems
- `player`: Player movement, shooting, hitbox
- `boss`: Boss phase, spell card, movement
- `collision`: Collision detection & graze
- `score`: Scoring system
- `shader`: WGSL shaders & Material2d
- `effects`: Visual effects & particles
- `docs`: Documentation updates
- `chore`: Build, dependencies, tooling

Please proceed with the implementation.
