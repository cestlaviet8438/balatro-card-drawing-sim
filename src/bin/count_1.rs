use std::collections::HashMap;

use balatro_card_drawing_sim::{
	cards::{
		CardCollection,
		CardSet,
		Deck,
	},
	strats::get_most_frequent_entries,
};
use combinatorial::Combinations;

/// Returns the maximum suit frequency (MSF) for the given set of cards.
fn get_msf(card_set: CardSet) -> usize {
	get_most_frequent_entries(&card_set.suit_frequencies()).1
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
	let mut max_suit_freqs: HashMap<usize, usize> = HashMap::new();
	let mut total_initial_draws = 0;
	for initial_draw in Combinations::of_size(Deck::default().into_iter(), 8) {
		total_initial_draws += 1;
		max_suit_freqs
			.entry(get_msf(CardSet::from_iter(initial_draw)))
			.and_modify(|count| *count += 1)
			.or_insert(1);
		if total_initial_draws % 100000 == 0 {
			print_status(&max_suit_freqs, total_initial_draws);
		}
	}
	println!(
		"total initial draws of size 8: {}\nmax suit frequencies: {:?}",
		total_initial_draws, max_suit_freqs
	);
}
