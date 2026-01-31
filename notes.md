tuning:
 * make categories (ET / temp / JI)
 * adjust notation

rastmic / neutral
  notation should use half-sharps if 243/242 is tempered
  rank 2 ~ 2.3.11 (33/32)
  rank 3 ~ 2.3.5.11 (81/80, 33/32) also can temper out 5120/5103 (81/80~64/63, 33/32)
  rank 4 ~ 11-limit

  some EDOs can also be notated like this (notably 31 and 41!)

move 'flush_messages' to backend

be more consistent with levels.
target should be ~ -18dB RMS

edit sustain
option to only edit active track (auto lock)
mono/poly switch (make sure mpe works, start w simple poly)
loop
snap transport time

- save layout
- project settings / tuning presets

if we crashed, don't load previous save automatically!

use C4 on backend for consistency

add lock all/none button on top?

ui/layout
  reduce the number of ad-hoc ui elements, make layout system more generic

more options for body responses in convolver
 -> maybe split menu into categories
 also add cabinet / microphone splits

panner: instead of only binaural panning also allow for some kind of virtual room
 - emulate common microphone setups (m/s, ORTF, AB, etc)

transient shaper

mute icon speaker

try to do something more sensible for snapping notes outside of the scale

merge undos for same actions (esp move commands)

move MIDI handling to Rust

ramp/envelope testing for compressor / limiter

note preview on move

transport buttons

autosave / crash recovery

implement delayline with interpolating reader

allow clone notes to be triggered when ctrl is pressed after initial drag

add some testing harness for instruments and effects
 -> test if process - flush - process produces identical outputs
  needs a macro helper to setup without too much copy-paste

File
 - reset workspace

Right click view header
 - close
 - split vertical
 - split horizontal
 - set to

stencil strokes: performance / quality?

display midi activity indicator for each device

in principle, the stream and render instances could be seperated
  allows for rendering even when there's no stream

collapse sections in device settings

allow only one instance of each view?
 makes some logic easier

add view resized callback
add view enter callback

add some way to tune instruments properly
 - can be done semi-automatically?

render in HDR and add bloom?

Linux: "null" ("Discard all samples ...") is used as default device. Should pick something else.

should we use audio_thread_priority?

modulation routing system for instruments would be nice to have
 e.g. allow pressure to be applied to any slider

## routing

grouping: tree structure of channels
 - group can be either just mix bus or an "instrument rack" type thing
    how to switch between these?
    instrument group can be a "fake" instrument with its own Roll
 - send tracks can be handled seperately (need to be processed just before master)
    do this later, maybe have a fixed max amount of sends


## devices

BBD delay
  time: discrete clock settings + flexible time settings w interpolation

Reverb: dispersion setting (lowest should be more delay-ish)

Gain: make util instead (gain / pan / width)
incrase max gain

Analog delay
  - mode switch
  BBD / bode shifter / pitch


## visuals

triple buffer generic vec<f32> for every device

```rust
pub struct Wavetable {
  // ...
  visual_tx: triple_buffer::Input<Vec<f32>>,
}


pub trait Instrument {
  fn new(sample_rate: f32) -> (Self, Option<Output<Vec<f32>>>)
}


pub struct AudioContext {
  visuals: HashMap<usize, triple_buffer::Output<Vec<f32>>>,
}

audio.set("get_visual", lua.create_function(|lua, handle: usize| {
    let state = lua.app_data_ref::<State>().unwrap();

    if let Some(output) = state.visuals.get(&handle) {
        let buf = output.read();
        return Ok(Some(buf.clone()));
    }
    Ok(None)
})?)?;

```
Hashmap is index by some handle, similar to how meters work.
Then we add add a widget for the specific device e.g.

```lua
function Visualizer:draw(ui, x, y, w, h)
    local data = tessera.audio.get_visual(self.handle)

    if data then
        tessera.graphics.set_color(theme.line)
        local points = {}
        local step = w / #data
        for i, val in ipairs(data) do
            table.insert(points, x + (i-1) * step)
            table.insert(points, y + h/2 - (val * h/2))
        end
        tessera.graphics.polyline(points)
    end
end
```

## building

on WSL:
# compiling for alsa
sudo apt install libasound2-dev
# windowing support
sudo apt install libwayland-dev libxkbcommon-dev libegl1-mesa-dev
# alsa
sudo apt install libasound2-plugins pulseaudio
# jack
sudo apt install libjack-dev

