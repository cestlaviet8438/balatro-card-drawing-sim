//! Strategies for drawing cards, used in a simulation.

use crate::{
	cards::{
		Card,
		Hand,
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
}
