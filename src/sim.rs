//! What this project is setting out to do.

use std::io::{
	Write,
	stdin,
	stdout,
};

use crate::{
	cards::{
		Hand,
		PokerHand,
	},
	round::Round,
	strats::Strategy,
};

// #[derive(Debug, Clone)]
// pub struct SimResults {
// 	  pub(crate) original_discard_count: usize,
// 	  pub(crate) remaining_discards: usize,
// 	  pub(crate) played_hands: Vec<Hand>,
// }

// impl SimResults {
// 	/// Returns how many of a certain hand was played during this round.
// 	pub fn get_played_hand_count(&self, hand: PokerHand) -> usize {
// 		self.played_hands
// 			.iter()
// 			.filter(|play| play.is_poker_hand(hand))
// 			.count()
// 	}
// }

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
	pub fn step(&mut self) {
		assert!(
			self.started,
			"cannot act when the simulation has not started"
		);
		self.strategy.act(&mut self.round);
	}

	/// Run the simulation, going through every step and action of the contained
	/// strategy until the game is finished.
	pub fn run(&mut self) {
		self.begin();
		while !self.round.is_finished() {
			self.step();
		}
	}

	/// Prints the status of the round.
	fn print_round_status(&self) {
		println!(
			"{}",
			self.round
				.fmt_status(self.strategy.get_card_sort_strategy())
		);
	}

	/// Run the simulation interactively, waiting for user input before running
	/// every step/action until the game is finished.
	pub fn run_interactive(&mut self) {
		self.begin();
		self.print_round_status();
		while !self.round.is_finished() {
			let _ = stdout().flush();
			stdin().read_line(&mut String::new());
			self.step();
			self.print_round_status();
		}
	}
}