## note datastructure:
pitch: {
    interval: array of harmonic coordinates
    time: float, // start_time
    velocity: float,
    verts: list of vert
}
vert: {
    [1] time: float // relative (first point always 0)
    [2] pitch_offset: float
    [3] pressure: float
}

## piano roll

piano roll playback
 - there is NO automatable bpm/speed setting
 - everything happens on the canvas *as is*.
 - makes things easier, at the expense of some tricks you can do by automating BPM
 - will need robust time manipulation tools to cope
 - (maybe a global speed mult because thats easy)

## envelope

also make ADS envelope for FM modulator
 how should these interact with pressure?
make some kind of nice "universal" envelope that works well with both pressure and velocity inputs
"universal" envelope for non-mpe input?
previous work
 - reason friktion
 - expressive E noisy 2

## instruments
double reed
flute model
bowed string/modal
extended 808 bass model

pluck model:
* multiple strings?
    - coupling -> two stage decay
    - rotation matrix

## tuning
make tuning import/export and add to project settings
add generic transposition / movement system
port over the tracking scripts for midi input

### Custom note font
maps ascii characters to accidentals
combinations are done mostly through ligatures, with some exceptions (#x)

Note names are just uppercase A, B, C, etc.
other characters:
`()+,-./0123456789:<=>~`

characters:
a: single flat b
b: natural
c: single sharp #
d: double sharp x
e: 1/2 flat d
f: 1/2 sharp t
g: 3/2 sharp
e: 1/2 sharp alt (HEJI)

l: small + (Johnston)
m: small - (Johnston)

n: septimal down L (HEJI)
o: septimal up (HEJI)
p: septimal down L (johnston)
q: septimal up 7 (johnston)

r: arrow up
s: arrow down

v: caret down
w: caret up

testing:
#: double caret down vv
$: double caret down stacked

## MIDI
also need midi out ports
linnstrument custom mode

midi impl is now kind of stupidly polling, but we can get better latency if we do everything in rust directly in the backend
 - bit more work to get right though since that means more logic on rust side
 - this is good though, since eventually we want to move as much as possible to rust
 - for now, lets leave it like this!

## other notes

reverb
https://ccrma.stanford.edu/~dattorro/EffectDesignPart1.pdf
https://msp.ucsd.edu/techniques/v0.11/book-html/node111.html


figure out ZDF for distortion with HP feedback

add noise to epiano? better impact model

support surge wavetable format?

port over tubes
-> add zdf feedback?
-> add DC offset compensation signal feedback path

delay differential equations

https://deps.rs/repo/github/Sin-tel/tessera

physmod delay line modulation by input (fm-ish)

make simper f32x4 bandpass for modal

static filter optimization?

use smoothstep for dry/wet
for crossfading (wet/dry and such)
https://signalsmith-audio.co.uk/writing/2021/cheap-energy-crossfade/
should use amplitude / energy preserving depending on context

debugger where you can type lua commands and monitor variables (pretty print tables, maybe even with a foldable UI)
(command palette?)

custom keyboard shortcut config?

spiral fft (maybe even something like harmonic CQT)

correct LUFS loudness monitoring

synth where osc is nonlinear system

some kind of simple but nice polysynth with good MPE
 - phase dist?

wavetable should be "more fun" and try to emulate low resolution variable speed DACs
 - focus on PPG wave esque sounds
 - maybe something like Klevgrand Tomofon is nicer

bell, modal, gamelan synth

try sine wave ODE system (modified coupled form)
```
x = x - ey;
y = ex + y;

e = 2 * sin(pi * freq / sr)

at low freqs:
e = 2 * pi * freq / sr
```
https://www.kvraudio.com/forum/viewtopic.php?t=412674&start=15


allpass interpolation
https://quod.lib.umich.edu/cgi/p/pod/dod-idx/lossless-click-free-pitchbend-able-delay-line-loop.pdf?c=icmc;idno=bbp2372.1997.068;format=pdf

## Tuning

4 layers:
  note names  (only view)
    | convert diatonic scale notation to oct/fifth pair
  notation coordinates (internal representation)
    | look up generator mapping
  generator coordinates
    | look up generator sizes
  pitch in semitones/midi number

info for tuning specification:
  - size of all generators (first two are period and gen)
  - which accidentals are used
     - mapping of these to generators
  - root of diatonic (default C)
  - size of diatonic scale + how many gens down
  - (optional chromatic scale?)

  there are seperate accidentals for "double half sharp/flat"
when 243/242 is tempered out (rastmic), these should revert to normal sharps/flats
(this is true for 17, 24, 31, 41, 72)

load tuning from text file (just lua?)

