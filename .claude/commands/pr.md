---
description: Commit changes and create PR (keep under 100 lines)
allowed-tools: ["bash", "read", "grep"]
argument-hint: "[file1] [file2] ..."
---

Complete the implementation workflow:

**Steps:**

1. **Check current status:**
   ```bash
   git status
   git diff --stat
   ```

2. **Verify changes are under 100 lines:**
   ```bash
   git diff | wc -l
   ```
   Check the diff size. If over 100 lines, consider splitting into multiple PRs.

3. **MANDATORY: Verify and move to working branch:**

   **CRITICAL**: NEVER commit directly to `main` branch!

   ```bash
   # Check current branch
   git branch --show-current
   ```

   **If currently on `main`:**
   - STOP and create/switch to a feature branch first
   - Example: `git checkout -b feat/issue-XXX` or `git checkout existing-branch`

   **If already on a feature branch:**
   - Verify the branch name is correct
   - Proceed to next step

   **Branch naming convention:**
   - `feat/<description>` for features
   - `fix/<description>` for bug fixes
   - `refactor/<description>` for refactoring
   - `docs/<description>` for documentation
   - `chore/<description>` for tooling/build

4. **Run quality checks:**
   ```bash
   # Using just (recommended)
   just fmt
   just clippy
   just test
   just build

   # Or using cargo directly
   cargo fmt --all
   cargo clippy --workspace -- -D warnings
   cargo test --workspace
   cargo build --release
   ```

5. **Test in-game (if applicable):**
   ```bash
   just dev
   # Verify the changes work correctly in-game
   ```

6. **Stage and commit changes:**

   **File selection:**
   - If specific files were provided as arguments: `$ARGUMENTS`
     → Use: `git add $ARGUMENTS` (commit only specified files)
   - If no arguments were provided:
     → Use: `git add .` (commit all changed files)

   **Commit guidelines:**
   - Create logical, atomic commits
   - Follow conventional commits format: `<type>(<scope>): <description>`
   - Reference issue numbers with "Closes #XXX" (one per line if multiple)
   - Write commit message in English
   - Example: `feat(danmaku): implement BulletGlowMaterial for bullet glow effect\n\nCloses #5`

   **Commit Scopes:**
   - `core`: Core game logic (scarlet-core)
   - `ui`: UI systems (scarlet-ui)
   - `audio`: Audio systems (scarlet-audio)
   - `assets`: Asset management (scarlet-assets)
   - `danmaku`: Bullet/emitter/pattern systems
   - `player`: Player movement, shooting, hitbox
   - `boss`: Boss phase, spell card, movement
   - `collision`: Collision detection & graze system
   - `score`: Scoring system
   - `shader`: WGSL shaders & Material2d
   - `effects`: Visual effects & particles
   - `docs`: Documentation
   - `chore`: Build/tooling

7. **Push changes:**
   ```bash
   git push -u origin <branch-name>
   ```

8. **Create PR using gh command:**
   ```bash
   gh pr create --title "..." --body "..." --assignee itsakeyfut
   ```

**PR Guidelines:**

**MANDATORY: Write PR in Japanese (日本語で記述)**

**MANDATORY PR Body Limit: MAXIMUM 100 LINES**

- **Keep PR body concise** - MUST be under 100 lines
- Follow the structure defined in `.github/PULL_REQUEST_TEMPLATE.md`
- Avoid verbose descriptions or redundant information
- If more details are needed, add them as issue comments instead

**PR Title:**
- Follow conventional commits format in English
- Include scope if applicable
- Example: `feat(danmaku): BulletGlowMaterialで弾グロー発光を実装`
- Example: `fix(collision): グレイズ判定のバグ修正`
- Example: `docs: WGSLシェーダー設計書を追加`

**PR Body Template** (from `.github/PULL_REQUEST_TEMPLATE.md`):
```markdown
## 概要

[このPRが何をするのか・なぜ変更するのかを1〜4文で記述]

## 変更内容

- [変更点1]
- [変更点2]
- [変更点3]

## 関連 Issue

Closes #XXX

<!-- 複数 Issue を閉じる場合は1行ずつ記述すること -->
<!-- Closes #20 -->
<!-- Closes #21 -->

## テスト計画

- [ ] `cargo test --workspace` 通過
- [ ] `cargo clippy --workspace -- -D warnings` 通過
- [ ] `cargo fmt --all -- --check` 通過
```

Please proceed with these steps.
