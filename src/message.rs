// Message struct to pass to audio thread
// Should not contain any boxed values (for now)

#[derive(Debug)]
pub enum Message {
	CV(usize, CV),
	Add,
}

#[derive(Debug)]
pub struct CV {
	pub freq: f32,
	pub vol: f32,
}
