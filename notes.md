catch lua error handler
- save to log
- save backup

* make seperate midi handlers per device
* translate those into CV
* record internal representation onto timeline
    - pre voice alloc
    - do voice alloc while recording, do another voice alloc pass during playback
* add simple playback (no editing yet!)
* prepare for play

test pad and keyboard input should go through interface instead of directly sending to backend!

text ordering in dropdown??

remove getdimensions and getmouse from the view thingys
 -- just store it in the thing?

## piano roll

piano roll playback
 - there is NO automatable bpm/speed setting
 - everything happens on the canvas *as is*.
 - makes things easier, at the expense of some tricks you can do by automating BPM
 - will need robust time manipuation tools to cope
 - (maybe a global speed mult because thats easy)

 ## envelope

also make ADS envelope for FM modulator
 how should these interact with pressure?
make some kind of nice "universal" envelope that works well with both pressure and velocity inputs
"universal" envelope for non-mpe input?
previous work
 - reason friktion
 - expressive E noisy 2

## MIDI

add note off?

midi impl is now kind of stupidly polling, but we can get better latency if we do everything in rust directly in the backend
 - bit more work to get right though since that means more logic on rust side
 - this is good though, since eventually we want to move as much as possible to rust
 - for now, lets leave it like this!

signal flow
```
                   midi_device       --- channel -----
                        |           /                   \
raw midi -[parse]-> midi msg -> event ->[voice alloc]-> backend
                                   v        ^
                                record------|
```

## UNITS
make everythin in samples?
 -> also envelope stuff should be in seconds not ms

add noise to epiano? better impact model

support surge wavetable format?


## other notes

datorro reverb
https://ccrma.stanford.edu/~dattorro/EffectDesignPart1.pdf

https://deps.rs/repo/github/Sin-tel/tessera

make simper f32x4 bandpass for modal

static filter optimization?

use smoothstep for dry/wet
for crossfading (wet/dry and such)
https://signalsmith-audio.co.uk/writing/2021/cheap-energy-crossfade/
should use amplitude / energy preserving depending on context

debugger where you can type lua commands and monitor variables (pretty print tables, maybe even with a foldable UI)
(command palette?)

use scancode instead of keycode
custom keyboard shortcut config?

spiral fft (maybe even something like harmonic CQT)

correct LUFS loudness monitoring

tonewheel emulation
* (mod pitch by noise bandpassed at some division of pitch )
* motor synth uses 7 cycles / rotation
* slightly non-linear optical transfer function + lowpass filter (different a/r)

synth where osc is nonlinear system

experiment with noise types and shaping
  - grainy, pink, velvet etc

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


# Style
lua style:

  - classes: PascalCase
  - instances/tables: camelCase, index with :
  - methods/functions: camelCase
  - variables: snake_case
  - constants: SCREAMING_SNAKE_CASE
  - singletons: camelCase, index with :

Switch to common style for lua classes

```lua
local Class = {}

function Class:new(args)
    local new = {}
    setmetatable(new, self)
    self.__index = self

    return new
end

return Class
```

better way:
```lua
local Class = {}
Class.__index = Class
-- ClassMt = {__index = Class}

-- ! note . and not :
function Class.new(args)
    local self = setmetatable({}, Class)

    return self
end

return Class
```

## parameter macro
```rust
// expands match arms for parameters
//     match_parameter!(index; a = value, b = value, c = value)
// becomes:
//     match index {
//         0 => a = value,
//         1 => b = value,
//         2 => c = value,
//         _ => eprintln!("Parameter with index {} not found", index),
//     }
macro_rules! match_parameter {
    ($e:expr; $($rest:tt)*) => {
        match_parameter!{@(0; $e; $($rest)*,)}
    };

    (@($idx:expr; $e:expr; $val:expr, $($rest:tt)*) $($arms:tt)*) => {
        match_parameter!{
            @(1+$idx; $e; $($rest)*)
            $($arms)*
            x if x == $idx => $val,
        }
    };

    (@($idx:expr; $e:expr; $(,)?) $($arms:tt)* ) => {
        match $e {
            $($arms)*
            _ => eprintln!("Parameter with index {} not found", $idx)
        }
    };
}

pub(crate) use match_parameter;
```
