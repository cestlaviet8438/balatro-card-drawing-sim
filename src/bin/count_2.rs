use std::collections::HashMap;

use balatro_card_drawing_sim::{
	cards::{
		CardCollection,
		CardSet,
		Suit,
	},
	round::{
		Round,
		Stake,
	},
};
use combinatorial::Combinations;

/// Returns the maximum suit frequency (MSF) for the given set of cards.
fn get_suit_freq(card_set: CardSet, suit: Suit) -> usize {
	*card_set.suit_frequencies().get(&suit).unwrap_or(&0)
}

fn print_status(msfs: &HashMap<usize, usize>, total_entries: usize) {
	for msf in 2..=8 {
		if let Some(freq) = msfs.get(&msf) {
			let probability = *freq as f64 / total_entries as f64;
			println!(
				"frequency of msf {msf}: {freq} / {total_entries} = \
				 {probability:.6}"
			);
		}
	}
	println!();
}

fn main() {
	let mut round = Round::default_with_stake(Stake::White);
	round.draw_certain(&CardSet::from_iter([
		"ah", "2h", "3h", "4h", "ac", "ad", "as", "2c",
	]));

	let mut heart_freqs: HashMap<usize, usize> = HashMap::new();
	let mut total_draws = 0;
	for initial_draw in Combinations::of_size(round.deck.clone().into_iter(), 4)
	{
		total_draws += 1;
		heart_freqs
			.entry(get_suit_freq(CardSet::from_iter(initial_draw), Suit::Heart))
			.and_modify(|count| *count += 1)
			.or_insert(1);
	}
	println!(
		"total draws of size 4: {}\nhearts frequencies: {:?}",
		total_draws, heart_freqs
	);
}
