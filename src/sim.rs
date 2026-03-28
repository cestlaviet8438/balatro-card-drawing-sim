//! What this project is setting out to do.

use crate::{
	cards::PokerHand,
	game::Game,
	strats::Strategy,
};

#[derive(Debug, Clone, Copy, Hash)]
pub struct SimResults {
	target_hand: PokerHand,
	discards_needed: u8,
}

/// A simulation of drawing, discarding (and optionally playing) cards in
/// Balatro.
struct Simulation {
	/// The Balatro game this simulation is looking at.
	game: Game,

	/// The drawing & discarding strategy this simulation is using.
	strategy: Box<dyn Strategy>,
}

impl Simulation {
	pub fn new<S: Strategy + 'static>(game: Game, strategy: S) -> Self {
		Self {
			game,
			strategy: Box::new(strategy),
		}
	}

	pub fn run() -> SimResults {
		todo!()
	}
}
