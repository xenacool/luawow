# Luawow
Luawow is a coding game designed as a first introduction to programming.

```lua
-- Entity and Element management
function createEntity(entityType, x, y, z)
-- Create an entity (player, enemy, machine, etc.) at the specified position
return entityId
end

function getEntityPosition(entityId)
-- Get the current position of an entity
return x, y, z
end

function setEntityPosition(entityId, x, y, z)
-- Set the position of an entity
end

function rotateEntity(entityId, yaw, pitch, roll)
-- Rotate an entity
end

function createElement(elementType, amount, sourceId)
-- Create a specified amount of an element, optionally from a source entity
return elementId
end

function transformElement(sourceElementId, targetElementType, amount)
-- Transform one element into another
return targetElementId
end

-- Spell creation and control
function createSpell(name, elements, effects)
-- Create a spell with specified elements and effects
return spellId
end

function castSpell(spellId, casterId, targetId)
-- Cast a spell from one entity to another (or to a position)
end

function createProjectile(spellId, size, casterId)
-- Create a projectile based on a spell
return projectileId
end

function setProjectileProperties(projectileId, speed, homingStrength)
-- Set properties of a projectile
end

function fireProjectile(projectileId, targetId)
-- Fire a projectile at a target entity or position
end

-- Environment interaction
function detectEntities(sourceId, radius, entityType)
-- Detect entities of a specific type within the specified radius
return entityList
end

function getClosestEntity(sourceId, entityType)
-- Get the closest entity of a specific type to the source
return entityId
end

function applyElementalEffect(elementId, targetId)
-- Apply an elemental effect to a target entity
end

-- Automation and machinery
function createMachine(machineType, x, y, z)
-- Create a machine (conveyor, elemental extractor, spell caster, etc.)
return machineId
end

function setMachineProgram(machineId, program)
-- Set a program for a machine to execute
end

function getEnvironmentalElementSource(elementType, x, y, z)
-- Get or create an environmental source of an element at the specified position
return sourceId
end

-- Utility functions
function wait(frames)
-- Wait for the specified number of frames
end

function getElementalFuel(entityId, elementType)
-- Get the current amount of elemental fuel for the specified entity and element
return fuelAmount
end

-- Example usage:
function setupAutomatedLightningCaster()
    local extractorAir = createMachine("extractor", 0, 0, 0)
    local extractorWater = createMachine("extractor", 5, 0, 0)
    local spellCaster = createMachine("spellcaster", 2.5, 0, 5)

    local airSource = getEnvironmentalElementSource("air", 0, 0, 0)
    local waterSource = getEnvironmentalElementSource("water", 5, 0, 0)
    local function extractAndCastLightning()
        local air = createElement("air", 50, airSource)
        local water = createElement("water", 25, waterSource)
        local lightning = transformElement(air, "lightning", 50)
        local lightningBolt = createSpell("LightningBolt", {lightning}, {"damage", "stun"})
        
        local target = getClosestEntity(spellCaster, "enemy")
        if target then
            castSpell(lightningBolt, spellCaster, target)
        end
        
        wait(100)  -- Wait 100 frames before casting again
    end

    setMachineProgram(extractorAir, function() createElement("air", 10, airSource) end)
    setMachineProgram(extractorWater, function() createElement("water", 5, waterSource) end)
    setMachineProgram(spellCaster, extractAndCastLightning)
end
```

# (Original template)Foxtrot

The all-in-one Bevy 3D game template.

https://user-images.githubusercontent.com/9047632/226387411-70f662de-0681-47ff-b1d1-ccc59b02fa7b.mov

## What does this template give you?

- Integration with Blender as an editor via
  the [`Blender_bevy_components_workflow`](https://github.com/kaosat-dev/Blender_bevy_components_workflow) set of tools
- Physics via [`bevy_xpbd`](https://crates.io/crates/bevy_xpbd_3d)
    - A 3D character controller via [`bevy-tnua`](https://crates.io/crates/bevy-tnua)
- Audio via [`bevy_kira_audio`](https://crates.io/crates/bevy_kira_audio)
- Pathfinding via [`oxidized_navigation`](https://crates.io/crates/oxidized_navigation)
- [`bevy_editor_pls`](https://crates.io/crates/bevy_editor_pls) bound to 'Q'
    - Custom editor found in the windows selection for `bevy_editor_pls`.
- Animations
- Dialogs via [`Yarn Spinner for Rust`](https://crates.io/crates/bevy_yarnspinner)
- Shaders, using the code from [DGriffin's tutorial](https://www.youtube.com/watch?v=O6A_nVmpvhc)
- Smooth cameras via [`bevy_dolly`](https://crates.io/crates/bevy_dolly)
- Particle effects via [`bevy_hanabi`](https://crates.io/crates/bevy_hanabi)
- Procedural skies via [`bevy_atmosphere`](https://crates.io/crates/bevy_atmosphere)
- Grass via [`warbler_grass`](https://crates.io/crates/warbler_grass)

## Usage

Simply use the [template button on GitHub](https://github.com/janhohenheim/foxtrot/generate) to create a new repository
from this template.
Then, replace all instances of the word `foxtrot` with the name of your game. Change the game version as well as the
author information in the following files:

- `Cargo.toml`
- `build/windows/installer/Package.wxs`
- `build/macos/src/Game.app/Contents/Resources/Info.plist`

### Updating assets

You should keep the `credits` directory up to date. The release workflow automatically includes the directory in every
build.

### Updating the icons

1. Replace `build/windows/icon.ico` (used for windows executable and as favicon for the web-builds)
2. Replace `build/macos/icon_1024x1024.png` with a `1024` times `1024` pixel png icon and run `create_icns.sh` (make
   sure to run the script inside the `macos` directory) - _Warning: sadly this seems to require a mac..._

## Help and Discussion

Feel free to shoot a message in the
dedicated [help thread on the Bevy Discord](https://discord.com/channels/691052431525675048/1110648523558506597)
or [open an issue on GitHub](https://github.com/janhohenheim/foxtrot/issues/new) if you want to discuss something or
need help :)
