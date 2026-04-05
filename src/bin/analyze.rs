use std::{
	collections::HashMap,
	error::Error,
	fmt::Display,
	fs::OpenOptions,
	hash::Hash,
	io::{
		Read,
		Write,
	},
	path::PathBuf,
};

use balatro_card_drawing_sim::sim::RoundData;
use json::{
	JsonValue,
	array,
	object,
};
use serde::{
	Deserialize,
	Serialize,
};

/// Maps each unique value in a [`HashMap`] to its frequency.
pub fn get_value_frequencies<K, V>(map: &HashMap<K, V>) -> HashMap<V, usize>
where
	K: Eq + Hash,
	V: Eq + Hash + Clone,
{
	map.iter().fold(HashMap::new(), |mut acc, (_key, value)| {
		acc.entry(value.clone())
			.and_modify(|freq| *freq += 1)
			.or_insert(1);
		acc
	})
}

/// Represents a quantity that is a part of something larger.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Part {
	part: usize,
	whole: usize,
}

impl From<Part> for JsonValue {
	fn from(part: Part) -> Self {
		array![part.part, part.whole]
	}
}

impl Display for Part {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} ({})", self.number(), self.fractional())
	}
}

impl Part {
	/// Constructs a new partial quantity.
	pub fn new(part: usize, whole: usize) -> Self {
		debug_assert!(
			part <= whole,
			"part cannot be larger than whole. received: part: {part}, whole \
			 {whole}"
		);
		Self { part, whole }
	}

	/// Returns the number quantity of the part.
	pub fn number(&self) -> usize {
		self.part
	}

	/// Returns the quantity as a fraction/percentage of the whole.
	pub fn fractional(&self) -> f64 {
		(self.part as f64) / (self.whole as f64)
	}
}

/// Returns the number of flushes played during the round.
/// Note that straight flushes are not excluded.
fn get_flushes_played(round: &RoundData) -> usize {
	round
		.plays
		.iter()
		.filter(|hand| hand.contains_flush())
		.count()
}

/// Returns the number of discards and throwaway hands ("throws") used during
/// the round.
fn get_throws_used(round: &RoundData) -> usize {
	// 🤜
	let throw_hands = round
		.plays
		.iter()
		.filter(|hand| !hand.contains_flush())
		.count();
	throw_hands + round.discards_used()
}

/// Returns when the first flush was acquired in the hand.
/// The number returned is how many actions was done until the first time held
/// cards contained a flush - if a flush was never acquired, [`None`] is
/// returned.
fn get_actions_to_first_flush(round: &RoundData) -> Option<usize> {
	round
		.history
		.iter()
		.enumerate()
		.find(|(_, (held, ..))| held.contains_flush())
		.map(|(index, _)| index)
}

/// Compiles the value of a metirc for every round into the frequencies for a
/// complete set of rounds played. The returned [`HashMap`] maps a certain value
/// of the metric to how many rounds shared that value.
fn metric_frequencies<F, M>(
	rounds: &[RoundData],
	get_metric: F,
) -> HashMap<M, Part>
where
	F: Fn(&RoundData) -> M,
	M: Eq + Hash + Clone,
{
	get_value_frequencies(
		&rounds
			.iter()
			.enumerate()
			.map(|(index, round)| (index, get_metric(round)))
			.collect(),
	)
	.iter()
	.map(|(metric, freq)| (metric.clone(), Part::new(*freq, rounds.len())))
	.collect()
}

/// Converst [`HashMap`] keys to [`String`]s.
fn stringify_keys<K: ToString, V>(map: HashMap<K, V>) -> HashMap<String, V> {
	map.into_iter()
		.map(|(key, value)| (key.to_string(), value))
		.collect()
}

fn main() -> Result<(), Box<dyn Error>> {
	let output_folder = PathBuf::from("./output");
	let data_file_path = output_folder.join("data.json");
	let mut data_file = OpenOptions::new().read(true).open(&data_file_path)?;
	let mut data_str = String::new();
	data_file.read_to_string(&mut data_str)?;
	let analysis_file_path = output_folder.join("analysis.json");
	let mut analysis_file = OpenOptions::new()
		.write(true)
		.truncate(true)
		.create(true)
		.open(analysis_file_path)?;

	let rounds: Vec<RoundData> = serde_json::from_str(&data_str)?;
	let flushes_played = metric_frequencies(&rounds, get_flushes_played);
	let throws_used = metric_frequencies(&rounds, get_throws_used);
	let actions_to_first_flush: HashMap<_, _> =
		metric_frequencies(&rounds, get_actions_to_first_flush)
			.into_iter()
			.map(|(key, value)| (key.map_or(-1, |k| k as isize), value))
			.collect();

	let analysis = object! {
		flushes_played: stringify_keys(flushes_played),
		throws_used: stringify_keys(throws_used),
		actions_to_first_flush: stringify_keys(actions_to_first_flush),
	};
	analysis_file.write_all(format!("{analysis:#}").as_bytes())?;

	Ok(())
}
