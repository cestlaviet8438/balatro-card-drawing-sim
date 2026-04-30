//! Strategies for drawing flushes.

use std::collections::{
	HashMap,
	HashSet,
};

use malachite::{
	Natural,
	Rational,
};
use serde::{
	Deserialize,
	Serialize,
};

use crate::{
	cards::{
		Card,
		CardCollection,
		CardSet,
		Deck,
		Hand,
		SortCardsBy,
		Suit,
	},
	round::{
		Action,
		MAX_CARDS_SELECTABLE,
		Round,
	},
	strats::{
		Strategy,
		get_most_frequent_entries,
		hits_and_misses,
		n_choose_r,
	},
};

fn set_to_vec<T>(set: HashSet<T>) -> Vec<T> {
	set.into_iter().collect()
}

/// A [`Strategy`] that looks for flushes in the given 8-card hand and tries
/// to build one if there isn't.
#[derive(Clone)]
pub struct FavorFlushes;

impl FavorFlushes {
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
	/// cards on hand ensures that a diamond is created. Such a case is deemed
	/// to be an unreachable edge case as this simulation is only concerned with
	/// the most basic of setups (no Jokers, so nothing including Jokers like
	/// Merry Andy).
	fn get_target_suit(held: &CardSet, deck: &Deck) -> Suit {
		let held_suit_freqs = held.suit_frequencies();
		// look for most frequent suits in hand.
		let (best_held_suits, _freq_in_hand) =
			get_most_frequent_entries(&held_suit_freqs);
		if best_held_suits.len() == 1 {
			return set_to_vec(best_held_suits)[0];
		}

		// check in deck for most frequent suit. only suits that are already
		// held are checked.
		let deck_suit_freqs = deck
			.suit_frequencies()
			.into_iter()
			.filter(|(suit, _)| best_held_suits.contains(suit))
			.collect::<HashMap<_, _>>();
		let (best_suits_in_deck, _freq_in_deck) =
			get_most_frequent_entries(&deck_suit_freqs);

		match best_suits_in_deck.len() {
			// just return one of them if there is nothing left matching the
			// hand.
			0 => set_to_vec(best_held_suits)[0],
			1.. => set_to_vec(best_suits_in_deck)[0],
		}
	}

	fn _suited_drawing_sanity_check(
		draw_count: u64,
		suited_count: u64,
		deck: &Deck,
	) {
		debug_assert_ne!(draw_count, 0, "cannot draw 0 cards");
		debug_assert!(
			suited_count <= draw_count,
			"cannot demand more suited cards than are drawing"
		);
		debug_assert!(
			draw_count as usize <= MAX_CARDS_SELECTABLE,
			"cannot draw more than {MAX_CARDS_SELECTABLE} at a time"
		);
		debug_assert!(!deck.is_empty(), "cannot draw from an empty deck");
		debug_assert!(
			draw_count <= deck.len_u64(),
			"cannot draw more than available"
		);
	}

	/// Returns, out of those that can be constructed from the given [`Deck`],
	/// the number of [`Card`] sets of size `d` that contain **exactly** `s`
	/// cards of the given [`Suit`].
	fn _card_combinations_with_exact_suit_count(
		d: u64,
		s: u64,
		target_suit: Suit,
		deck: &Deck,
	) -> Natural {
		Self::_suited_drawing_sanity_check(d, s, deck);

		let deck_suit_freqs = deck.suit_frequencies();
		let deck_target_suit_count =
			*deck_suit_freqs.get(&target_suit).unwrap();
		let deck_non_target_suit_count = deck.len() - deck_target_suit_count;

		// draw set will be composed of `s` cards with the target suit and `d -
		// s` cards with other suits.
		let suited_combs =
			n_choose_r(deck_target_suit_count.try_into().unwrap(), s);
		let unsuited_combs =
			n_choose_r(deck_non_target_suit_count.try_into().unwrap(), d - s);
		suited_combs * unsuited_combs
	}

