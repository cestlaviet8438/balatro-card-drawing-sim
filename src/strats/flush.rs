//! Strategies for drawing flushes.

use std::collections::{
	HashMap,
	HashSet,
};

use cached::proc_macro::cached;
use enum_iterator::all;
use malachite::{
	Natural,
	Rational,
	base::num::arithmetic::traits::Factorial,
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
		StrategyData,
		get_most_frequent_entries,
		hits_and_misses,
	},
};

fn set_to_vec<T>(set: HashSet<T>) -> Vec<T> {
	set.into_iter().collect()
}

/// Evalutes `n!`, i.e. n factorial.
#[cached]
pub fn factorial(n: u64) -> Natural {
	Natural::factorial(n)
}

/// Evalutes `nCr(n, r)`, i.e. how many combinations of size `r` can be made
/// from `n` distinct symbols.
///
/// This function is a translation of the formula `nCr = n! / r!(n-r)!`.
#[cached]
pub fn n_choose_k(n: u64, r: u64) -> Natural {
	debug_assert!(
		r <= n,
		"cannot choose more than available: {r} was not <= {n}"
	);
	match (n, r) {
		(0, _) => 0u64.into(),
		(_, 0) => 1u64.into(),
		(n, r) if n == r => 1u64.into(),
		_ => factorial(n) / (factorial(r) * factorial(n - r)),
	}
}

const SIZE_OF_FLUSH: usize = 5; // no magic numbers allowed in this house

/// A [`Strategy`] that looks for flushes in the given 8-card hand and tries
/// to build one if there isn't.
#[derive(Clone)]
pub struct FavorFlushes;

impl FavorFlushes {
	/// Gets the [`Suit`] that the strategy will try to finish a
	/// [`PokerHand::Flush`] for. Essentially, this function returns which suit
	/// is the most plentiful in the current held cards and in the available
	/// deck. The procedure for selection is follows:
	/// - If there already is a Flush in hand,
	/// - Record the suits currently held in hand. Only the suits that have
	///   enough cards in the deck to make a Flush with is considered.
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
	/// This algorithm is geared towards the first few turns of a round. Some
	/// edge cases are neglected, for example: holding 4 hearts, 1 clubs, 1
	/// spades, and 2 diamonds while there is 1 heart card and 10 diamond
	/// cards. Here it is clearly better to draw the diamonds, but this
	/// function will nonetheless evaluate to hearts for simplicity - held
	/// cards take priority over everything else.
	fn get_target_suit(held: &CardSet, deck: &Deck) -> Suit {
		// check in held card for most frequent suit.
		let held_suit_freqs = held.suit_frequencies();
		let (best_held_suits, _freq_in_hand) =
			get_most_frequent_entries(&held_suit_freqs);
		if held.contains_flush() {
			return set_to_vec(best_held_suits)[0];
		}

		let deck_suit_freqs = deck.suit_frequencies();
		let eligible_suits_and_freqs: HashMap<_, _> = all::<Suit>()
			.filter_map(|suit| {
				let total_frequency = held_suit_freqs.get(&suit).unwrap_or(&0)
					+ deck_suit_freqs.get(&suit).unwrap_or(&0);
				if total_frequency >= SIZE_OF_FLUSH {
					Some((suit, total_frequency))
				} else {
					None
				}
			})
			.collect();

		match eligible_suits_and_freqs.len() {
			0 => return held.0[0].1,
			1 => return *eligible_suits_and_freqs.keys().next().unwrap(),
			_ => {},
		};

		if best_held_suits.len() == 1 {
			return set_to_vec(best_held_suits)[0];
		}

		// restrict deck to only suits that appear in `best_held_suits`.
		let held_suit_freqs_in_deck: HashMap<_, _> = deck_suit_freqs
			.into_iter()
			.filter(|(suit, _)| best_held_suits.contains(suit))
			.collect();
		let (best_suits_in_deck, _freq_in_deck) =
			get_most_frequent_entries(&held_suit_freqs_in_deck);
		set_to_vec(best_suits_in_deck)[0]
	}

	fn _suited_drawing_sanity_check(
		draw_count: usize,
		suited_count: usize,
		target_suit: Suit,
		deck: &Deck,
	) {
		let deck_len = deck.len_u64();
		let deck_suited_cards_count = deck.suited_cards_count(target_suit);

		debug_assert_ne!(draw_count, 0, "cannot draw 0 cards");
		debug_assert!(
			suited_count <= draw_count,
			"cannot demand more suited cards than are drawing: {suited_count} \
			 was not < {draw_count}"
		);
		debug_assert!(
			draw_count <= MAX_CARDS_SELECTABLE,
			"cannot draw more than {MAX_CARDS_SELECTABLE} at a time"
		);
		debug_assert!(!deck.is_empty(), "cannot draw from an empty deck");
		debug_assert!(
			draw_count <= deck_len as usize,
			"cannot draw more than available: tried to draw {draw_count} from \
			 a deck of {deck_len}",
		);
		debug_assert!(
			suited_count <= deck.suited_cards_count(target_suit),
			"cannot draw more suited cards than available: tried to draw \
			 {suited_count} suited cards from a deck with \
			 {deck_suited_cards_count} suited cards remaining."
		)
	}

