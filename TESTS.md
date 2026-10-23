# Tests

Use this file as a guideline for testing the various features in the practice tool.

# Versions

- [ ] 1.02.0
- [ ] 1.02.1
- [ ] 1.02.2
- [ ] 1.02.3
- [ ] 1.03.0
- [ ] 1.03.1
- [ ] 1.03.2
- [ ] 1.04.0
- [ ] 1.04.1
- [ ] 1.05.0
- [ ] 1.06.0
- [ ] 1.07.0
- [ ] 1.08.0
- [ ] 1.08.1
- [ ] 1.09.0
- [ ] 1.09.1
- [ ] 2.00.0
- [ ] 2.00.1
- [ ] 2.02.0
- [ ] 2.02.3
- [ ] 2.03.0
- [ ] 2.04.0
- [ ] 2.05.0
- [ ] 2.06.0
- [ ] 2.06.1
- [ ] 2.06.2
- [ ] 2.07.0
- [ ] 2.07.1
- [ ] 2.07.2

# Test tasks

## Startup

- [ ] Installed as `dinput8.dll`: holding right shift at startup starts the tool.
- [ ] Installed as `dinput8.dll`: running the exe starts the copy the game loaded.
- [ ] Running the exe from another folder injects its own DLL.
- [ ] Running the exe twice reports that the tool is already running.
- [ ] Unsupported game version: a message box explains it.
- [ ] EAC not bypassed: the exe offers to bypass it and writes `steam_appid.txt`.

## Flags

- [ ] All No Damage: neither you or enemies should take damage.
- [ ] No Death: you and enemies take damage, but you don't die.
- [ ] No Update AI: enemies start/stop their AI.
- [ ] One shot: you deal 999999 damage.
- [ ] Inf Consumables: consumable count doesn't go down when used.
- [ ] Inf Stamina: stamina doesn't deplete when attacking/running during a fight.
- [ ] Inf Focus: focus doesn't deplete when using Ashes of War or Spells in a fight.
- [ ] No Gravity: you are able to walk on air.
- [ ] No Collision: you should fall below the floor.
- [ ] Deathcam: should toggle top-down camera.
- [ ] Hitbox flags: should appear/disappear.
- [ ] Stable pos: a red ball should appear below the character.
- [ ] Show/hide map/character: self-descriptive.
- [ ] Multi flag: toggles all of its flags at once.

## Widgets

- [ ] Savefile manager: should load the selected savefile.
- [ ] Item spawn: should spawn the desired item.
- [ ] Item spawn: should apply upgrades and affinity correctly to the spawned item.
- [ ] Item spawn: should provide the specified amount of items.
- [ ] Item spawn: should filter correctly.
- [ ] Edit stats: should apply the correct stats and not crash for each stat.
- [ ] Save position: should change the numbers on 2nd line when saving.
- [ ] Load position: should reposition and reorient character.
- [ ] Nudge: should change character's height.
- [ ] Speed: should change character's animation speed.
- [ ] Color: should cycle the collision mesh colors.
- [ ] Add runes: self-descriptive.
- [ ] Warp: should warp to the selected grace.
- [ ] Quitout: should exit to main menu.
- [ ] Target: should show the info of the targeted enemy.
- [ ] Input viewer: should show controller and keyboard input.
- [ ] Group: should open a submenu with its widgets.
- [ ] Radial menu: opens with the configured combo, blocks the controller while it's open, and
  triggers the selected entry.
- [ ] Long menus scroll instead of overflowing the screen.

## Indicators

- [ ] Game version, IGT, FPS, Animation, ImGui debug info: should show.
- [ ] Player position, Player velocity: should update as you move.
- [ ] Player distance: should reset to zero with "Start XYZ", and grow as you move away.
- [ ] Frame counter: should reset with "Reset".

## Settings

- [ ] Config editor: change a hotkey and an indicator, reorder and delete an entry, then save and
  see the widgets reload.
- [ ] Config editor: an invalid edit shows an error with its key path.
- [ ] Update: the update button appears in the overlay when an update is available.
- [ ] `disable_update_prompt = true`: no update button appears.
