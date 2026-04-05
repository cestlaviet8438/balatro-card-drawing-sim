//! Implementation of parts of a Balatro game, core for these simulations.

use std::{
	cmp::Ordering,
	collections::HashSet,
	fmt::{
		Display,
		Write,
	},
};

use serde::{
	Deserialize,
	Serialize,
};

use crate::cards::{
	Card,
	CardCollection,
	CardSet,
	Deck,
	Hand,
	SortCardsBy,
};

/// An action in a Balatro round.
///
/// The two included actions are effectively equivalent in the sense that they
/// both are actions that remove cards from the hand and draw extra cards
/// afterwards.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
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

/// A stake/difficulty setting in Balatro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stake {
	/// The minimum difficulty in Balatro. Starts with 3 discards, and 4
	/// hands/plays.
	White,

	/// The maximum difficulty in Balatro. Starts with 2 discards, and 4
	/// hands/plays.
	Gold,
}

impl Stake {
	/// Returns, respectively, the number of discards and the number of plays
	/// this difficulty starts with.
	pub fn get_discards_and_plays(&self) -> (usize, usize) {
		match self {
			Stake::White => (3, 4),
			Stake::Gold => (2, 4),
		}
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
	/// Whether the round has started.
	#[serde(skip)]
	started: bool,

	/// Cards held in the hand.
	#[serde(skip)]
	pub held: CardSet,

	/// The hand's capacity.
	pub held_capacity: usize,

	/// The deck to draw cards from.
	#[serde(skip)]
	pub deck: Deck,

	/// The number of discards this round starts with.
	pub discards_given: usize,

	/// The number of discards left.
	pub discards_remaining: usize,

	/// The pile of cards that has been discarded.
	#[serde(skip)]
	pub discard_pile: Vec<Card>,

	/// The number of plays this round started with.
	pub plays_given: usize,

	/// The number of plays left.
	pub plays_remaining: usize,

	/// The hands that have been played.
	pub plays: Vec<Hand>,

	/// The history of actions taken during this round.
	pub history: Vec<(Action, Hand)>,
}

impl Round {
	const BALATRO_HELD_CAPACITY: usize = 8;

	/// Construct a new round, given a capacity, a [`Deck`], a given number of
	/// discards and hands, and no held cards in the beginning.
	pub fn new(
		held_capacity: usize,
		deck: Deck,
		discards: usize,
		plays: usize,
	) -> Self {
		Self {
			started: false,
			held: CardSet(Vec::new()),
			held_capacity,
			deck,
			discards_given: discards,
			discards_remaining: discards,
			discard_pile: vec![],
			plays_given: plays,
			plays_remaining: plays,
			plays: vec![],
			history: vec![],
		}
	}

	/// A default simulation of Balatro card drawing on a certain stake.
	pub fn default_with_stake(stake: Stake) -> Self {
		let (discards, plays) = stake.get_discards_and_plays();
		Self::new(8, Deck::default(), discards, plays)
	}

	/// Returns a printable string showing the round status.
	pub fn fmt_status(&self, card_sort: SortCardsBy) -> String {
		let last_hand_string =
			if let Some((last_action, last_hand)) = self.history.last() {
				let sorted_last_hand =
					CardSet::from_iter(card_sort.get_sorted_view(last_hand));
				format!(
					"\nlast action: {}; hand: {}",
					last_action,
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
	pub fn draw_to_capacity(&mut self) {
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
	pub fn draw_certain(&mut self, cards: &[Card]) {
		self.deck.take_certain(cards);
		self.held.extend_from_slice(cards);
	}

	/// Removes certain cards from the hand.
	pub fn remove_from_hand(&mut self, cards: &[Card]) {
		let held_set: HashSet<_> = self.held.iter().copied().collect();
		self.held = held_set
			.difference(&cards.iter().copied().collect())
			.collect();
	}

	/// Returns the first `n` cards from the hand.
	pub fn get_first_held_cards(&self, n: usize) -> Vec<Card> {
		self.held[0..n].to_vec()
	}

	/// Discard certain cards from the hand.
	pub fn discard(&mut self, cards: &[Card]) {
		self.remove_from_hand(cards);
		self.discard_pile.extend(cards);
	}

	/// Play certain cards from the hand.
	pub fn play(&mut self, cards: &[Card]) {
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
			Stake,
		},
	};

	#[test]
	fn manual_round_works() {
		let mut round = Round::default_with_stake(Stake::White);

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
		let mut round = Round::default_with_stake(Stake::White);

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
		let mut round = Round::default_with_stake(Stake::White);
		round.begin();
		round.plays_remaining = 0;
		round.act(Action::Play, Hand::from_iter(round.get_first_held_cards(1)));
	}
}
