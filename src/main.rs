use balatro_card_drawing_sim::{
	round::Round,
	sim::Simulation,
	strats::flush::FavorFlushes,
};

fn main() {
	let mut sim = Simulation::new(Round::white_stake_default(), FavorFlushes);
	sim.run_interactive();
}
