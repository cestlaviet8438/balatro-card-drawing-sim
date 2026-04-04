//! Implementation of parts of a Balatro game, core for these simulations.

use std::{
	cmp::Ordering,
	collections::HashSet,
	fmt::{
		Display,
		Write,
	},
};

use crate::cards::{
	Card,
	CardCollection,
	CardSet,
	Deck,
	Hand,
};

/// An action in a Balatro round.
///
/// The two included actions are effectively equivalent in the sense that they
/// both are actions that remove cards from the hand and draw extra cards
/// afterwards.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
	/// Discard a number of cards held in hand, drawing more cards afterwards to
	/// capacity.
	Discard,

	/// Play a number of cards held in hand, drawing more cards afterwards to
	/// capacity.
	Play,
}

impl Display for Action {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Action::Discard => "discard",
			Action::Play => "play",
		})
	}
}

/// Ways to view sorted cards by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortCardsBy {
	/// Sort cards by rank first, then by suits.
	RanksFirst,

	/// Sort cards by suits first, then by rank.
	SuitsFirst,
}

impl SortCardsBy {
	/// Compares two cards based on the sorting strategy.
	pub fn compare_cards(&self, card_1: Card, card_2: Card) -> Ordering {
		let rank_cmp = card_1.0.cmp(&card_2.0);
		let suit_cmp = card_1.1.cmp(&card_2.1);
		match self {
			SortCardsBy::RanksFirst => {
				if rank_cmp == Ordering::Equal {
					suit_cmp
				} else {
					rank_cmp
				}
			},
			SortCardsBy::SuitsFirst => {
				if suit_cmp == Ordering::Equal {
					rank_cmp
				} else {
					suit_cmp
				}
			},
		}
	}

	/// Returns an ordered [`Vec`] arranging the cards in a sorted fashion.
	pub fn get_sorted_view(&self, cards: &[Card]) -> Vec<Card> {
		let mut cards = cards.to_vec();
		cards.sort_by(|card_1, card_2| self.compare_cards(*card_1, *card_2));
		cards
	}
}

/// A basic simulation of a Balatro round, with only the cards part and one hand
/// (chance to play a set of cards).
///
/// Certain restrictions are imposed. The [`Deck`] can only contain unique
/// cards. Discards and hands can only select a maximum of 5 cards.
///
/// The simulation includes every information a Balatro player has access
/// to: cards currently held in hand, discarded cards, and remaining cards in
/// the deck.
#[derive(Debug, Clone)]
pub struct Round {
	/// Whether the round has started.
	started: bool,

	/// Cards held in the hand.
	pub(crate) held: CardSet,

	/// The hand's capacity.
	pub(crate) held_capacity: usize,

	/// The deck to draw cards from.
	pub(crate) deck: Deck,

	/// The number of discards left.
	pub(crate) discards_remaining: usize,

	/// The pile of cards that has been discarded.
	pub(crate) discard_pile: Vec<Card>,

	/// The number of hands left.
	pub(crate) plays_remaining: usize,

	/// The hands that have been played.
	pub(crate) plays: Vec<Hand>,

	/// The history of actions taken during this round.
	pub(crate) history: Vec<(Action, Hand)>,
}

impl Round {
	const BALATRO_HELD_CAPACITY: usize = 8;

	/// Construct a new round, given a capacity, a [`Deck`], a given number of
	/// discards and hands, and no held cards in the beginning.
	pub fn new(
		held_capacity: usize,
		deck: Deck,
		discards: usize,
		hand_count: usize,
	) -> Self {
		Self {
			started: false,
			held: CardSet(Vec::new()),
			held_capacity,
			deck,
			discards_remaining: discards,
			discard_pile: vec![],
			plays_remaining: hand_count,
			plays: vec![],
			history: vec![],
		}
	}

	/// A default simulation of Balatro card drawing on White stake (the easiest
	/// difficulty): 4 hands and 3 discards are provided.
	pub fn white_stake_default() -> Self {
		Self::new(Self::BALATRO_HELD_CAPACITY, Deck::default(), 3, 4)
	}

	/// A default simulation of Balatro card drawing on Gold stake (the hardest
	/// difficulty): 4 hands and 2 discards are provided.
	pub fn gold_stake_default() -> Self {
		Self::new(Self::BALATRO_HELD_CAPACITY, Deck::default(), 2, 4)
	}

	/// Returns a printable string showing the round status.
	pub fn fmt_status(&self, card_sort: SortCardsBy) -> String {
		let last_hand_string =
			if let Some((last_action, last_hand)) = self.history.last() {
				let sorted_last_hand =
					CardSet::from_iter(card_sort.get_sorted_view(last_hand));
				format!(
					"\nlast action: {}; hand: {}",
					last_action.to_string(),
					sorted_last_hand.fmt_display(card_sort)
				)
			} else {
				"".into()
			};

		format!(
			"started: {}\ndiscards remaining: {}\nplays remaining: {}\nheld: \
			 {} (capacity {}){}",
			self.started,
			self.discards_remaining,
			self.plays_remaining,
			self.held.fmt_display(card_sort),
			self.held_capacity,
			last_hand_string,
		)
	}

	/// Returns whether the round has finished. In Balatro, this is when
	/// all hands have been used up (regardless of how many discards are left).
	pub fn is_finished(&self) -> bool {
		self.plays_remaining == 0
	}

	/// Begin the round, drawing cards to the hand to start with.
	pub fn begin(&mut self) {
		assert!(!self.started, "cannot start an already started round");
		self.started = true;
		self.draw_to_capacity();
	}