	/// Returns, out of those that can be constructed from the given [`Deck`],
	/// the number of [`Card`] sets of size `d` that contain **exactly** `s`
	/// cards of the given [`Suit`].
	fn _card_combinations_with_exact_suit_count(
		d: usize,
		s: usize,
		target_suit: Suit,
		deck: &Deck,
	) -> Natural {
		Self::_suited_drawing_sanity_check(d, s, target_suit, deck);

		let deck_target_suit_count = deck.suited_cards_count(target_suit);
		let deck_non_target_suit_count = deck.len() - deck_target_suit_count;

		// draw set will be composed of `s` cards with the target suit and `d -
		// s` cards with other suits.
		let suited_combs = n_choose_k(
			deck_target_suit_count.try_into().unwrap(),
			s.try_into().unwrap(),
		);
		let unsuited_combs = n_choose_k(
			deck_non_target_suit_count.try_into().unwrap(),
			(d - s).try_into().unwrap(),
		);
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
		d: usize,
		s: usize,
		target_suit: Suit,
		deck: &Deck,
	) -> Natural {
		// prevent overdrawing
		let deck_suited_cards_count = deck.suited_cards_count(target_suit);
		let max_suited_draw = deck_suited_cards_count.min(d);
		(s..=max_suited_draw)
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
		d: usize,
		s: usize,
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
			n_choose_k(deck.len_u64(), d.try_into().unwrap()),
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
		draw_count: usize,
		suited_count: usize,
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
			n_choose_k(deck.len_u64(), draw_count.try_into().unwrap()),
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

		let target_suit = Self::get_target_suit(held, deck);
		let held_target_suit_count = held.suited_cards_count(target_suit);
		let cards_to_draw =
			(held.len() - held_target_suit_count).min(MAX_CARDS_SELECTABLE);
		let suited_cards_to_draw = SIZE_OF_FLUSH - held_target_suit_count;

		Some(Self::_probability_to_draw_at_least_n_suited(
			cards_to_draw,
			suited_cards_to_draw,
			target_suit,
			deck,
		))
	}
}

impl Strategy for FavorFlushes {
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
	fn get_strategy_data(&self, round: &Round) -> StrategyData {
		StrategyData {
			probability_to_complete_hand: self
				.probability_to_complete_flush(round),
		}
	}
}

#[cfg(test)]
mod test {
	use std::collections::{
		HashMap,
		HashSet,
	};

	use enum_iterator::all;
	use malachite::{
		Rational,
		base::num::conversion::traits::ToSci,
	};

	use crate::{
		cards::{
			Card,
			CardSet,
			Deck,
			Hand,
			Rank,
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
				n_choose_k,
			},
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
			n_choose_k(9, 1) * n_choose_k(35, 3),
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
				n_choose_k(9, 1) * n_choose_k(35, 3),
				n_choose_k(44, 4)
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
				n_choose_k(9, 2) * n_choose_k(35, 2),
				n_choose_k(44, 4)
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
					.map(|s| n_choose_k(9, s) * n_choose_k(35, 4 - s))
					.sum(),
				n_choose_k(44, 4)
			),
			"probabilty to draw 4 cards with exactly 1 heart card"
		);

		assert_eq!(
			strategy.probability_to_complete_flush(&round),
			Some(Rational::from_naturals(
				(1..=4)
					.map(|s| n_choose_k(9, s) * n_choose_k(35, 4 - s))
					.sum(),
				n_choose_k(44, 4)
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
			"discarding non-heart cards"
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

	#[test]
	fn favor_flushes_not_enough_cards_edge_case() {
		// deck skewed heavily towards diamonds but still having 5 hearts
		let deck = Deck::from_iter([
			"ah", "2h", "3h", "as", "2s", "3s", "ac", "2c", "ad", "2d", "3d",
			"4d", "5d",
		]);
		let mut round = Round::new(8, deck, 1, 1);

		round.draw_certain(&CardSet::from_iter([
			"ah", "2h", "3h", "as", "2s", "3s", "ac", "2c",
		]));
		assert_eq!(
			FavorFlushes::get_target_suit(&round.held, &round.deck),
			Suit::Diamond,
			"none of the suits other than diamonds are enough for a flush"
		);
	}

	#[test]
	fn favor_flushes_strategy_plentiful_in_deck_edge_case() {
		// deck skewed heavily towards diamonds but still having 5 hearts
		let deck = Deck::from_iter(
			[
				CardSet::from_iter(["ah", "2h", "3h", "4h", "5h", "as", "ac"])
					.to_vec(),
				all::<Rank>()
					.map(|rank| Card(rank, Suit::Diamond))
					.collect(),
			]
			.concat(),
		);
		let mut round = Round::new(8, deck, 1, 1);

		round.draw_certain(&CardSet::from_iter([
			"ah", "2h", "3h", "4h", "as", "ac", "ad", "2d",
		]));
		assert_eq!(
			FavorFlushes::get_target_suit(&round.held, &round.deck),
			Suit::Heart,
			"even though diamonds are more plentiful overall, hearts are more \
			 plentiful in hand"
		);
	}
}
