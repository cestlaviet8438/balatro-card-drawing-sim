//! What this project is setting out to do.

use std::io::{
	Write,
	stdin,
	stdout,
};

use serde::{
	Deserialize,
	Serialize,
};

use crate::{
	cards::{
		CardSet,
		Hand,
		PokerHand,
	},
	round::{
		Action,
		Round,
	},
	strats::Strategy,
};

/// The data relevant to a round, after having been run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundData {
	/// The hand's capacity.
	pub held_capacity: usize,

	/// The number of discards this round starts with.
	pub discards_given: usize,

	/// The number of discards left.
	pub discards_remaining: usize,

	/// The number of plays this round started with.
	pub plays_given: usize,

	/// The number of plays left.
	pub plays_remaining: usize,

	/// The hands that have been played.
	pub plays: Vec<Hand>,

	/// The history of actions taken during this round.
	pub history: Vec<(CardSet, Action, Hand)>,
}

impl From<&Round> for RoundData {
	fn from(round: &Round) -> Self {
		RoundData {
			held_capacity: round.held_capacity,
			discards_given: round.discards_given,
			discards_remaining: round.discards_remaining,
			plays_given: round.plays_given,
			plays_remaining: round.plays_remaining,
			plays: round.plays.clone(),
			history: round.history.clone(),
		}
	}
}

impl RoundData {
	/// Returns the number of discards used.
	pub fn discards_used(&self) -> usize {
		self.discards_given - self.discards_remaining
	}
}

/// A simulation of drawing, discarding (and optionally playing) cards in
/// Balatro.
pub struct Simulation {
	/// Whether the simulation has started.
	pub started: bool,

	/// The Balatro round this simulation is looking at.
	pub round: Round,

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

	/// Gets the resulting data from a round that has been run.
	pub fn get_round_data(&self) -> RoundData {
		(&self.round).into()
	}
}
