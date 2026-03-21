use crate::cards::{
	Card,
	Pack,
};

/// A strategy to simulate a player, making decisions on the cards in their
/// hand.
pub trait Strategy {
	/// Returns which cards to discard.
	fn get_cards_to_discard(simulation: &Game) -> Vec<Card>;
}

/// A basic simulation of a Balatro round, with only the cards part and one hand
/// (chance to play a set of cards).
///
/// The simulation includes every information a Balatro player has access
/// to.
pub struct Game {
	/// Cards held in the hand.
	held: Vec<Card>,

	/// The hand's capacity.
	capacity: usize,

	/// The pack to draw cards from.
	pack: Pack,

	/// The pile of cards that has been discarded.
	discard_pile: Vec<Card>,

	/// The number of discards left.
	discards: usize,

	/// The number of hands left.
	hands: usize,
}

impl Game {
	pub fn new(
		capacity: usize,
		pack: Pack,
		discards: usize,
		hands: usize,
	) -> Self {
		Self {
			held: vec![],
			capacity,
			pack,
			discard_pile: vec![],
			discards,
			hands,
		}
	}

	/// A default simulation of Balatro card drawing on White stake (the easiest
	/// difficulty): 4 hands and 3 discards are provided.
	pub fn white_stake_default() -> Self {
		Self::new(8, Pack::default(), 3, 4)
	}

	/// A default simulation of Balatro card drawing on Gold stake (the hardest
	/// difficulty): 2 discards are provided.
	pub fn gold_stake_default() -> Self {
		Self::new(8, Pack::default(), 2, 4)
	}
}