	/// Asserts that during an [`Action`], between 1 and 5 cards are selected
	/// and all of them are from the hand/currently held.
	fn action_sanity_check(&self, cards: &Hand) {
		assert!(
			!cards.is_empty() && cards.len() <= 5,
			"an action can only be done with between 1 to 5 cards selected. \
			 received {cards:?}"
		);
		assert!(
			cards.iter().all(|card| self.held.contains(card)),
			"an action can only be done with cards that are being held. \
			 received {cards:?}"
		);
		assert!(!self.is_finished(), "cannot act when round is finished");
		assert_eq!(
			cards.len(),
			cards.iter().collect::<HashSet<_>>().len(),
			"no duplicate cards are allowed. received {cards:?}"
		);
	}

	/// Carry out an [`Action`] in a Balatro round.
	pub fn act(&mut self, action: Action, cards: Hand) {
		self.action_sanity_check(&cards);
		match action {
			Action::Discard => {
				assert_ne!(self.discards_remaining, 0, "discards have run out");
				self.discard(&cards);
				self.discards_remaining -= 1;
			},
			Action::Play => {
				// action sanity has already checked if there are hands left
				self.play(&cards);
				self.plays_remaining -= 1;
			},
		}
		self.history.push((action, cards));
		self.draw_to_capacity();
	}

	/// Get the number of cards to draw.
	pub fn get_cards_to_draw_count(&self) -> usize {
		self.held_capacity - self.held.len()
	}

	/// Draw the top cards in the deck to the hand's capacity.
	/// Note that the order of the deck is not guaranteed to be preserved
	/// between every draw or game action.
	pub(crate) fn draw_to_capacity(&mut self) {
		let draw_count = self.get_cards_to_draw_count();
		assert_ne!(
			draw_count, 0,
			"cannot draw more cards; hand is already full."
		);
		assert!(!self.deck.is_empty(), "deck is empty");
		self.held.extend(self.deck.draw(draw_count));
	}

	/// Draw certain cards to the hand.
	/// This is used mostly to create mock hands for testing. Drawing over
	/// the capacity is not checked.
	pub(crate) fn draw_certain(&mut self, cards: &[Card]) {
		self.deck.take_certain(cards);
		self.held.extend_from_slice(cards);
	}

	/// Removes certain cards from the hand.
	pub(crate) fn remove_from_hand(&mut self, cards: &[Card]) {
		let held_set: HashSet<_> = self.held.iter().copied().collect();
		self.held = held_set
			.difference(&cards.iter().copied().collect())
			.collect();
	}

	/// Returns the first `n` cards from the hand.
	pub(crate) fn get_first_held_cards(&self, n: usize) -> Vec<Card> {
		self.held[0..n].iter().copied().collect()
	}

	/// Discard certain cards from the hand.
	pub(crate) fn discard(&mut self, cards: &[Card]) {
		self.remove_from_hand(cards);
		self.discard_pile.extend(cards);
	}

	/// Play certain cards from the hand.
	pub(crate) fn play(&mut self, cards: &[Card]) {
		self.remove_from_hand(cards);
		self.plays.push(Hand::from_iter(cards));
	}
}

#[cfg(test)]
mod test {
	use std::collections::HashSet;

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
	};

	#[test]
	fn manual_round_works() {
		let mut round = Round::white_stake_default();

		let flush = vec!["ah", "2h", "3h", "4h", "6h"];
		let other_cards = vec!["7s", "8s", "9s"];

		let flush_and_other =
			CardSet::from_iter([flush.clone(), other_cards.clone()].concat());
		round.draw_certain(&flush_and_other);
		assert_eq!(round.held.len(), 8, "currently round has 8 held cards");

		round.discard(&CardSet::from_iter(other_cards));
		assert_eq!(round.held.len(), 5, "after discard round has 5 cards");

		let flush = CardSet::from_iter(["ah", "2h", "3h", "4h", "6h"]);
		round.play(&flush);
		assert_eq!(
			round.held.len(),
			0,
			"after play all cards have been played"
		);
		assert!(
			round.plays[0].is_poker_hand(PokerHand::Flush),
			"a flush has been played"
		);
	}

	#[test]
	fn actual_round_works() {
		let mut round = Round::white_stake_default();

		round.begin();

		assert_eq!(
			round.deck.len(),
			52 - 8,
			"8 cards are drawn during beginning"
		);
		assert_eq!(round.held.len(), 8, "8 cards are drawn during beginning");

		round.act(
			Action::Discard,
			Hand::from_iter(round.get_first_held_cards(5)),
		);
		assert_eq!(round.discard_pile.len(), 5, "5 cards are discarded");
		assert_eq!(
			round.held.len(),
			8,
			"cards are redrawn to capacity after discard action"
		);
		assert_eq!(round.discards_remaining, 3 - 1, "2 discards are left");

		round.act(Action::Play, Hand::from_iter(round.get_first_held_cards(4)));
		assert_eq!(round.plays[0].len(), 4, "4 cards are played");
		assert_eq!(
			round.held.len(),
			8,
			"cards are redrawn to capacity after play action"
		);
		assert_eq!(round.plays_remaining, 4 - 1, "3 hands are left");

		for _ in 0..3 {
			round.act(
				Action::Play,
				Hand::from_iter(round.get_first_held_cards(1)),
			);
			assert_eq!(
				round.held.len(),
				8,
				"cards are redrawn to capacity after play action"
			);
		}
		assert_eq!(round.plays.len(), 4, "4 total plays have been made");
		assert_eq!(round.plays_remaining, 0, "0 hands are left");
		assert!(
			round.is_finished(),
			"0 hands are left, so the round is finished"
		);
	}

	#[test]
	#[should_panic]
	fn cannot_play_when_finished() {
		let mut round = Round::white_stake_default();
		round.begin();
		round.plays_remaining = 0;
		round.act(Action::Play, Hand::from_iter(round.get_first_held_cards(1)));
	}
}
