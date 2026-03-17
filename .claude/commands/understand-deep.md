# understand-deep - Comprehensive Codebase Analysis

## Overview

Provides **comprehensive and detailed** analysis of specific files, structs, or features with extensive documentation, code examples, and dependency graphs. Use this for deep dives into complex implementations.

For quick overview, use the `understand` command instead.

## Usage

```bash
understand-deep <target>
```

**target** can be any of:
- **File path**: `app/core/src/systems/collision.rs`
- **Module name**: `danmaku`, `collision`, `boss`
- **Struct/Enum name**: `BossPhaseData`, `SpatialGrid`, `BulletEmitter`
- **Feature name**: `graze-system`, `spell-card-bg`, `bullet-glow`

## Output Format

Generates detailed documentation in Markdown format containing:

### 1. Overview Section
- **Purpose**: What this feature/module does
- **Responsibility**: Role within the system
- **Related Phase**: Which roadmap phase this belongs to

### 2. Type Definitions Section
- **Struct/Enum definitions**: All fields and their types
- **Doc comments**: Existing documentation comments
- **Derive traits**: List of derived traits
- **Visibility**: pub/pub(crate)/private distinction

### 3. impl Blocks Section
- **Method list**: All method signatures
- **Method classification**: Constructors, getters, setters, conversions, helpers
- **Important methods**: Highlighted methods to understand

### 4. Trait Implementations Section
- **Standard traits**: Debug, Clone, Serialize, Deserialize, etc.
- **Custom traits**: Project-specific traits (Material2d, AsBindGroup, etc.)
- **Trait bounds**: Traits required in generics

### 5. Dependencies Section
- **Imports**: What this file imports
- **Usage locations**: Where this file's types are used
  - File paths and context
  - Common usage patterns
- **Dependency graph**: ASCII art visualization

### 6. Usage Examples Section
- **Real code examples**: Actual usage extracted from codebase
- **Test code**: Usage patterns from unit tests
- **WGSL shader**: Shader file content (when applicable)

### 7. Related Files Section
- **Related modules**: Files to understand together
- **Documentation**: Related docs in docs/ directory
- **Next files to read**: Recommended path to deepen understanding

## Processing Steps

1. **Identify target**
   - Search for target file using Glob/Grep
   - Display list for confirmation if multiple candidates exist

2. **Parse file**
   - Read target file with Read tool
   - Parse structs, enums, impl blocks, traits
   - Extract doc comments

3. **Analyze dependencies**
   - Search for type names across all files using Grep
   - Identify usage in use statements, field types, method arguments
   - List imports (what this file depends on)

4. **Extract usage examples**
   - Extract code examples from actual usage locations
   - Get usage patterns from test files
   - Search `assets/shaders/*.wgsl` for shader examples (when applicable)

5. **Generate ASCII diagram**
   - Visualize dependencies in ASCII art graph format
   - Distinguish imports (dependencies) and usages
   - Use box drawing characters for clear visualization

6. **Output Markdown**
   - Detailed documentation organized by section
   - Proper syntax highlighting in code blocks
   - Embedded ASCII diagrams

7. **Save to file**
   - Create `.claude/tmp/` directory if it doesn't exist
   - Save output to `.claude/tmp/understand-deep_<target>.md`
   - Sanitize target name for filename (replace `/`, `::`, spaces with `_`)
   - Display file path to user after completion

## Example Output

