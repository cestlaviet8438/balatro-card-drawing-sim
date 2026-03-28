//! Strategies for drawing cards, used in a simulation.

use crate::{
	cards::Card,
	game::Game,
};

mod flush;

/// A strategy to simulate a player, making decisions on the cards in their
/// hand.
pub trait Strategy {
	/// Returns which cards to discard.
	fn get_cards_to_discard(&self, game: &Game) -> Vec<Card>;
}
