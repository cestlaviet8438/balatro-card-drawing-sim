//! Strategies for drawing cards, used in a simulation.

use std::{
	collections::{
		HashMap,
		HashSet,
	},
	fmt::Display,
	hash::Hash,
};

use cached::proc_macro::cached;
use malachite::{
	Natural,
	Rational,
	base::num::{
		arithmetic::traits::Factorial,
		conversion::traits::ToSci,
	},
};
use serde::{
	Deserialize,
	Serialize,
};

use crate::{
	cards::{
		Card,
		Hand,
		SortCardsBy,
	},
	round::{
		Action,
		Round,
	},
};

pub mod flush;

/// A more complicated version of `filter`. Based on the predicate,
/// returns two separate collections using the provided collection, in the order
/// of the returned tuple: one the predicate returns `true` for, and one the
/// predicate returns `false` for.
pub fn hits_and_misses<'a, I, C, P>(
	collection: C,
	predicate: P,
) -> (Vec<&'a I>, Vec<&'a I>)
where
	C: Iterator<Item = &'a I>,
	P: Fn(&'a I) -> bool,
{
	let mut hits = vec![];
	let mut misses = vec![];
	for item in collection {
		match predicate(item) {
			true => hits.push(item),
			false => misses.push(item),
		}
	}
	(hits, misses)
}

/// Utility to get the keys if their value is the maximum value in a certain
/// [`HashMap`]. This function returns the maximum value in a set of
/// values, and a list of keys that correspond to that maximum value.
pub fn get_most_frequent_entries<K, V>(map: &HashMap<K, V>) -> (HashSet<K>, V)
where
	K: Eq + Hash + Clone,
	V: Ord + Clone,
{
	debug_assert!(!map.is_empty(), "no entries to get");
	let max = map.values().max().unwrap();
	(
		map.iter()
			.filter_map(|(key, value)| {
				if *max == *value {
					Some(key.clone())
				} else {
					None
				}
			})
			.collect(),
		max.clone(),
	)
}

/// Data associated with the [`Strategy`]. This struct may contain
/// anything, for example valuable insights the strategy is making
/// based on the state of the round.
///
/// p.s. i gave up on trying to make this generic lmao
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyData {
	/// The probability that a [`crate::cards::PokerHand`]-oriented
	/// strategy can complete the hand with the next [`Action`].
	probability_to_complete_hand: Option<Rational>,
}

impl Display for StrategyData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "probability to complete hand: {}", match &self
			.probability_to_complete_hand
		{
			Some(ratio) => ratio.to_sci().to_string(),
			None => "n/a".into(),
		})
	}
}

/// A strategy to simulate a player, making decisions on the cards in their
/// hand.
pub trait Strategy {
	/// Returns which cards to discard.
	fn get_hand_to_discard(&self, round: &Round) -> Hand;

	/// Returns which cards to play.
	fn get_hand_to_play(&self, round: &Round) -> Hand;

	/// Returns what action to take next.
	fn get_next_action(&self, round: &Round) -> Action;

	/// Returns the cards to be used in the next action.
	fn get_next_hand(&self, round: &Round) -> Hand {
		match self.get_next_action(round) {
			Action::Discard => self.get_hand_to_discard(round),
			Action::Play => self.get_hand_to_play(round),
		}
	}

	/// Runs the next action for the round, based on the strategy.
	fn act(&self, round: &mut Round) {
		round.act(self.get_next_action(round), self.get_next_hand(round));
	}

	/// Obtains the simulation-relevant data for this strategy.
	fn get_strategy_data(&self, round: &Round) -> StrategyData;

	/// Gets the preferred method to sort cards by when formatting and printing.
	/// This function is run for every turn in a round.
	fn get_card_sort_strategy(&self) -> SortCardsBy;
}

#[cfg(test)]
mod test {
	use std::collections::{
		HashMap,
		HashSet,
	};

	use crate::{
		cards::Suit,
		strats::get_most_frequent_entries,
	};

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
}