```markdown
# `SpatialGrid` - Spatial Partitioning for Collision

## Overview

**Purpose**: Grid-based spatial partitioning to optimize collision detection between many bullets and enemies

**Responsibilities**:
- Partition entities into grid cells for O(1) neighbor lookup
- Reduce collision checks from O(n²) to O(n)
- Support 500+ simultaneous bullets at 60fps

**Phase**: Phase 3 (Collision System)

---

## Type Definitions

### `SpatialGrid` Struct

\`\`\`rust
#[derive(Resource, Default)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), Vec<Entity>>,
}
\`\`\`

**Fields**:
- `cell_size: f32` - Grid cell size in pixels (default: 64.0)
- `cells: HashMap<(i32, i32), Vec<Entity>>` - Entity lists keyed by grid position

**Derived Traits**: `Resource, Default`

---

## impl Blocks

### Constructors

\`\`\`rust
pub fn new(cell_size: f32) -> Self
\`\`\`

### Grid Operations

\`\`\`rust
pub fn clear(&mut self)
pub fn insert(&mut self, pos: Vec2, entity: Entity)
pub fn get_nearby(&self, pos: Vec2, radius: f32) -> Vec<Entity>
fn world_to_cell(&self, pos: Vec2) -> (i32, i32)
\`\`\`

---

## Dependencies

### What This File Imports

\`\`\`rust
use bevy::prelude::*;
use std::collections::HashMap;
\`\`\`

### Where This File's Types Are Used

- `systems/collision.rs`: Populated each frame, queried for bullet-player collisions
- `systems/graze.rs`: Queried for graze detection (graze radius vs hitbox radius)

### Dependency Graph

\`\`\`
┌─────────────────────────────────────────────────────────────┐
│                     spatial_grid.rs                          │
│             (Spatial Partitioning Resource)                  │
└────────┬────────────────────────────────┬──────────────────┘
         │                                │
         │ Dependencies (imports)         │ Used by
         │                                │
    ┌────▼────────────────┐         ┌────▼──────────────────────┐
    │ bevy::prelude       │         │ systems/collision.rs       │
    │ std::HashMap        │         │  - Bullet-player detection │
    └─────────────────────┘         └───────────────────────────┘

                                    ┌───────────────────────────┐
                                    │ systems/graze.rs           │
                                    │  - Graze radius detection  │
                                    └───────────────────────────┘
\`\`\`

---

## Usage Examples

### Actual Usage

**Usage in systems/collision.rs**:
\`\`\`rust
pub fn update_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    bullet_query: Query<(Entity, &Transform), With<EnemyBullet>>,
) {
    grid.clear();
    for (entity, transform) in bullet_query.iter() {
        grid.insert(transform.translation.truncate(), entity);
    }
}

pub fn check_player_bullet_collision(
    grid: Res<SpatialGrid>,
    player_query: Query<(&Transform, &CircleCollider), With<Player>>,
    bullet_query: Query<(&Transform, &CircleCollider, &EnemyBullet)>,
    mut events: EventWriter<PlayerHitEvent>,
) {
    let Ok((p_transform, p_col)) = player_query.single() else { return };
    let player_pos = p_transform.translation.truncate();
    let nearby = grid.get_nearby(player_pos, p_col.radius + 16.0);
    for entity in nearby {
        if let Ok((b_transform, b_col, bullet)) = bullet_query.get(entity) {
            let dist = player_pos.distance(b_transform.translation.truncate());
            if dist < p_col.radius + b_col.radius {
                events.send(PlayerHitEvent { bullet_entity: entity });
            }
        }
    }
}
\`\`\`

---

## Related Files

### Related Modules to Understand

1. **Collision** (`systems/collision.rs`) - Consumer of spatial grid for bullet-player
2. **Graze** (`systems/graze.rs`) - Graze detection using nearby bullet lookup
3. **Components** (`components.rs`) - `CircleCollider` radius definitions

### Related Documentation

- [docs/02_architecture.md](../../docs/02_architecture.md) - Collision system design
- [docs/09_quick_reference.md](../../docs/09_quick_reference.md) - Collision radius table
- [docs/roadmap/phase-03.md](../../docs/roadmap/phase-03.md) - Phase 3 tasks

### Next Files to Read

- **Add collision logic**: `systems/collision.rs` → `components.rs`
- **Understand graze**: `systems/graze.rs` → `events.rs`

---

**Generated**: 2026-03-17
**Command**: `understand-deep SpatialGrid`
```

## Output File

After generating the documentation, save it to:
```
.claude/tmp/understand-deep_<sanitized_target>.md
```

