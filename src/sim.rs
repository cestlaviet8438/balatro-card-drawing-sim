//! What this project is setting out to do.

use crate::{
	cards::PokerHand,
	round::Round,
	strats::Strategy,
};

#[derive(Debug, Clone, Copy, Hash)]
pub struct SimResults {}

/// A simulation of drawing, discarding (and optionally playing) cards in
/// Balatro.
pub struct Simulation {
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

	/// Run the simulation, going through every step and action of the contained
	/// strategy until the game is finished.
	pub fn run(&mut self) -> SimResults {
		self.round.begin();
		while !self.round.is_finished() {
			let (action, cards) = (
				self.strategy.get_next_action(&self.round),
				self.strategy.get_next_hand(&self.round),
			);
			self.round.act(action, &cards);
		}
		todo!()
	}
}
