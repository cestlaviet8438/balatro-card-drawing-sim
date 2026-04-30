use balatro_card_drawing_sim::{
	round::{
		Round,
		Stake,
	},
	sim::Simulation,
	strats::flush::FavorFlushes,
};

fn main() {
	let mut simulation =
		Simulation::new(Round::default_with_stake(Stake::White), FavorFlushes);
	simulation.run_interactive();
}
