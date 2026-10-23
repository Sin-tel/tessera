# Tessera 0.1.3

## New features

### Tuning system

v0.1.2 had a single built-in tuning (meantone). Tuning is now a project setting, with presets and notation options.

There is a dedicated panel for tuning options in the settings tab. Tunings can be selected from three categories:
**Temperament**, **Equal** and **Just Intonation**, each with a list of presets.

Each preset comes with different options for how it should be notated:
plain (letters and sharps/flats only), one or more extra accidentals (ups and downs, HEJI-style),
neutral (half sharps and flats for quarter tones), and Johnston notation for just intonation.

- All notes will be automatically converted to the new tuning when you press apply.
- **Snapping follows the tuning**: Diatonic, Chromatic and Fine scales are derived from the chosen tuning.
- **Tuning center** (`C`): moves the tonic to the selected note.
- **Relative note names** (Options menu): names notes relative to the center (movable-do style) instead of absolutely.
- **Respell** (`Shift + R`): cycles a note through its simplest enharmonic spellings. With several notes selected,
  the first press simplifies them all and further presses cycle them.
- **Auto simplify** (Options menu): moved and transposed notes are respelled immediately when a simpler spelling exists.

### VST3 instruments (experimental)

VST support is experimental. Only instruments are supported, no effects.
So far, Pianoteq and Surge XT are known to work on Windows, but most MPE-compatible plugins should work.
If you use macOS or Linux, please try it and report how it goes.

- Plugins in the standard system folders are scanned, and found instruments appear in the "Add channel" list.
  There is a **Rescan** button in Settings -> Plugins.
- For now, only **Pianoteq** and **Surge XT** are enabled after a scan. Other plugins are found but not listed yet.
- Per-note pitch bend and pressure are sent via MPE.
- Set the pitch bend range (2 or 48 semitones) to match the plugin's setting.

### Master channel

- Projects now have a master channel, so effects can be added to the whole mix.

### New devices

- **Organ**: electronic drawbar organ with tonewheel emulation.
- **Rotary**: rotary speaker simulation, intended to be used with the organ, but also fun on other things.
- **Delay Tape**: tape echo modelled on a multi-head machine.
- **Reverb Room**: dense room reverb.

## Improvements

- **Crash recovery**: When Tessera runs into an error, the project is saved to `out/crash_recovery.sav` before the error screen.
  After a crash, the next start doesn't auto-load the last save (in case it caused the crash).
- **macOS**: the release is a universal binary (Apple Silicon and Intel).
- **New application icon.**
- **Rendering is reproducible.** Every instrument and effect is now fully reset before a render, so no leftover tails
  from earlier playback end up in the file, and rendering the same project twice gives identical output.

### Devices

- **Epiano:**
  - Tone and velocity adjustments.
  - Each key now sounds the same every time it's played.
- **Compressor:**
  - Stereo-linked: both channels get the same gain reduction, which keeps the stereo image stable.
  - The detector is a bit more sensitive to high frequencies.
- **Convolution:** output level is increased by 6 dB.

## Upgrading from 0.1.2

- **Projects from 0.1.2 open as before.** They get a master channel and the default meantone tuning, so everything plays back the same.
- **Settings are reset on first launch.** Audio device, MIDI devices and buffer size are stored per version now, so you'll need to
  pick your audio and MIDI devices once when you update.
- **Linux:** PipeWire is now enabled alongside JACK. Building from source needs the PipeWire development package
  (`libpipewire-0.3-dev` on Debian/Ubuntu, `pipewire-devel` on Fedora).

---

# Tessera v0.1.2

### Instruments
*   Sampler: Polyphonic sampler with high-quality resampling.
*   Wavetable: The Wavetable device is now polyphonic and feature complete. Includes wavetable selection, unison, envelope and LFO modulation.

### Effects
*   Convolution Reverb: Added the ability to select different impulse responses at runtime. Update to a more efficient algorithm.
*   Decimator: Classic sample-rate reduction effect.
*   Limiter: Analog style limiter with stereo linking and soft-clipping.
*   Phaser, Chorus, Tremolo: Basic modulation effects.

### UI
*   Snapping: New context menu (right-click on the canvas) to configure time and pitch snapping separately.
*   Tempo tool: Utility tool to scale the project tempo.
*   Metronome: Added a built-in metronome toggle in the options menu.
*   Auto-zoom: Press `.` (period) to zoom the view to fit all notes.
*   Per channel gain: Added a dedicated gain slider per channel.
*   Update checks: Tessera now checks for updates on startup and notifies you in the settings tab if a new version is available.
*   Logging: Error logs are now displayed in the settings tab for easier bug reporting.

---

# Tessera v0.1.1

* Fix crash on linux (#5)
* Add a check so you don't accidentally overwrite save file
* Various quality fixes (declicking, glitches)

Note on versions: save files and settings should be compatible across patch (0.1.x) versions, this is now checked automatically.

---

# Tessera v0.1.0

First release. Only Windows is tested, so the others might not work.