	/// Returns, out of those that can be constructed from the given [`Deck`],
	/// the number of [`Card`] sets of size `d` that contain *at least* `s`
	/// cards of the given [`Suit`].
	///
	/// In reality, this function calculates separately the number of
	/// combinations for [`Card`] sets of size `d` that contain between `s` and
	/// `d` cards of the given [`Suit`], before summing them together. This is
	/// based on the logic that since the sample space is composed of card sets
	/// that contain 0 suited cards, 1 suited card, etc. all the way to `d`
	/// suited cards, counting everything with >= `s` suited cards is sure to
	/// give the exact number.
	fn _card_combinations_with_at_least_suit_count(
		d: u64,
		s: u64,
		target_suit: Suit,
		deck: &Deck,
	) -> Natural {
		(s..=d)
			.map(|suited_count| {
				Self::_card_combinations_with_exact_suit_count(
					d,
					suited_count,
					target_suit,
					deck,
				)
			})
			.sum()
	}

	/// Returns the probability that a set of `d` [`Card`]s, drawn from a given
	/// [`Deck`]s, will contain **exactly** `s` cards with the given [`Suit`].
	fn _probability_to_draw_exactly_n_suited(
		d: u64,
		s: u64,
		target_suit: Suit,
		deck: &Deck,
	) -> Rational {
		Rational::from_naturals(
			Self::_card_combinations_with_exact_suit_count(
				d,
				s,
				target_suit,
				deck,
			),
			n_choose_r(deck.len_u64(), d),
		)
	}

	/// Returns the probability that a set of [`Card`]s, drawn from a given
	/// [`Deck`]s, will contain at least a given number of cards with the given
	/// [`Suit`].
	///
	/// This function uses the cumulative number obtained from
	/// [`Self::_card_combinations_with_at_least_suit_count`] for the ratio.
	/// This is based on the logic that since the sample space for drawing `d`
	/// random cards is composed of card sets that contain 0 suited cards, 1
	/// suited card, etc. all the way to `d` suited cards, counting everything
	/// with `s` suited cards or more is sure to account for all the draws that
	/// would satisfy the given conditions.
	///
	/// I don't know if I'll like combinatorics after this.
	fn _probability_to_draw_at_least_n_suited(
		draw_count: u64,
		suited_count: u64,
		target_suit: Suit,
		deck: &Deck,
	) -> Rational {
		Rational::from_naturals(
			Self::_card_combinations_with_at_least_suit_count(
				draw_count,
				suited_count,
				target_suit,
				deck,
			),
			n_choose_r(deck.len_u64(), draw_count),
		)
	}

	/// Returns if the next [`Action`] is a throw - either a discard, or a
	/// non-Flush play intending to look for more cards to complete a Flush.
	fn next_action_is_throw(&self, round: &Round) -> bool {
		match self.get_next_action(round) {
			Action::Discard => true,
			Action::Play => !self.get_hand_to_play(round).contains_flush(),
		}
	}

	/// Returns the probabilty of completing a Flush considering the current
	/// cards in hand and which cards will be discarded/how many cards will be
	/// drawn in their place.
	///
	/// This method assumes that the next action is a throw - a discard or a
	/// non-Flush play to search for more cards. [`None`] is returned if this
	/// not is the case, or if the round is already finished.
	fn probability_to_complete_flush(&self, round: &Round) -> Option<Rational> {
		let held = &round.held;
		let deck = &round.deck;

		if !self.next_action_is_throw(round) || round.is_finished() {
			return None;
		}

		const SIZE_OF_FLUSH: usize = 5; // no magic numbers allowed in this house
		let target_suit = Self::get_target_suit(held, deck);
		let held_suit_freqs = held.suit_frequencies();
		let held_target_suit_count = held_suit_freqs.get(&target_suit).unwrap();
		let cards_to_draw =
			(held.len() - held_target_suit_count).min(MAX_CARDS_SELECTABLE);
		let suited_cards_to_draw = SIZE_OF_FLUSH - held_target_suit_count;

		Some(Self::_probability_to_draw_at_least_n_suited(
			cards_to_draw.try_into().unwrap(),
			suited_cards_to_draw.try_into().unwrap(),
			target_suit,
			deck,
		))
	}
}

