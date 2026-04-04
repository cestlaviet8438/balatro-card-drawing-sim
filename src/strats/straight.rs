use crate::{
	cards::{
		CardCollection,
		CardSet,
		Deck,
		Rank,
	},
	round::{
		Action,
		Round,
		SortCardsBy,
	},
	strats::Strategy,
};

/// Represents a straight in Poker.
struct Straight {
	/// The lowest [`Rank`] where the straight starts.
	start: Rank,

	/// The highest [`Rank`] where the straight ends.
	end: Rank,
}

/// A [`Strategy`] that looks for straights in the given 8-card hand and tries
/// to build one if there isn't.
pub struct SeekStraights;

impl SeekStraights {
	fn get_target_straight(held: &CardSet, deck: &Deck) -> Straight {
		let held_ranks = held.rank_set();
		todo!()
	}
}

impl Strategy for SeekStraights {
	fn get_hand_to_discard(
		&self,
		round: &crate::round::Round,
	) -> crate::cards::Hand {
		todo!()
	}

	fn get_hand_to_play(
		&self,
		round: &crate::round::Round,
	) -> crate::cards::Hand {
		todo!()
	}

	/// Returns the next action.
	///
	/// If there is a flush in hand or if there is no discards left, play.
	/// Otherwise, discard.
	fn get_next_action(&self, round: &Round) -> Action {
		if round.held.contains_straight() || round.discards_remaining == 0 {
			Action::Play
		} else {
			Action::Discard
		}
	}

	/// Gets the card sorting strategy for straights - ranks first.
	fn get_card_sort_strategy(&self) -> SortCardsBy {
		SortCardsBy::RanksFirst
	}
}
