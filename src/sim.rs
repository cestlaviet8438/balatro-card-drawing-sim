//! What this project is setting out to do.

use std::io::{
	Write,
	stdin,
	stdout,
};

use derive_new::new;
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
		ActionData,
		Round,
		Stake,
	},
	strats::{
		Strategy,
		StrategyData,
	},
};

/// The data created by a simulation.
/// This contains everything relevant to a [`Round`] after having been run with
/// a certain [`Strategy`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationData {
	/// ID for the round.
	/// The ID is a character representing the stake, followed by a string of
	/// numbers.
	pub id: String,

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
	pub action_history: Vec<ActionData>,

	/// A history for strategy data.
	/// This history begins before any actions have been taken, and ends at the
	/// last round.
	pub strategy_history: Vec<StrategyData>,
}

impl SimulationData {
	/// Constructs a new [`RoundData`] with the given data.
	pub fn new(
		round: &Round,
		strategy_history: Vec<StrategyData>,
		stake: Stake,
		id: u64,
	) -> Self {
		debug_assert_eq!(
			round.history.len(),
			strategy_history.len(),
			"there must be data every for every action"
		);

		let id_prefix = match stake {
			Stake::White => "W",
			Stake::Gold => "G",
		};
		Self {
			id: format!("{id_prefix}{id}"),
			held_capacity: round.held_capacity,
			discards_given: round.discards_given,
			discards_remaining: round.discards_remaining,
			plays_given: round.plays_given,
			plays_remaining: round.plays_remaining,
			plays: round.plays.clone(),
			action_history: round.history.clone(),
			strategy_history,
		}
	}

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

	/// The data this strategy outputs every turn.
	strategy_history: Vec<StrategyData>,
}

impl Simulation {
	/// Constructs a new [`Simulation`].
	pub fn new<St>(round: Round, strategy: St) -> Self
	where
		St: Strategy + 'static,
	{
		Self {
			started: false,
			round,
			strategy: Box::new(strategy),
			strategy_history: vec![],
		}
	}

	/// Add one item to [`Self::strategy_history`].
	/// Essentially, this is asking the strategy to make an assessment
	/// and generate some data related to the initial state of the round
	/// (after it has begun).
	pub fn assess_round(&mut self) {
		self.strategy_history
			.push(self.strategy.get_strategy_data(&self.round));
	}

	/// Begin the simulation. This is only used when intending to step through
	/// the round action by the action.
	pub fn begin(&mut self) {
		self.round.begin();
		self.started = true;
		self.assess_round();
	}

	/// Step through one action in the round.
	pub fn step(&mut self) {
		assert!(
			self.started,
			"cannot act when the simulation has not started"
		);
		self.strategy.act(&mut self.round);
		if !self.round.is_finished() {
			self.assess_round();
		}
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
			"{}\n{}",
			self.round
				.fmt_status(self.strategy.get_card_sort_strategy()),
			match self.round.is_finished() {
				false => format!("{}", self.strategy_history.last().unwrap()),
				true => "".into(),
			},
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

	/// Consumes this simulation, returning the data from the round that has
	/// been run, supplemented by a [`Stake`] and an ID.
	pub fn to_round_data(self, stake: Stake, id: u64) -> SimulationData {
		SimulationData::new(&self.round, self.strategy_history, stake, id)
	}
}
