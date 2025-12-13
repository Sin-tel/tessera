note datastructure:
pitch: {
    pitch: table,
    time: float, // start_time
    velocity: float,
    verts: list of vert
}
vert: {
    [1] time: float // relative (first point always 0)
    [2] pitch_offset: float
    [3] pressure: float
}

util
 find_index
 map
 min
 max


prompt for save as.. when first saving new project

File
 - reset workspace

Right click view header
 - close
 - split vertical
 - split horizontal
 - set to

stencil strokes: performance / quality?

display midi activity indicator for each device
enable MPE in settings for each device

in principle, the stream and render instances could be seperated
  allows for rendering even when there's no stream

make channels and buttons bigger

add master meter to atomics
add cpu to atomics

allow only one instance of each view?
 makes some logic easier

add view resized callback
add view enter callback

add some way to tune instruments properly
 - can be done semi-automatically?

render in HDR and add bloom?

smooth meters
fix gain to be smoothed
mute declicking

should we use audio_thread_priority?

macos universal binary:
```yml
    - name: Build Release
      run: |
        rustup target add x86_64-apple-darwin
        cargo build --release --target x86_64-apple-darwin
        cargo build --release --target aarch64-apple-darwin
        lipo -create -output target/release/tessera \
          target/x86_64-apple-darwin/release/tessera \
          target/aarch64-apple-darwin/release/tessera
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

Note names are just uppercase A, B, C, etc.
other characters:
`()+,-./0123456789:<=>~`

accidentals:
a: single flat b
b: natural
c: single sharp #
d: double sharp x
e: double flat bb
f: triple flat bbb
g: 1/2 flat d
h: 1/2 sharp t
i: 3/2 sharp
j: caret down
k: caret up
l: small + (Johnston)
m: small - (Johnston)
n: septimal down L (heji)
o: septimal up (heji)
p: septimal double down (heji)
q: septimal double up (heji)
r: arrow up
s: arrow down
t: arrow double up
u: arrow double down

testing:
!: alternative triple flat (slanted)
": triple +
#: double caret down vv
$: double caret down stacked

unused:
`vwxyz%&'*;?@[]{}|^_`

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

