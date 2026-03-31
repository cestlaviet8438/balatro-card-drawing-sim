//! Strategies for drawing cards, used in a simulation.

use std::collections::HashSet;

use crate::{
	cards::Card,
	round::{
		Action,
		Round,
	},
};

mod flush;

/// A strategy to simulate a player, making decisions on the cards in their
/// hand.
pub trait Strategy {
	/// Returns which cards to discard.
	fn get_cards_to_discard(&self, round: &Round) -> HashSet<Card>;

	/// Returns which cards to play.
	fn get_cards_to_play(&self, round: &Round) -> HashSet<Card>;

	/// Returns what action to take next.
	fn get_next_action(&self, round: &Round) -> Action;
}
