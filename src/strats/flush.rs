//! Strategies for drawing flushes.

use std::collections::{
	HashMap,
	HashSet,
};

use crate::{
	cards::{
		Card,
		CardCollection,
		CardSet,
		Deck,
		Hand,
		Suit,
	},
	round::{
		Action,
		Round,
	},
	strats::{
		Strategy,
		hits_and_misses,
	},
};

/// A [Strategy] that looks for flushes in the given 8-card hand and tries
/// to build one if there isn't.
pub struct FavorFlushes;

/// Utility to get the keys if their value is the maximum value in a certain
/// [`HashMap`]. This function returns the maximum value in a set of
/// values, and a list of keys that correspond to that maximum value.
fn get_most_frequent_entries<K, V>(map: &HashMap<K, V>) -> (HashSet<K>, V)
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

impl FavorFlushes {
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
	fn get_target_suit(held: &CardSet, deck: &Deck) -> Suit {
		let hand_suit_freqs = held.suit_frequencies();
		// look for most frequent suits in hand.
		let (best_held_suits, _freq_in_hand) =
			get_most_frequent_entries(&hand_suit_freqs);
		if best_held_suits.len() == 1 {
			return set_to_vec(best_held_suits)[0];
		}

		// check in deck for most frequent suit. only suits that are already
		// held are checked.
		let deck_suit_freqs = deck
			.suit_frequencies()
			.into_iter()
			.filter(|(suit, _)| best_held_suits.contains(suit))
			.collect::<HashMap<_, _>>();
		let (best_suits_in_deck, _freq_in_deck) =
			get_most_frequent_entries(&deck_suit_freqs);

		match best_suits_in_deck.len() {
			// just return one of them if there is nothing left matching the
			// hand.
			0 => set_to_vec(best_held_suits)[0],
			1.. => set_to_vec(best_suits_in_deck)[0],
		}
	}
}

impl Strategy for FavorFlushes {
	/// Returns the cards to discard. The first five cards (or less) held in
	/// hand that are not the target suit chosen by [`Self::get_target_suit`]
	/// are returned.
	fn get_hand_to_discard(&self, round: &Round) -> Hand {
		let target_suit = Self::get_target_suit(&round.held, &round.deck);
		round
			.held
			.iter()
			.filter(|card| card.1 != target_suit)
			.take(5)
			.copied()
			.collect()
	}

	/// Returns the cards to play.
	///
	/// If there is a flush in hand, that flush is played (or parts of it).
	/// Otherwise, using [`Self::get_target_suit`], play away some cards to
	/// draw cards that might complete the flush in hand.
	fn get_hand_to_play(&self, round: &Round) -> Hand {
		let target_suit = Self::get_target_suit(&round.held, &round.deck);
		let (hits, misses) =
			hits_and_misses(round.held.iter(), |card| card.1 == target_suit);
		if round.held.contains_flush() {
			hits.into_iter().take(5).copied().collect()
		} else {
			misses.into_iter().take(5).copied().collect()
		}
	}

	/// Returns the next action.
	///
	/// If there is a flush in hand or if there is no discards left, play.
	/// Otherwise, discard.
	fn get_next_action(&self, round: &Round) -> Action {
		if round.held.contains_flush() || round.discards_remaining == 0 {
			Action::Play
		} else {
			Action::Discard
		}
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
			Hand,
			Suit,
		},
		round::{
			Action,
			Round,
		},
		strats::{
			Strategy,
			flush::{
				FavorFlushes,
				get_most_frequent_entries,
			},
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
			get_most_frequent_entries(&data_1),
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
			get_most_frequent_entries(&data_2),
			(HashSet::from_iter([Suit::Heart, Suit::Diamond]), 10),
			"hearts and diamonds are equally common with 10 cards"
		);
	}

	#[test]
	fn favor_flushes_works() {
		// in this case round is manually manipulated.
		let mut round = Round::white_stake_default();

		round.draw_certain(&CardSet::from_iter([
			"ah", "2h", "3h", "4h", "5s", "as", "ac", "ad",
		]));
		assert_eq!(
			FavorFlushes::get_target_suit(&round.held, &round.deck),
			Suit::Heart,
			"four hearts currently in hand"
		);
		assert_eq!(
			FavorFlushes.get_next_action(&round),
			Action::Discard,
			"flushes are not complete yet, so keep discarding"
		);
		assert_eq!(
			FavorFlushes.get_next_hand(&round),
			Hand::from_iter(["5s", "as", "ac", "ad"]),
			"discarding non-heart trash cards"
		);

		// make the strategy play trash cards instead of discard them
		round.discards_remaining = 0;
		assert_eq!(
			FavorFlushes.get_next_action(&round),
			Action::Play,
			"no discards left so has to play",
		);

		round.discard(&CardSet::from_iter(["ac", "ad"]));
		// now there is equal hearts and spades in deck.
		round.draw_certain(&CardSet::from_iter(["2s", "3s"]));
		assert!(
			[Suit::Heart, Suit::Spade].contains(
				&FavorFlushes::get_target_suit(&round.held, &round.deck)
			),
			"both hearts and spades are equally available"
		);

		// make spades less abundant in deck (take out king of spades from deck)
		round.deck.take_certain(&CardSet::from_iter(["ks"]));
		// ... so now it's better to target hearts instead
		assert_eq!(
			FavorFlushes::get_target_suit(&round.held, &round.deck),
			Suit::Heart,
			"spades are less abundant in deck so hearts take over"
		);
		assert_eq!(
			FavorFlushes.get_next_hand(&round),
			Hand::from_iter(["5s", "as", "2s", "3s"]),
			"trying to get more hearts so ridding spades"
		);

		// complete spade straight flush
		round.draw_certain(&CardSet::from_iter(["4s"]));
		assert_eq!(
			FavorFlushes::get_target_suit(&round.held, &round.deck),
			Suit::Spade,
			"spade straight flush currently in hand"
		);
		assert_eq!(
			FavorFlushes.get_next_hand(&round),
			Hand::from_iter(["as", "2s", "3s", "4s", "5s"]),
			"playing spade straight flush in hand"
		);
	}
}
