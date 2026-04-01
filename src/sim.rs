//! What this project is setting out to do.

use std::io::{
	Write,
	stdin,
	stdout,
};

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
	/// Whether the simulation has started.
	pub(crate) started: bool,

	/// The Balatro round this simulation is looking at.
	pub(crate) round: Round,

	/// The drawing & discarding strategy this simulation is using.
	strategy: Box<dyn Strategy>,
}

impl Simulation {
	pub fn new<S: Strategy + 'static>(round: Round, strategy: S) -> Self {
		Self {
			started: false,
			round,
			strategy: Box::new(strategy),
		}
	}

	/// Begin the simulation. This is only used when intending to step through
	/// the round action by the action.
	pub fn begin(&mut self) {
		self.round.begin();
		self.started = true;
	}

	/// Step through one action in the round.
	pub fn act(&mut self) {
		assert!(
			self.started,
			"cannot act when the simulation has not started"
		);
		let (action, cards) = (
			self.strategy.get_next_action(&self.round),
			self.strategy.get_next_hand(&self.round),
		);
		self.round.act(action, cards);
	}

	/// Run the simulation, going through every step and action of the contained
	/// strategy until the game is finished.
	pub fn run(&mut self) -> SimResults {
		self.begin();
		while !self.round.is_finished() {
			self.act();
		}
		SimResults {}
	}

	/// Run the simulation interactively, waiting for user input before running
	/// every step/action until the game is finished.
	pub fn run_interactive(&mut self) {
		self.begin();
		println!("{}", self.round);
		while !self.round.is_finished() {
			let mut _input = String::new();
			let _ = stdout().flush();
			stdin().read_line(&mut _input);

			self.act();
			println!("{}\n", self.round);
		}
	}
}
