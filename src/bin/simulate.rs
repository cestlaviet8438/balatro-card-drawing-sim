use std::{
	error::Error,
	fs::{
		OpenOptions,
		create_dir,
	},
	io::Write,
	path::PathBuf,
	sync::atomic::AtomicUsize,
};

use balatro_card_drawing_sim::{
	round::{
		Round,
		Stake,
	},
	sim::Simulation,
	strats::{
		Strategy,
		flush::FavorFlushes,
	},
};
use lazy_static::lazy_static;

lazy_static! {
	static ref SIMS_COMPLETED: AtomicUsize = AtomicUsize::new(0);
}

/// Run `n` simulations, returning the list of resulting [`Round`]s.
fn run_n_sims<S: Strategy + Clone + 'static>(
	n: usize,
	stake: Stake,
	strategy: S,
) -> Vec<Round> {
	(0..n)
		.map(|_| {
			let mut sim = Simulation::new(
				Round::default_with_stake(stake),
				strategy.clone(),
			);
			sim.run();
			sim.round
		})
		.collect()
}

fn main() -> Result<(), Box<dyn Error>> {
	let output_folder = PathBuf::from("./output/");
	let output_file = output_folder.join("data.json");
	let _ = create_dir(&output_folder);
	let mut file = OpenOptions::new()
		.write(true)
		.truncate(true)
		.create(true)
		.open(output_file)?;
	let data = run_n_sims(100_000, Stake::White, FavorFlushes);
	file.write_all(serde_json::to_string(&data)?.as_bytes())?;
	Ok(())
}
