//! Strategies for drawing flushes.

use std::collections::HashMap;

use crate::{
	cards::{
		Card,
		CardCollection,
		Suit,
	},
	game::Game,
	strats::Strategy,
};

/// A [Strategy] that looks for flushes in the given 8-card hand.
///
/// If there is none, the suit with the most cards currently in hand
/// is the target suit for a [Flush], and every other card is discarded (or up
/// to 5 of them). This is repeated until no discards are left and/or a [Flush]
/// is created.
///
/// [Flush]: [PokerHand::Flush]
pub struct FinishFlushes;

impl FinishFlushes {
	/// Utility to get the keys if their value is the maximum value in a certain
	/// [`HashMap`]. This function returns the maximum value in a set of
	/// values, and a list of keys that correspond to that maximum value.
	fn get_max_entries<K, V>(map: &HashMap<K, V>) -> (Vec<K>, V)
	where
		K: Clone + Copy,
		V: Clone + Copy + PartialOrd + Ord,
	{
		assert!(!map.is_empty(), "no entries to get");
		let max = *map.values().max().unwrap();
		(
			map.iter()
				.filter_map(
					|(key, value)| {
						if max == *value { Some(*key) } else { None }
					},
				)
				.collect(),
			max,
		)
	}

	/// Gets the [`Suit`] that the strategy will try to finish a
	/// [`PokerHand::Flush`] for. Essentially, this function returns which suit
	/// is the most plentiful in the current held cards and in the available
	/// deck. The procedure for selection is follows:
	/// - Record the most plentiful suits currently held in hand.
	/// - Check in the deck to see which held suits is the most plentiful there.
	///   For example, holding 4 hearts and 4 diamonds in hand, if there are 9
	///   hearts left in deck but 8 or less diamonds, hearts is chosen as the
	///   target suit.
	/// - If there are multiple eligible suits, "the first one" is chosen
	///   effectively at random. For the purposes of this simulation, suit
	///   orders do not matter; in this scenario, however, whichever suit
	///   happened to come first when looking through the hand/deck will be
	///   returned.
	///
	/// This algorithm does neglect certain edge cases like having 3 hearts, 3
	/// spades, 3 clubs having 5 diamonds still in deck, where discarding any 5
	/// cards on hand ensures that a diamond is created.
	fn get_target_suit(game: &Game) -> Suit {
		let hand_suit_counts = &game.held.suit_counts();
		// look for most suits in hand.
		let (best_held_suits, _count_in_hand) =
			Self::get_max_entries(hand_suit_counts);
		if best_held_suits.len() == 1 {
			return best_held_suits[0];
		}

		// check in deck for most plentiful suit
		let deck_suit_counts = &game.deck.suit_counts();
		let (best_suits_in_deck, _count_in_deck) = Self::get_max_entries(
			&deck_suit_counts
				.iter()
				.filter(|(suit, _)| best_held_suits.contains(suit))
				.collect::<HashMap<_, _>>(),
		);

		match best_suits_in_deck.len() {
			// just return one of them if there is nothing left matching the
			// hand.
			0 => best_held_suits[0],
			1.. => *best_suits_in_deck[0],
		}
	}
}

impl Strategy for FinishFlushes {
	fn get_cards_to_discard(&self, _game: &Game) -> Vec<Card> {
		todo!()
	}
}

#[cfg(test)]
mod test {
	use crate::cards::Deck;

	fn standard_deck() -> Deck {
		Deck::default()
	}
}