**Filename sanitization rules**:
- Replace `/` with `_` (e.g., `app/core/src/collision.rs` → `understand-deep_app_core_src_collision_rs.md`)
- Replace `::` with `_` (e.g., `systems::danmaku` → `understand-deep_systems_danmaku.md`)
- Replace spaces with `_` (e.g., `graze system` → `understand-deep_graze_system.md`)
- Convert to lowercase for consistency
- Remove special characters except `_` and `-`

**Examples**:
- `understand-deep SpatialGrid` → `.claude/tmp/understand-deep_spatialgrid.md`
- `understand-deep app/core/src/collision.rs` → `.claude/tmp/understand-deep_app_core_src_collision_rs.md`
- `understand-deep systems::danmaku` → `.claude/tmp/understand-deep_systems_danmaku.md`
- `understand-deep graze-system` → `.claude/tmp/understand-deep_graze-system.md`

**Directory creation**:
- If `.claude/tmp/` doesn't exist, create it using Bash: `mkdir -p .claude/tmp`
- Always notify the user of the output file path after completion

## Important Notes

- **Accuracy First**: Accurately reflect actual code behavior
- **Real Examples**: Learn from actual code, not theory
- **Comprehensive**: Don't miss any important usage locations
- **Readable**: Write in a way new developers can understand
- **Up-to-date**: Reflect the current state of the codebase
- **Save Output**: Always save the complete documentation to `.claude/tmp/understand-deep_*.md`

## Technical Implementation Guidelines

### Search Strategy

1. **Glob search**: Search by file name pattern
   - `app/**/src/**/${target}*.rs`
   - For module names, also search `**/mod.rs`

2. **Grep search**: Content-based search
   - `struct ${target}` - Struct definitions
   - `enum ${target}` - Enum definitions
   - `impl.*${target}` - impl blocks
   - `use.*${target}` - import statements
   - `${target}::` - Usage locations

3. **Identify dependencies**:
   ```bash
   # imports (this file's dependencies)
   Grep pattern="^use " file_path=target_file

   # usages (used by others)
   Grep pattern="TargetType" path=app/ output_mode=files_with_matches
   ```

4. **For shader-related types**: Also search `assets/shaders/*.wgsl`

### ASCII Diagram Generation Rules

```
┌─────────────────────────────────────────────────────────────┐
│                      target.rs                               │
│                   (Target Module)                            │
└────────┬────────────────────────────────┬──────────────────┘
         │                                │
         │ Dependencies (imports)         │ Used by
         │                                │
    ┌────▼────────────────┐         ┌────▼──────────────────────┐
    │ dependency1         │         │ user1.rs                  │
    │ dependency2         │         │  - Context                │
    │ dependency3         │         └───────────────────────────┘
    └─────────────────────┘
                                    ┌───────────────────────────┐
                                    │ user2.rs                  │
                                    │  - Context                │
                                    └───────────────────────────┘

                                    ┌───────────────────────────┐
                                    │ assets/shaders/*.wgsl     │
                                    │  (shader file reference)  │
                                    └───────────────────────────┘
```

**Format rules**:
- Use box drawing characters (┌─┐│└┘├┤┬┴┼)
- Target module at top in a box
- Dependencies on left side
- Users on right side
- Shader/data files indicated with parentheses
- Keep layout clean and readable in VSCode

### Error Handling

- **File not found**: Suggest similar files
- **Multiple candidates**: Prompt user to select
- **Parse failure**: Output partial information

### Performance Optimization

- Parallelize Grep searches when possible
- For large files, use offset/limit for chunked reading
- Extract only necessary information (don't copy entire content)

## Extensibility

Future features to add:

- **Change history**: Display recent changes from git log
- **Metrics**: Code complexity, test coverage
- **Refactoring suggestions**: Improvement recommendations
- **Interactive mode**: Display details progressively
- **Export**: HTML, PDF output

---

**Purpose of this command**: Help new developers get started quickly and existing developers recall forgotten implementations.