/// Strategy data for [`FavorFlushes`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavorFlushesData {
	/// Probability for this strategy to complete a Flush.
	probability_to_complete_flush: Option<Rational>,
}

impl Strategy for FavorFlushes {
	type StrategyData = FavorFlushesData;

	/// Returns the cards to discard. The first five cards (or less) held in
	/// hand that are not the target suit chosen by [`Self::get_target_suit`]
	/// are returned.
	fn get_hand_to_discard(&self, round: &Round) -> Hand {
		let target_suit = Self::get_target_suit(&round.held, &round.deck);
		round
			.held
			.iter()
			.filter(|card| card.1 != target_suit)
			.take(MAX_CARDS_SELECTABLE)
			.copied()
			.collect()
	}

	/// Returns the cards to play.
	///
	/// If there is a flush in hand, that flush is played (or parts of it).
	/// Otherwise, using [`Self::get_target_suit`], play away some cards to
	/// draw cards that might complete the flush in hand.
	fn get_hand_to_play(&self, round: &Round) -> Hand {
		let target_suit = Self::get_target_suit(&round.held, &round.deck);
		let (hits, misses) =
			hits_and_misses(round.held.iter(), |card| card.1 == target_suit);
		if round.held.contains_flush() {
			hits.into_iter()
				.take(MAX_CARDS_SELECTABLE)
				.copied()
				.collect()
		} else {
			misses
				.into_iter()
				.take(MAX_CARDS_SELECTABLE)
				.copied()
				.collect()
		}
	}

	/// Returns the next action.
	///
	/// If there is a flush in hand or if there is no discards left, play.
	/// Otherwise, discard.
	fn get_next_action(&self, round: &Round) -> Action {
		if round.held.contains_flush() || round.discards_remaining == 0 {
			Action::Play
		} else {
			Action::Discard
		}
	}

	/// Returns the preferred card sorting for to finish flushes - which is
	/// suits first.
	fn get_card_sort_strategy(&self) -> SortCardsBy {
		SortCardsBy::SuitsFirst
	}

	/// Returns the strategy data for this struct.
	fn get_strategy_data(&self, round: &Round) -> Self::StrategyData {
		FavorFlushesData {
			probability_to_complete_flush: todo!(),
		}
	}
}

#[cfg(test)]
mod test {
	use std::collections::{
		HashMap,
		HashSet,
	};

	use malachite::{
		Rational,
		base::num::conversion::traits::ToSci,
	};

	use crate::{
		cards::{
			CardSet,
			Deck,
			Hand,
			Suit,
		},
		round::{
			Action,
			Round,
			Stake,
		},
		strats::{
			Strategy,
			flush::{
				FavorFlushes,
				get_most_frequent_entries,
			},
			n_choose_r,
		},
	};

