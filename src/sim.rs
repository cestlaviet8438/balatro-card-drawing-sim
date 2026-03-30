//! What this project is setting out to do.

use crate::{
	cards::PokerHand,
	round::Round,
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
	/// The Balatro round this simulation is looking at.
	round: Round,

	/// The drawing & discarding strategy this simulation is using.
	strategy: Box<dyn Strategy>,
}

impl Simulation {
	pub fn new<S: Strategy + 'static>(round: Round, strategy: S) -> Self {
		Self {
			round,
			strategy: Box::new(strategy),
		}
	}

	pub fn run() -> SimResults {
		todo!()
	}
}
