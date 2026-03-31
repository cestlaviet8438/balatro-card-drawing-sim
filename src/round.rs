//! Implementation of parts of a Balatro game, core for these simulations.

use std::collections::HashSet;

use crate::cards::{
	Card,
	CardSet,
	Deck,
	Hand,
};

/// An action in a Balatro round.
///
/// The two included actions are effectively equivalent in the sense that they
/// both are actions that remove cards from the hand and draw extra cards
/// afterwards.
pub enum Action {
	/// Discard a number of cards held in hand, drawing more cards afterwards to
	/// capacity.
	Discard,

	/// Play a number of cards held in hand, drawing more cards afterwards to
	/// capacity.
	Play,
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
pub struct Round {
	/// Whether the round has started.
	started: bool,

	/// Cards held in the hand.
	pub held: CardSet,

	/// The hand's capacity.
	pub held_capacity: usize,

	/// The deck to draw cards from.
	pub deck: Deck,

	/// The number of discards left.
	pub discard_count: usize,

	/// The pile of cards that has been discarded.
	pub discard_pile: Vec<Card>,

	/// The number of hands left.
	pub hand_count: usize,

	/// The hands that have been played.
	pub hands: Vec<Hand>,
}

impl Round {
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
			discard_count: discards,
			discard_pile: vec![],
			hand_count,
			hands: vec![],
		}
	}

	/// A default simulation of Balatro card drawing on White stake (the easiest
	/// difficulty): 4 hands and 3 discards are provided.
	pub fn white_stake_default() -> Self {
		Self::new(8, Deck::default(), 3, 4)
	}

	/// A default simulation of Balatro card drawing on Gold stake (the hardest
	/// difficulty): 4 hands and 2 discards are provided.
	pub fn gold_stake_default() -> Self {
		Self::new(8, Deck::default(), 2, 4)
	}

	/// Returns whether the round has finished. In Balatro, this is when
	/// all hands have been used up (regardless of how many discards are left).
	pub fn is_finished(&self) -> bool {
		self.hand_count == 0
	}

	/// Begin the round, drawing cards to the hand to start with.
	pub fn begin(&mut self) {
		assert!(!self.started, "the round is already started");
		self.draw_to_capacity();
	}

	/// Asserts that during an [`Action`], between 1 and 5 cards are selected
	/// and all of them are from the hand/currently held.
	fn action_sanity_check(&self, cards: &HashSet<Card>) {
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
		assert!(
			self.is_finished(),
			"the round is finished - no more actions can be taken."
		);
	}

	/// Carry out an [`Action`] in a Balatro round.
	pub fn act(&mut self, action: Action, cards: HashSet<Card>) {
		self.action_sanity_check(&cards);
		match action {
			Action::Discard => {
				assert_ne!(self.discard_count, 0, "discards have run out");
				self.discard(cards);
				self.discard_count -= 1;
			},
			Action::Play => {
				assert_ne!(self.hand_count, 0, "hands have run out");
				self.play(cards);
				self.hand_count -= 1;
			},
		}
		self.draw_to_capacity();
	}

	/// Get the number of cards to draw.
	fn get_cards_to_draw_count(&self) -> usize {
		self.held_capacity - self.held.len()
	}

	/// Draw the top cards in the deck to the hand's capacity.
	/// Note that the order of the deck is not guaranteed to be preserved
	/// between every draw or game action.
	fn draw_to_capacity(&mut self) {
		let draw_count = self.get_cards_to_draw_count();
		assert_ne!(
			draw_count, 0,
			"cannot draw more cards; hand is already full."
		);
		assert!(!self.deck.is_empty(), "deck is empty");
		self.held.extend(self.deck.draw(draw_count));
	}

	/// Removes certain cards from the hand.
	fn remove_from_hand(&mut self, cards: &HashSet<Card>) {
		let held_set: HashSet<_> = self.held.iter().copied().collect();
		self.held = held_set.difference(&cards).collect();
	}

	/// Discard certain cards from the hand.
	fn discard(&mut self, cards: HashSet<Card>) {
		self.remove_from_hand(&cards);
		self.discard_pile.extend(cards);
	}

	/// Play certain cards from the hand.
	fn play(&mut self, cards: HashSet<Card>) {
		self.remove_from_hand(&cards);
		self.hands.push(Hand::from_iter(cards));
	}
}
