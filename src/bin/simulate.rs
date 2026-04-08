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
	sim::{
		RoundData,
		Simulation,
	},
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
	n: u64,
	stake: Stake,
	strategy: S,
) -> Vec<RoundData> {
	(0..n)
		.map(|id| {
			let mut sim = Simulation::new(
				Round::default_with_stake(stake),
				strategy.clone(),
			);
			sim.run();
			sim.get_round_data(stake, id + 1)
		})
		.collect()
}

/// Produces the data of `n` simulations on a particular [`Stake`] for a
/// certain [`Strategy`], writing it to an appropriate file.
fn produce_data_for_stake<S: Strategy + Clone + 'static>(
	n: u64,
	stake: Stake,
	strategy: S,
	output_file_name: String,
) -> Result<(), Box<dyn Error>> {
	let output_folder = PathBuf::from("./output/");
	let output_file = output_folder.join(output_file_name);
	let _ = create_dir(&output_folder);
	let mut file = OpenOptions::new()
		.write(true)
		.truncate(true)
		.create(true)
		.open(output_file)?;
	let data = run_n_sims(n, stake, strategy);
	file.write_all(serde_json::to_string(&data)?.as_bytes())?;
	Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
	const TRIALS: u64 = 1_000_000;

	println!("producing data for white stake...");
	produce_data_for_stake(
		TRIALS,
		Stake::White,
		FavorFlushes,
		"white_stake_data.json".into(),
	)?;

	println!("producing data for gold stake...");
	produce_data_for_stake(
		TRIALS,
		Stake::Gold,
		FavorFlushes,
		"gold_stake_data.json".into(),
	)?;

	Ok(())
}
