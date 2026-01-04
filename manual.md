# Tessera User Manual

## 1. General

Tessera uses a flexible tiling UI. Panes can be resized by dragging the borders.

> [!NOTE]
> On macOS, use the `command` and `option` keys instead of `ctrl` and `alt` respsectively.

### File Operations
| Action                     | Shortcut                  |
| :---                       | :---                      |
| **New project**            | `Ctrl + N`                |
| **Open project**           | `Ctrl + O`                |
| **Save project**           | `Ctrl + S`                |
| **Save as...**             | `Ctrl + shift + S`        |
| **Render audio**           | `Ctrl + R`                |
| **Cancel render**          | `Ctrl + C` or `Esc`       |
| **Quit**                   | `Esc`                     |

### Transport (Playback)
| Action                     | Shortcut                  |
| :---                       | :---                      |
| **Play / Stop**            | `Space`                   |
| **Toggle Recording**       | `B`                       |

### Interface
| Action                     | Shortcut                  |
| :---                       | :---                      |
| **Undo**                   | `Ctrl + Z`                |
| **Redo**                   | `Ctrl + Y`                |
| **Next Tab**               | `Ctrl + Tab`              |
| **Previous Tab**           | `Ctrl + Shift + Tab`      |


## 2. The Canvas (Piano Roll)
Clicking anywhere in the top ribbon will set the transport start time.

> [!TIP]
> Tessera uses an infinite canvas. If you ever get lost, pressing the `.` (period) key will bring the project back into view.

**Middle mouse / Ctrl + alt + left mouse:** Pan view.

**Right click:** Open the context menu. Here you can adjust the snapping settings.

**Scroll:** Zoom.

### Edit Tool
**Left click:**
* Background: Box selection.
* Note body: Move note. (Hold shift to constrain to one axis.)
* Note tail: Resize note.

**Ctrl + drag:** Clone notes.

**Alt + drag:** Adjust velocity up/down.

### Pen Tool
**Left click:** Draw notes.

### Tempo Tool
**Drag left/right:** Change location of tempo marker (first downbeat).

**Drag up/down:** Change tempo. This currently only affects the grid, not the playback speed.


### Tools & Selection
| Action                     | Shortcut                  |
| :---                       | :---                      |
| **Toggle tool** (pen/edit) | `Tab`                     |
| **Zoom to fit**            | `.`                       |
| **Select all notes**       | `Ctrl + A`                |
| **Delete notes**           | `Delete` or `Backspace`   |
| **Copy**                   | `Ctrl + C`                |
| **Cut**                    | `Ctrl + X`                |
| **Paste**                  | `Ctrl + V`                |

### Transformations
| Action                     | Shortcut                  |
| :---                       | :---                      |
| **Grab**                   | `G`                       |
| **Scale**                  | `S`                       |

### Nudging Notes (Time)
Use `Left` and `Right` arrow keys to move selected notes in time.

| Modifier                   | Amount                    |
| :---                       | :---                      |
| **None**                   | Whole note                |
| **Shift**                  | Quarter note              |
| **Ctrl**                   | 1/16th note               |
| **Alt**                    | 1/128th (fine)            |

### Transposing Notes (Pitch)
Use `Up` and `Down` arrow keys to transpose selected notes.

| Modifier                   | Interval                  |
| :---                       | :---                      |
| **None**                   | Diatonic step             |
| **Shift**                  | Octave                    |
| **Ctrl**                   | Chromatic step (semitone) |
| **Alt**                    | Comma (diesis)            |

## 3. Channels & Effects

### Channels
Click on a channel to select it.

#### Properties
* Arm: Route midi input to this channel. Use shift + click to arm multiple tracks.
* M: Mute channel. Double click to solo.
* Eye: Hide/unhide channel. Double click to hide all channels except this one.
* Lock: Lock/unlock channel for editing. Double click to lock all channels except this one.

| Action                     | Shortcut                  |
| :---                       | :---                      |
| **Delete channel**         | `Delete` or `Backspace`   |

### Channel settings
Click anywhere in a device to select it. Clicking on the header will fold it up.
The toggle button will mute/unmute the device.

When an effect is selected in the 'Channel settings' panel:

| Action                     | Shortcut                  |
| :---                       | :---                      |
| **Remove effect**          | `Delete` or `Backspace`   |
| **Move effect up**         | `Shift + up`              |
| **Move effect down**       | `Shift + down`            |

