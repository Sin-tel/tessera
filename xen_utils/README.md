# xen_utils

Notation for regular temperaments.

This library handles only extended Pythagorean notation.
Notes are written on a chain of fifths (`F C G D A E B`, sharps and flats), and each prime beyond 3 gets its own pair of accidentals, in the spirit of Helmholtz-Ellis.
It supports just intonation (JI), regular temperaments and equal temperaments over prime subgroups.

## What it does

- **Temperaments.** Build a regular temperament over any prime subgroup: from the commas it tempers out, from a mapping matrix, or as an equal temperament.
- **Notations.** Derives available notations for any temperament. Accidentals can be automatically derived from the subgroup. The library then picks which of these are necessary, and runs a search to find the best notation systems. It tries to find notations that write every prime on its usual letter (i.e. a major third is written as third, not a diminished fourth).
- **Spelling.** Finds the best ways to write a tempered interval from the enharmonic intervals that the notation implies (e.g. `C## = D`). `spellings` returns a ranked list of ways to spell an interval under a cost function.
- **Simplification.** Finds the simplest just intervals a tempered interval stands for, ranked by the Wilson norm (sum of prime factors).
- **Tuning.** Computes the optimal tuning of a temperament, and the size in cents of any interval or written note.

Instead of relying on heuristics, all searches are implemented using exact lattice algorithms, built on the [diophantine](https://github.com/Sin-tel/diophantine) crate.

## Example

```rust
use xen_utils::{Notation, Simplifier, Subgroup, Temperament};

let subgroup: Subgroup = "2.3.5.7.11".parse()?;
let temperament = Temperament::equal(41, &subgroup)?;
let notation = Notation::from_temperament(&temperament)?;

// How to write 11/8, and its size in 41et.
let eleven = subgroup.parse_ratio("11/8")?;
let spelling = notation.spell_interval(&eleven)?;
notation.note(&spelling);    // "^^F5", written up from C5
notation.pitch(&spelling)?;  // 556.3 cents

// Every way to write that note, best first.
notation.spellings(&temperament.temper(&eleven)?, 3)?;  // ^^F5, vvF#5, vGb5

// What 81/64 is in 41et: the simplest just intervals it stands for.
let simplifier = Simplifier::new(&temperament);
simplifier.simplifications_interval(&subgroup.parse_ratio("81/64")?, 3)?;  // 14/11, 81/64, 63/50
```

The accidental symbols `^ v < > ...` are placeholders for debugging. Unicode has no real microtonal accidentals, glyph selection and rendering is left to the application.

## Examples

```
cargo run --example spellings        # every step of 12et and 41et, written three ways
cargo run --example simplifications  # the simplest readings of each step
cargo run --example notations        # all of the notations derived for data/temperaments.txt
cargo run --example tunings          # optimal tunings of the same list
```
