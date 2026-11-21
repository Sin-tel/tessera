// Prints every MIDI message from the input ports in the order they arrive, also to out/midi_dump.txt.
// One line per message: time in ms since start, raw bytes in hex, port, then a decoding.
//
// cargo run --example midi_dump -- --list                   list input ports
// cargo run --example midi_dump -- [name]                   record ports whose name contains `name` (all without it)
// cargo run --example midi_dump -- [name] --seconds 20
//
// Stop with Enter, or after `--seconds`. Close Tessera first: on Windows a port can only be open once.

use midir::{Ignore, MidiInput};
use std::fs::File;
use std::io::{Write, stdin};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const OUT: &str = "out/midi_dump.txt";

fn cc_name(cc: u8) -> &'static str {
	match cc {
		1 => " (mod wheel)",
		6 => " (data entry)",
		38 => " (data entry LSB)",
		64 => " (sustain)",
		74 => " (brightness)",
		100 => " (RPN LSB)",
		101 => " (RPN MSB)",
		_ => "",
	}
}

fn describe(data: &[u8]) -> String {
	let Some(&status) = data.first() else {
		return "empty".into();
	};
	let a = data.get(1).copied().unwrap_or(0);
	let b = data.get(2).copied().unwrap_or(0);
	let what = match status >> 4 {
		0x8 => format!("note off  {a:3}  vel {b}"),
		0x9 if b == 0 => format!("note off  {a:3}  (note on, vel 0)"),
		0x9 => format!("note on   {a:3}  vel {b}"),
		0xA => format!("poly aftertouch {a:3}  {b}"),
		0xB => format!("cc {a}{}  {b}", cc_name(a)),
		0xC => format!("program   {a}"),
		0xD => format!("pressure  {a}"),
		0xE => format!("pitchbend {:+}", i32::from(a) + i32::from(b) * 128 - 8192),
		_ => return "system".into(),
	};
	format!("ch {:2}  {what}", (status & 0x0f) + 1)
}

fn main() {
	let args: Vec<String> = std::env::args().skip(1).collect();
	let list = args.iter().any(|a| a == "--list");
	let mut seconds = None;
	let mut filter = None;
	let mut rest = args.iter();
	while let Some(a) = rest.next() {
		if a == "--seconds" {
			seconds = rest.next().and_then(|s| s.parse::<f64>().ok());
		} else if !a.starts_with("--") {
			filter = Some(a.to_lowercase());
		}
	}

	let probe = MidiInput::new("midi_dump").expect("could not start MIDI input");
	let names: Vec<String> = probe.ports().iter().filter_map(|p| probe.port_name(p).ok()).collect();

	if list || names.is_empty() {
		println!("MIDI input ports:");
		for name in &names {
			println!("  {name}");
		}
		if names.is_empty() {
			println!("  (none)");
		}
		return;
	}

	let chosen: Vec<&String> = names
		.iter()
		.filter(|n| filter.as_ref().is_none_or(|f| n.to_lowercase().contains(f)))
		.collect();
	if chosen.is_empty() {
		println!("No port matches {:?}. Use --list.", filter.unwrap_or_default());
		return;
	}

	std::fs::create_dir_all("out").unwrap();
	let file = Arc::new(Mutex::new(File::create(OUT).unwrap()));
	// Receive time, not the driver's timestamp: those start from a different origin per port.
	let start = Instant::now();

	let mut connections = Vec::new();
	for (i, name) in chosen.iter().enumerate() {
		let mut input = MidiInput::new("midi_dump").unwrap();
		// sysex, clock and active sensing
		input.ignore(Ignore::All);
		let Some(port) = input
			.ports()
			.into_iter()
			.find(|p| input.port_name(p).ok().as_ref() == Some(*name))
		else {
			println!("Port {name:?} disappeared");
			continue;
		};

		let file = file.clone();
		let connection = input.connect(
			&port,
			"midi_dump",
			move |_stamp, data, ()| {
				let t = start.elapsed().as_secs_f64() * 1000.0;
				let hex: Vec<String> = data.iter().map(|b| format!("{b:02X}")).collect();
				let line = format!("{t:10.3}  {:<8}  [{i}]  {}", hex.join(" "), describe(data));
				println!("{line}");
				writeln!(file.lock().unwrap(), "{line}").unwrap();
			},
			(),
		);
		match connection {
			Ok(c) => {
				println!("[{i}] {name}");
				connections.push(c);
			},
			Err(e) => println!("Could not open {name:?}: {e}"),
		}
	}
	if connections.is_empty() {
		return;
	}

	if let Some(s) = seconds {
		println!("Recording to {OUT} for {s} s.");
		std::thread::sleep(Duration::from_secs_f64(s));
	} else {
		println!("Recording to {OUT}, press Enter to stop.");
		stdin().read_line(&mut String::new()).unwrap();
	}
}