	#[test]
	fn probability_to_draw_suited_works() {
		let strategy = FavorFlushes;
		let mut round = Round::default_with_stake(Stake::White);
		round.draw_certain(&CardSet::from_iter([
			"ah", "2h", "3h", "4h", "ac", "2c", "as", "2s",
		]));

		assert_eq!(round.deck.len(), 44);

		assert_eq!(
			FavorFlushes::_card_combinations_with_exact_suit_count(
				1,
				1,
				Suit::Heart,
				&round.deck
			),
			9,
			"combinations for drawing 1 heart card"
		);
		assert_eq!(
			FavorFlushes::_probability_to_draw_exactly_n_suited(
				1,
				1,
				Suit::Heart,
				&round.deck
			),
			Rational::from_naturals(9u32.into(), 44u32.into()),
			"probability to draw 1 card and get exactly 1 heart"
		);

		assert_eq!(
			FavorFlushes::_card_combinations_with_exact_suit_count(
				4,
				1,
				Suit::Heart,
				&round.deck
			),
			n_choose_r(9, 1) * n_choose_r(35, 3),
			"combnations for drawing 4 cards with exactly 1 heart card"
		);
		assert_eq!(
			FavorFlushes::_probability_to_draw_exactly_n_suited(
				4,
				1,
				Suit::Heart,
				&round.deck
			),
			Rational::from_naturals(
				n_choose_r(9, 1) * n_choose_r(35, 3),
				n_choose_r(44, 4)
			),
			"probabilty to draw 4 cards with exactly 1 heart card"
		);

		assert_eq!(
			FavorFlushes::_probability_to_draw_exactly_n_suited(
				4,
				2,
				Suit::Heart,
				&round.deck
			),
			Rational::from_naturals(
				n_choose_r(9, 2) * n_choose_r(35, 2),
				n_choose_r(44, 4)
			),
			"probabilty to draw 4 cards with exactly 1 heart card"
		);

		assert_eq!(
			FavorFlushes::_probability_to_draw_at_least_n_suited(
				4,
				1,
				Suit::Heart,
				&round.deck
			),
			Rational::from_naturals(
				(1..=4)
					.map(|s| n_choose_r(9, s) * n_choose_r(35, 4 - s))
					.sum(),
				n_choose_r(44, 4)
			),
			"probabilty to draw 4 cards with exactly 1 heart card"
		);

		assert_eq!(
			strategy.probability_to_complete_flush(&round),
			Some(Rational::from_naturals(
				(1..=4)
					.map(|s| n_choose_r(9, s) * n_choose_r(35, 4 - s))
					.sum(),
				n_choose_r(44, 4)
			)),
			"probabilty to draw 4 cards with exactly 1 heart card"
		);

		// complete the flush so calculating the probability doesn't make sense
		round.draw_certain(&CardSet::from_iter(["5h"]));
		assert_eq!(
			strategy.probability_to_complete_flush(&round),
			None,
			"flush already completed; no probability to speak of"
		);
	}

	#[test]
	fn favor_flushes_strategy_works() {
		// in this case round is manually manipulated.
		let mut round = Round::default_with_stake(Stake::White);

		round.draw_certain(&CardSet::from_iter([
			"ah", "2h", "3h", "4h", "5s", "as", "ac", "ad",
		]));
		assert_eq!(
			FavorFlushes::get_target_suit(&round.held, &round.deck),
			Suit::Heart,
			"four hearts currently in hand"
		);
		assert_eq!(
			FavorFlushes.get_next_action(&round),
			Action::Discard,
			"flushes are not complete yet, so keep discarding"
		);
		assert_eq!(
			FavorFlushes.get_next_hand(&round),
			Hand::from_iter(["5s", "as", "ac", "ad"]),
			"discarding non-heart throw cards"
		);

		// make the strategy play throw cards instead of discard them
		round.discards_remaining = 0;
		assert_eq!(
			FavorFlushes.get_next_action(&round),
			Action::Play,
			"no discards left so has to play",
		);

		round.discard(&CardSet::from_iter(["ac", "ad"]));
		// now there is equal hearts and spades in deck.
		round.draw_certain(&CardSet::from_iter(["2s", "3s"]));
		assert!(
			[Suit::Heart, Suit::Spade].contains(
				&FavorFlushes::get_target_suit(&round.held, &round.deck)
			),
			"both hearts and spades are equally available"
		);

		// make spades less abundant in deck (take out king of spades from deck)
		round.deck.take_certain(&CardSet::from_iter(["ks"]));
		// ... so now it's better to target hearts instead
		assert_eq!(
			FavorFlushes::get_target_suit(&round.held, &round.deck),
			Suit::Heart,
			"spades are less abundant in deck so hearts take over"
		);
		assert_eq!(
			FavorFlushes.get_next_hand(&round),
			Hand::from_iter(["5s", "as", "2s", "3s"]),
			"trying to get more hearts so ridding spades"
		);

		// complete spade straight flush
		round.draw_certain(&CardSet::from_iter(["4s"]));
		assert_eq!(
			FavorFlushes::get_target_suit(&round.held, &round.deck),
			Suit::Spade,
			"spade straight flush currently in hand"
		);
		assert_eq!(
			FavorFlushes.get_next_hand(&round),
			Hand::from_iter(["as", "2s", "3s", "4s", "5s"]),
			"playing spade straight flush in hand"
		);
	}
}
