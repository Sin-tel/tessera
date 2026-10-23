# Tessera User Manual

## 1. General

Tessera uses a flexible tiling UI. Panes can be resized by dragging the borders.

> [!NOTE]
> On macOS, use the `command` and `option` keys instead of `ctrl` and `alt` respectively.

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

### Options menu
| Option                     | Summary                   |
| :---                       | :---                      |
| **Preview notes**          | Play notes as you draw them with the pen tool. |
| **Chase notes**            | When playback starts in the middle of a note, play that note too instead of skipping it. |
| **Follow**                 | Scroll the canvas along with the playhead during playback. |
| **Metronome**              | Click on every beat, accented on the first beat of each bar. |
| **Relative note names**    | Display notes relative to the tuning center (see [Tuning](#4-tuning)). |
| **Auto simplify**          | Respell moved notes when a simpler spelling exists (see [Tuning](#4-tuning)). |


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
| **Respell** (cycle enharmonic spellings) | `Shift + R` |
| **Set tuning center**      | `C`                       |

Respelling a single note cycles through its simplest few spellings. With several notes selected, the first press
simplifies them all, pressing again cycles them. See [Tuning](#4-tuning) for the tuning center.

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
| **Alt**                    | Smallest step (see below) |
| **Ctrl + Alt**             | Second extra accidental   |
| **Shift + Alt**            | Third extra accidental    |
| **Ctrl + Shift**           | Alternative chromatic step |

The small steps depend on the current tuning and notation. In an equal temperament, the extra accidental is usually just one step up or down. See [Notation](#5-notation) for more information.

* **Alt** moves by the first extra accidental. Without extra accidentals, it moves one step in an equal temperament,
  or otherwise by a Pythagorean comma (e.g. C# to Db).
* **Ctrl + Alt** and **Shift + Alt** move by the second and third extra accidental, if the notation has them.
* **Ctrl + Shift** moves by an alternative chromatic semitone, built with the first accidental: 25/24 when that's the
  accidental for 5, or 28/27 when it's the one for 7.

If the current tuning doesn't have a step, it does nothing.

## 3. Channels & Effects

### Channels
Click on a channel to select it. **Add channel** lists the instruments, including enabled VST plugins
(see [VST instruments](#vst-instruments-experimental)).

The first channel is the master channel. Its effects apply to the whole mix. It has no instrument and can't be
deleted.

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

### VST instruments (experimental)
VST3 instrument plugins can be used like the built-in instruments. Support is experimental and only a few plugins have been tested so far.

* Plugins are scanned from the standard VST3 folders:
  * Windows: `C:\Program Files\Common Files\VST3`
  * macOS: `/Library/Audio/Plug-Ins/VST3` and `~/Library/Audio/Plug-Ins/VST3`
  * Linux: `~/.vst3`, `/usr/lib/vst3` and `/usr/local/lib/vst3`
* Only plugins on the whitelist are enabled after a scan, which currently includes 'Pianoteq' and 'Surge XT'. Other plugins are found but don't show up in
  the instrument list yet.
* **Rescan** in Settings -> Plugins scans again, for example after installing a plugin.
* **Show UI** opens the plugin's own window.
* **Pitch Bend Range** must match the plugin's pitch bend range (2 or 48 semitones) for notes to be in tune.
* Notes are sent as MPE (lower zone, 15 channels) with per-note pitch bend and pressure, so microtonal pitches and pressure changes work
  with any plugin that supports MPE.

## 4. Tuning
Every project has a tuning, which sets the pitch of each note and how notes are displayed. It's chosen in the
**Project Settings** panel, on the right of the **Settings** tab.

* **Category**: Temperament, Equal or Just Intonation.
* **Tuning**: a preset from that category.
* **Notation**: how notes are written. "Plain" uses only letters with sharps and flats. Other options add extra
  accidentals (like ups and downs), half sharps and flats (Neutral), or Johnston notation for just intonation.
  Each option shows how the primes are notated, and the recommended one is selected by default.
* **Apply** switches the project to the selected tuning and notation. Existing notes are converted, and a note next to the button says what will happen:
  If the tuning is compatible, notes will only be respelled to keep their pitch.
  If the tuning is different but the notation has the same accidentals, notes will keep their spelling but change pitch.
  Otherwise, accidentals will be dropped.
  Changing the tuning can be undone, so you can freely experiment with other systems.
* **Input map** shows which note, and which (approximate) ratios each MIDI key in an octave plays.

The snapping scales (Diatonic, Chromatic, Fine) are chosen automatically based on the tuning.

### Tuning center
`C` in the canvas makes the selected note the tuning center (root). With nothing selected, `C` resets the center to C.
The center decides how notes are named and how the grid and snapping line up. This only affects display, pitches don't change.

With **Relative note names** (Options menu) on, notes are named relative to the center instead, as in a movable-do system.

### Spelling
The same pitch can often be written in several ways (e.g. G# and Ab, or with extra accidentals). `Shift + R`
respells the selected notes (see [Transformations](#transformations)).

With **Auto simplify** (Options menu) on, notes that are moved or transposed get a simpler spelling when one exists.
A selection is respelled as a whole, so chords keep their spelling.

## 5. Notation
Tessera uses a unified notation system, combining ideas from existing notations: HEJI (the Helmholtz-Ellis JI Pitch Notation) for
just intonation, ups and downs for equal temperaments, and Stein-Zimmermann half sharps and flats.

All systems currently only go up to 11-limit.

### Letters, sharps and flats
The basic letters come from a chain of fifths, F C G D A E B. Going further along the chain adds sharps (F# C# G# ...) or
flats (Bb Eb Ab ...). A sharp raises by a chromatic semitone: seven fifths up and four octaves down, 2187/2048 in just
intonation. With the **Plain** notation, every note is written with these alone.

### Extra accidentals
Some tunings can not be written using the plain notation, and even if they can, readability is often improved by introducing extra accidentals.
Each accidental stands for a comma, associated with a prime, as in HEJI:

| Prime  | Associated comma |
| :---   | :---             |
| 5      | 81/80            |
| 7      | 64/63            |
| 11     | 33/32            |

Ups and downs (`^`/`v`) are subject to some specific rules for consistency:
* Ups and downs go in front of the letter, the others after the sharps and flats.
* In equal temperaments, they are always one step.
* If a notation has only one extra accidental, it's always written as ups and downs.
* In just intonation, the ups and downs are re-used for 81/80 instead of HEJI arrows.

### Half sharps and flats
Some tunings split the chromatic semitone into two equal parts. In equal temperaments, these are the ones where a
sharp is an even number of steps, such as 24, 31, 41 and 72 equal. The **Neutral** notation writes these with half
sharps and flats. Two half sharps make a sharp, so they're always combined.

### Johnston notation
Ben Johnston's notation is an alternative for just intonation. Its natural notes form a just C major scale:
C-E-G, G-B-D and F-A-C are pure 4:5:6 triads, so E, A and B are 5/4, 5/3 and 15/8 above C. As a result the natural
notes aren't a chain of pure fifths: D to A is a comma (81/80) short. Sharps and flats raise and lower by 25/24, the
difference between a major and a minor third.

This is less systematic than the main notation, but music built on 5-limit harmony needs fewer accidentals.
