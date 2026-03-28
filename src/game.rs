//! Implementation of parts of a Balatro game, core for these simulations.

use crate::cards::{
	Card,
	CardSet,
	Deck,
};

/// A basic simulation of a Balatro round, with only the cards part and one hand
/// (chance to play a set of cards).
///
/// The simulation includes every information a Balatro player has access
/// to: cards currently held in hand, discarded cards, and remaining cards in
/// the deck.
pub struct Game {
	/// Cards held in the hand.
	pub held: CardSet,

	/// The hand's capacity.
	pub capacity: usize,

	/// The deck to draw cards from.
	pub deck: Deck,

	/// The pile of cards that has been discarded.
	pub discard_pile: Vec<Card>,

	/// The number of discards left.
	pub discards: usize,

	/// The number of hands left.
	pub hands: usize,
}

impl Game {
	pub fn new(
		capacity: usize,
		deck: Deck,
		discards: usize,
		hands: usize,
	) -> Self {
		Self {
			held: CardSet(Vec::new()),
			capacity,
			deck,
			discard_pile: vec![],
			discards,
			hands,
		}
	}

	/// A default simulation of Balatro card drawing on White stake (the easiest
	/// difficulty): 4 hands and 3 discards are provided.
	pub fn white_stake_default() -> Self {
		Self::new(8, Deck::default(), 3, 4)
	}

	/// A default simulation of Balatro card drawing on Gold stake (the hardest
	/// difficulty): 2 discards are provided.
	pub fn gold_stake_default() -> Self {
		Self::new(8, Deck::default(), 2, 4)
	}
}
