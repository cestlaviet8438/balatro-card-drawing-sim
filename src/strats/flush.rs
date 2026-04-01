//! Strategies for drawing flushes.

use std::collections::{
	HashMap,
	HashSet,
};

use crate::{
	cards::{
		Card,
		CardCollection,
		Suit,
	},
	round::{
		Action,
		Round,
	},
	strats::Strategy,
};

/// A [Strategy] that looks for flushes in the given 8-card hand.
///
/// If there is none, the suit with the most cards currently in hand
/// is the target suit for a [Flush], and every other card is discarded (or up
/// to 5 of them). This is repeated until no discards are left and/or a [Flush]
/// is created.
///
/// [Flush]: [PokerHand::Flush]
pub struct FinishFlushes;

/// Utility to get the keys if their value is the maximum value in a certain
/// [`HashMap`]. This function returns the maximum value in a set of
/// values, and a list of keys that correspond to that maximum value.
fn get_most_freq_entries<K, V>(map: &HashMap<K, V>) -> (HashSet<K>, V)
where
	K: Clone + Copy + PartialEq + Eq + std::hash::Hash,
	V: Clone + Copy + PartialOrd + Ord,
{
	debug_assert!(!map.is_empty(), "no entries to get");
	let max = *map.values().max().unwrap();
	(
		map.iter()
			.filter_map(
				|(key, value)| {
					if max == *value { Some(*key) } else { None }
				},
			)
			.collect(),
		max,
	)
}

fn set_to_vec<T>(set: HashSet<T>) -> Vec<T> {
	set.into_iter().collect()
}

impl FinishFlushes {
	/// Gets the [`Suit`] that the strategy will try to finish a
	/// [`PokerHand::Flush`] for. Essentially, this function returns which suit
	/// is the most plentiful in the current held cards and in the available
	/// deck. The procedure for selection is follows:
	/// - Record the most plentiful suits currently held in hand.
	/// - Check in the deck to see which held suits is the most plentiful there.
	///   For example, holding 4 hearts and 4 diamonds in hand, if there are 9
	///   hearts left in deck but 8 or less diamonds, hearts is chosen as the
	///   target suit.
	/// - If there are multiple eligible suits, "the first one" is chosen
	///   effectively at random. For the purposes of this simulation, suit
	///   orders do not matter; in this scenario, however, whichever suit
	///   happened to come first when looking through the hand/deck will be
	///   returned.
	///
	/// This algorithm does neglect certain edge cases like having 3 hearts, 3
	/// spades, 3 clubs having 5 diamonds still in deck, where discarding any 5
	/// cards on hand ensures that a diamond is created.
	fn get_target_suit(round: &Round) -> Suit {
		let hand_suit_freqs = &round.held.suit_frequencies();
		// look for most suits in hand.
		let (best_held_suits, _freq_in_hand) =
			get_most_freq_entries(hand_suit_freqs);
		if best_held_suits.len() == 1 {
			return set_to_vec(best_held_suits)[0];
		}

		// check in deck for most plentiful suit. only suits that are already
		// held are checked.
		let deck_suit_freqs = &round
			.deck
			.suit_frequencies()
			.into_iter()
			.filter(|(suit, _)| best_held_suits.contains(suit))
			.collect::<HashMap<_, _>>();
		let (best_suits_in_deck, _freq_in_deck) =
			get_most_freq_entries(deck_suit_freqs);

		match best_suits_in_deck.len() {
			// just return one of them if there is nothing left matching the
			// hand.
			0 => set_to_vec(best_held_suits)[0],
			1.. => set_to_vec(best_suits_in_deck)[0],
		}
	}
}

impl Strategy for FinishFlushes {
	fn get_cards_to_discard(&self, round: &Round) -> HashSet<Card> {
		todo!()
	}

	fn get_cards_to_play(&self, round: &Round) -> HashSet<Card> {
		todo!()
	}

	fn get_next_action(&self, round: &Round) -> Action {
		todo!()
	}
}

#[cfg(test)]
mod test {
	use std::collections::{
		HashMap,
		HashSet,
	};

	use crate::{
		cards::{
			CardSet,
			Deck,
			Suit,
		},
		round::Round,
		strats::flush::{
			FinishFlushes,
			get_most_freq_entries,
		},
	};

	fn standard_deck() -> Deck {
		Deck::default()
	}

	#[test]
	fn get_most_freq_entries_works() {
		let data_1 = HashMap::from_iter(vec![
			(Suit::Heart, 13),
			(Suit::Diamond, 10),
			(Suit::Spade, 9),
			(Suit::Club, 8),
		]);
		assert_eq!(
			get_most_freq_entries(&data_1),
			(HashSet::from_iter([(Suit::Heart)]), 13),
			"13 hearts is the most common so one entry is returned"
		);

		let data_2 = HashMap::from_iter(vec![
			(Suit::Heart, 10),
			(Suit::Diamond, 10),
			(Suit::Spade, 9),
			(Suit::Club, 8),
		]);
		assert_eq!(
			get_most_freq_entries(&data_2),
			(HashSet::from_iter([Suit::Heart, Suit::Diamond]), 10),
			"hearts and diamonds are equally common with 10 cards"
		);
	}

	#[test]
	fn works_with_already_flushed() {
		let mut round = Round::white_stake_default();
		round.draw_certain(&CardSet::from_iter([
			"ah", "2h", "3h", "4h", "5h", "as", "ac", "ad",
		]));
		assert_eq!(
			FinishFlushes::get_target_suit(&round),
			Suit::Heart,
			"ace to five stragiht flush already present so target is heart"
		);
	}
}
