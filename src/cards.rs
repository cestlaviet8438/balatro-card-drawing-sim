//! Mechanisms dealing with poker cards and hands.

use std::{
	cmp::Ordering,
	collections::{
		HashMap,
		HashSet,
	},
	error::Error,
	fmt::Display,
	ops::{
		Deref,
		DerefMut,
	},
	str::FromStr,
};

use enum_iterator::{
	Sequence,
	all,
};
use rand::{
	rngs::ThreadRng,
	seq::SliceRandom,
};
use serde::{
	Deserialize,
	Serialize,
};

/// The rank of a playing card.
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Sequence,
	Serialize,
	Deserialize,
)]
pub enum Rank {
	Two = 2,
	Three = 3,
	Four = 4,
	Five = 5,
	Six = 6,
	Seven = 7,
	Eight = 8,
	Nine = 9,
	Ten = 10,
	Jack = 11,
	Queen = 12,
	King = 13,
	Ace = 14,
}

impl Display for Rank {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_str())
	}
}

impl Rank {
	/// Returns if this rank is adjacent to another rank. This condition is
	/// required for a hand to count as a straight in poker.
	pub fn is_adjacent_to(self, other: Rank) -> bool {
		match (self, other) {
			(Rank::Two, Rank::Ace) | (Rank::Ace, Rank::Two) => true,
			_ => (self as u8).abs_diff(other as u8) == 1,
		}
	}

	/// Converts the rank to a string representation.
	pub fn to_str(&self) -> &'static str {
		match self {
			Rank::Two => "2",
			Rank::Three => "3",
			Rank::Four => "4",
			Rank::Five => "5",
			Rank::Six => "6",
			Rank::Seven => "7",
			Rank::Eight => "8",
			Rank::Nine => "9",
			Rank::Ten => "10",
			Rank::Jack => "J",
			Rank::Queen => "K",
			Rank::King => "Q",
			Rank::Ace => "A",
		}
	}
}

impl FromStr for Rank {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		debug_assert_eq!(
			s.len(),
			1,
			"rank string has to be one character. received {s}"
		);
		Ok(match s.chars().next().unwrap() {
			'2' => Rank::Two,
			'3' => Rank::Three,
			'4' => Rank::Four,
			'5' => Rank::Five,
			'6' => Rank::Six,
			'7' => Rank::Seven,
			'8' => Rank::Eight,
			'9' => Rank::Nine,
			't' | 'T' => Rank::Ten,
			'j' | 'J' => Rank::Jack,
			'q' | 'Q' => Rank::Queen,
			'k' | 'K' => Rank::King,
			'a' | 'A' => Rank::Ace,
			_ => panic!("unknown rank character: {s}"),
		})
	}
}

/// The suit of a playing card.
/// Though ordering is implemented/derived, it is not significant in the game
/// itself.
#[derive(
	Debug,
	Clone,
	Copy,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Hash,
	Sequence,
	Serialize,
	Deserialize,
)]
pub enum Suit {
	Diamond,
	Club,
	Heart,
	Spade,
}

impl Display for Suit {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.to_char().to_string())
	}
}

impl FromStr for Suit {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		debug_assert_eq!(
			s.len(),
			1,
			"suit string has to be one character. received {s}"
		);
		Ok(match s.chars().next().unwrap() {
			'd' | 'D' | '♦' | '♢' => Suit::Diamond,
			'c' | 'C' | '♣' | '♧' => Suit::Club,
			'h' | 'H' | '♥' | '♡' => Suit::Heart,
			's' | 'S' | '♠' | '♤' => Suit::Spade,
			_ => panic!("unknown suit character: {s}"),
		})
	}
}

impl Suit {
	/// Returns a character representing the suit.
	pub fn to_char(&self) -> char {
		match self {
			Suit::Diamond => '♦',
			Suit::Club => '♧',
			Suit::Heart => '♥',
			Suit::Spade => '♤',
		}
	}
}

/// A playing card in Balatro/Poker, in general.
///
/// For this simulation, enhancements and editions are not included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card(pub Rank, pub Suit);

impl Display for Card {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{}", self.0, self.1)
	}
}

impl FromStr for Card {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		debug_assert_eq!(
			s.len(),
			2,
			"card string has to be exactly two characters. received {s}"
		);
		let mut s_chars = s.chars();
		let (rank, suit) = (
			Rank::from_str(&String::from(s_chars.next().unwrap()))?,
			Suit::from_str(&String::from(s_chars.next().unwrap()))?,
		);
		Ok(Self(rank, suit))
	}
}

impl From<&Card> for Card {
	fn from(card: &Card) -> Self {
		*card
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

/// A collection of cards.
pub trait CardCollection: AsRef<[Card]> + AsMut<[Card]> {
	/// Returns the set of ranks this collection contains.
	fn rank_set(&self) -> HashSet<Rank> {
		self.as_ref().iter().map(|card| card.0).collect()
	}

	/// Returns a [`HashMap`] of ranks this collection contains, mapping each
	/// rank in the hand to how many cards shared that rank.
	fn rank_frequencies(&self) -> HashMap<Rank, usize> {
		let mut freqs = HashMap::new();
		for card in self.as_ref() {
			freqs
				.entry(card.0)
				.and_modify(|freq| *freq += 1)
				.or_insert(1);
		}
		freqs
	}

	/// Returns the set of suits this collection contains.
	fn suit_set(&self) -> HashSet<Suit> {
		self.as_ref().iter().map(|card| card.1).collect()
	}

	/// Returns a [`HashMap`] of suits this collection contains, mapping each
	/// suit in the hand to how many cards shared that suit.
	fn suit_frequencies(&self) -> HashMap<Suit, usize> {
		let mut freqs = HashMap::new();
		for card in self.as_ref() {
			freqs
				.entry(card.1)
				.and_modify(|freq| *freq += 1)
				.or_insert(1);
		}
		freqs
	}

	/// Returns a generic array-like human-readable display string of this card
	/// collection.
	fn fmt_display(&self, sort_by: SortCardsBy) -> String {
		let mut cards_str = sort_by
			.get_sorted_view(self.as_ref())
			.iter()
			.map(|card| card.to_string())
			.collect::<Vec<_>>()
			.join(", ");
		format!("[{cards_str}]")
	}
}

// apparently this doesn't work smh
// impl Display for dyn CardCollection {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		let cards_str = self
// 			.as_ref()
// 			.iter()
// 			.map(|card| card.to_string())
// 			.collect::<Vec<_>>()
// 			.join(", ");
// 		write!(f, "[{cards_str}]",)
// 	}
// }

/// Standard hands of poker. This assumes no card modifications and no joker
/// card in the deck itself, so there is no Flush House, Flush Five, or Five of
/// a Kind and such.
///
/// Note that Royal Flushes are not distinguished from a Straight Flush.
#[derive(
	Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Sequence,
)]
pub enum PokerHand {
	/// A hand that satisfies the requirement for none of the other poker hands.
	HighCard,

	/// Two cards that share the same rank.
	Pair,

	/// A set of two distinct ranks.
	TwoPair,

	/// Three cards that share the same rank.
	ThreeOfAKind,

	/// A set of 5 cards that can be arranged into a strictly
	/// increasing/decreasing sequence adjacent to each other.
	Straight,

	/// A set of 5 cards sharing the same rank.
	Flush,

	/// A pair and a three of a kind.
	FullHouse,

	/// Four cards that share the same rank.
	FourOfAKind,

	/// A straight that also qualifies as a flush.
	StraightFlush,
}

/// A set of cards in Poker, considered as a group. It is not a [`Hand`] - the
/// set can have as many cards as desired. For a played/discarded hand in
/// Balatro, see [`Hand`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardSet(pub Vec<Card>);

impl AsRef<[Card]> for CardSet {
	fn as_ref(&self) -> &[Card] {
		&self.0
	}
}

impl AsMut<[Card]> for CardSet {
	fn as_mut(&mut self) -> &mut [Card] {
		&mut self.0
	}
}

impl From<CardSet> for HashSet<Card> {
	fn from(set: CardSet) -> Self {
		set.0.into_iter().collect()
	}
}

impl Deref for CardSet {
	type Target = Vec<Card>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for CardSet {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl CardCollection for CardSet {}

impl<C: Into<Card>> FromIterator<C> for CardSet {
	fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
		Self(iter.into_iter().map(|c| c.into()).collect())
	}
}

impl<'a> FromIterator<&'a str> for CardSet {
	fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
		Self(
			iter.into_iter()
				.map(|c| Card::from_str(c).unwrap())
				.collect(),
		)
	}
}

impl PartialEq for CardSet {
	fn eq(&self, other: &Self) -> bool {
		self.0.iter().collect::<HashSet<_>>()
			== other.0.iter().collect::<HashSet<_>>()
	}
}

impl Eq for CardSet {}

impl CardSet {
	/// Checks if this card set contains a [`PokerHand::Straight`], i.e. at
	/// least 5 of the cards' ranks can be arranged in strictly increasing
	/// order by steps of 1.
	/// In a [`Hand`] of 5 cards, all of the cards can be arranged this way.
	pub fn contains_straight(&self) -> bool {
		let mut ranks = self.rank_set().into_iter().collect::<Vec<_>>();
		ranks.sort();
		if ranks.len() < 5 {
			return false;
		}
		// account for low ace if high ace is present
		if ranks.contains(&Rank::Ace) {
			ranks.insert(0, Rank::Ace);
		}

		let (mut streak, mut highest_streak) = (1, 1);
		for [rank_1, rank_2] in ranks.array_windows::<2>() {
			if rank_1.is_adjacent_to(*rank_2) {
				streak += 1;
				if highest_streak < streak {
					highest_streak = streak;
				}
			} else {
				streak = 1;
			}
		}

		highest_streak >= 5
	}

	/// Checks if this card set contains a [`PokerHand::Flush`], i.e. at least 5
	/// of the cards' suits are the same. In a [`Hand`] of 5 cards, all of the
	/// cards' suits are the same.
	pub fn contains_flush(&self) -> bool {
		self.suit_frequencies().iter().any(|(_, freq)| *freq >= 5)
	}
}

/// A played set of cards in Balatro.
///
/// This structure is a restrictive version of [`CardSet`] - a set of cards can
/// have any number of cards, but this set is restricted to only having 1 to 5
/// unique cards. This enables the hand to be categorized into a particular
/// [`PokerHand`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hand(pub CardSet);

impl AsRef<[Card]> for Hand {
	fn as_ref(&self) -> &[Card] {
		&self.0
	}
}

impl AsMut<[Card]> for Hand {
	fn as_mut(&mut self) -> &mut [Card] {
		&mut self.0
	}
}

impl Deref for Hand {
	type Target = CardSet;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Hand {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl CardCollection for Hand {}

impl<C: Into<Card>> FromIterator<C> for Hand {
	fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
		Self::new(CardSet::from_iter(iter.into_iter().map(|c| c.into())))
	}
}

impl<'a> FromIterator<&'a str> for Hand {
	fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
		Self::new(CardSet::from_iter(iter))
	}
}

impl Hand {
	/// Constructs a new hand. Cards are deduplicated.
	pub fn new(mut cards: CardSet) -> Self {
		debug_assert!(
			!cards.is_empty() && cards.len() <= 5,
			"a hand must be between 1 or 5 cards. received: {cards:?}",
		);
		cards.dedup();
		Hand(cards)
	}

	/// Checks if this hand contains a pair, i.e. at least two cards
	/// share the same rank.
	pub fn contains_pair(&self) -> bool {
		self.rank_frequencies().values().any(|freq| *freq >= 2)
	}

	/// Checks if this hand contains a two pair, i.e. there are at least
	/// two distinct pairs.
	pub fn contains_two_pair(&self) -> bool {
		self.rank_frequencies()
			.iter()
			.filter(|(_, freq)| **freq >= 2)
			.count() >= 2
	}

	/// Checks if this hand contains a three of a kind, i.e. at least
	/// three cards share the same rank.
	pub fn contains_three_of_a_kind(&self) -> bool {
		self.rank_frequencies().values().any(|freq| *freq >= 3)
	}

	/// Checks if this hand is a full house, i.e. there is a
	/// three of a kind and pair, distinct in rank.
	pub fn is_full_house(&self) -> bool {
		let freqs = self
			.rank_frequencies()
			.values()
			.copied()
			.collect::<Vec<_>>();
		freqs.contains(&2) && freqs.contains(&3)
	}

	/// Checks if this hand is a four of a kind, i.e. all four
	/// suits of the same rank are present in the hand.
	pub fn is_four_of_a_kind(&self) -> bool {
		*self.rank_frequencies().values().max().unwrap() == 4
	}

	/// Checks if this hand is a straight flush, i.e. there is
	/// both a straight and a flush in the hand.
	pub fn is_straight_flush(&self) -> bool {
		self.contains_straight() && self.contains_flush()
	}

	/// Returns whether the hand *contains* a certain poker hand, that is
	/// if the hand satisfies or *exceeds* a certain requirement for the
	/// poker hand. For example, a [`PokerHand::StraightFlush`] contains
	/// both a straight and a flush; a [`PokerHand::FourOfAKind`] contains a
	/// pair and a three of a kind as well.
	///
	/// The poker hand checking implmeneted in this struct assumes that the
	/// hand was acquired from an unmodified, standard deck of 52 playing poker
	/// cards. For this reason, this function will behave exactly like
	/// [`Self::is_poker_hand`] for the *rarer, more specific* hand
	/// types: [`PokerHand::FullHouse`], [`PokerHand::FourOfAKind`], and
	/// [`PokerHand::StraightFlush`].
	pub fn contains_poker_hand(&self, poker_hand: PokerHand) -> bool {
		match poker_hand {
			PokerHand::HighCard => true,
			PokerHand::Pair => self.contains_pair(),
			PokerHand::TwoPair => self.contains_two_pair(),
			PokerHand::ThreeOfAKind => self.contains_three_of_a_kind(),
			PokerHand::Straight => self.contains_straight(),
			PokerHand::Flush => self.contains_flush(),
			PokerHand::FullHouse => self.is_full_house(),
			PokerHand::FourOfAKind => self.is_four_of_a_kind(),
			PokerHand::StraightFlush => self.is_straight_flush(),
		}
	}

	/// Returns whether the hand is a certain poker hand.
	///
	/// The poker hand checking implemeneted in this struct assumes that the
	/// hand was acquired from an unmodified, standard deck of 52 playing poker
	/// cards.
	pub fn is_poker_hand(&self, poker_hand: PokerHand) -> bool {
		match poker_hand {
			PokerHand::HighCard => {
				!self.contains_pair()
					&& !self.contains_straight()
					&& !self.contains_flush()
			},
			PokerHand::Pair => {
				self.contains_pair()
					&& !self.contains_two_pair()
					&& !self.contains_three_of_a_kind()
					&& !self.is_four_of_a_kind()
			},
			PokerHand::TwoPair => {
				self.contains_two_pair() && !self.is_full_house()
			},
			PokerHand::ThreeOfAKind => {
				self.contains_three_of_a_kind()
					&& !self.contains_two_pair()
					&& !self.is_four_of_a_kind()
			},
			PokerHand::Straight => {
				self.contains_straight() && !self.contains_flush()
			},
			PokerHand::Flush => {
				self.contains_flush() && !self.contains_straight()
			},
			PokerHand::FullHouse => self.is_full_house(),
			PokerHand::FourOfAKind => self.is_four_of_a_kind(),
			PokerHand::StraightFlush => self.is_straight_flush(),
		}
	}

	/// Returns the best (highest-value type) that this hand qualifies as.
	/// For example, though a [`PokerHand::FourOfAKind`] contains a
	/// [`PokerHand::ThreeOfAKind`] as well, it would officially be considered
	/// a [`PokerHand::FourOfAKind`].
	pub fn get_poker_hand_type(&self) -> PokerHand {
		let result = all::<PokerHand>()
			.map(|poker_hand| {
				(poker_hand, Self::is_poker_hand(self, poker_hand))
			})
			.filter(|(_, is_poker_hand)| *is_poker_hand)
			.collect::<Vec<_>>();
		debug_assert!(
			result.len() == 1,
			"only one hand should return true. received {result:?}"
		);
		result[0].0
	}
}

/// A standard deck of 52 playing [`Card`]`s`.
/// For the purposes of the simulation, no duplicate cards are allowed, so a
/// [`HashSet`] is used to contain the cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck(Vec<Card>);

impl Default for Deck {
	fn default() -> Self {
		let mut cards = vec![];
		for rank in all::<Rank>() {
			for suit in all::<Suit>() {
				cards.push(Card(rank, suit));
			}
		}
		Self::new(cards, true)
	}
}

impl AsRef<[Card]> for Deck {
	fn as_ref(&self) -> &[Card] {
		&self.0
	}
}

impl AsMut<[Card]> for Deck {
	fn as_mut(&mut self) -> &mut [Card] {
		&mut self.0
	}
}

impl Deref for Deck {
	type Target = [Card];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Deck {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl CardCollection for Deck {}

impl Deck {
	/// Creates a new deck of cards with the given cards, and the option to
	/// shuffle.
	///
	/// Cards will be deduplicated.
	pub fn new(mut cards: Vec<Card>, shuffle: bool) -> Self {
		cards.dedup();
		let mut rng = rand::rng();
		if shuffle {
			cards.shuffle(&mut rng);
		}
		Self(cards)
	}

	/// Draw `n` cards from the top of the deck.
	///
	/// Note that if there are not enough cards left, the deck will simply
	/// be exhausted and all remaining cards are drawn. The method will,
	/// however, panic if trying to draw from an empty deck.
	pub fn draw(&mut self, n: usize) -> Vec<Card> {
		debug_assert!(n > 0, "why are you drawing 0 cards");
		debug_assert!(!self.is_empty(), "cannot draw from an empty deck");

		if n > self.len() {
			self.0.drain(..).collect()
		} else {
			let remove_slice_from = self.0.len() - n;
			self.0.drain(remove_slice_from..).collect()
		}
	}

	/// Utility function to take/remove certain cards from the deck.
	/// The cards' order is not guaranteed after this - the [`Self::cards`]
	/// property is turned into a [`HashSet`] and turned back into a [`Vec`].
	pub fn take_certain(&mut self, cards: &[Card]) {
		let mut card_set: HashSet<_> = self.iter().copied().collect();
		for card in cards {
			if !card_set.remove(card) {
				panic!("could not find card {card:?} in deck")
			}
		}
		self.0 = card_set.into_iter().collect();
	}
}

#[cfg(test)]
mod test {
	use std::collections::HashMap;

	use enum_iterator::all;

	use crate::cards::{
		CardCollection,
		CardSet,
		Deck,
		Hand,
		PokerHand,
		Rank,
		Suit,
	};

	fn lone_ace_of_hearts() -> Hand {
		Hand::from_iter(["ah"])
	}

	fn high_6_hand() -> Hand {
		Hand::from_iter(["ah", "2h", "3h", "4h", "6s"])
	}

	fn weird_straight_high_ace_hand() -> Hand {
		Hand::from_iter(["jh", "qh", "kh", "ah", "2s"])
	}

	fn lone_ace_pair() -> Hand {
		Hand::from_iter(["ah", "as"])
	}

	fn ace_pair_hand() -> Hand {
		Hand::from_iter(["ah", "as", "3h", "4h", "6s"])
	}

	fn ace_three_two_pair_hand() -> Hand {
		Hand::from_iter(["ah", "as", "3h", "3s", "6s"])
	}

	fn lone_ace_three_two_pair() -> Hand {
		Hand::from_iter(["ah", "as", "3h", "3s"])
	}

	fn three_aces_hand() -> Hand {
		Hand::from_iter(["ah", "as", "ac", "4h", "5h"])
	}

	fn lone_three_aces() -> Hand {
		Hand::from_iter(["ah", "as", "ac"])
	}

	fn ace_to_five_straight_test_hand() -> Hand {
		Hand::from_iter(["as", "2h", "3h", "4h", "5h"])
	}

	fn ten_to_ace_straight_test_hand() -> Hand {
		Hand::from_iter(["ts", "jh", "qh", "kh", "ah"])
	}

	fn heart_flush_test_hand() -> Hand {
		Hand::from_iter(["ah", "2h", "3h", "4h", "6h"])
	}

	fn spade_flush_test_hand() -> Hand {
		Hand::from_iter(["as", "3s", "5s", "7s", "9s"])
	}

	fn ace_four_full_house_test_hand() -> Hand {
		Hand::from_iter(["ah", "as", "ac", "4h", "4s"])
	}

	fn four_aces_test_hand() -> Hand {
		Hand::from_iter(["ah", "as", "ac", "ad", "6h"])
	}

	fn lone_four_aces() -> Hand {
		Hand::from_iter(["ah", "as", "ac", "ad"])
	}

	fn ace_to_five_heart_straight_flush_test_hand() -> Hand {
		Hand::from_iter(["ah", "2h", "3h", "4h", "5h"])
	}

	fn ten_to_ace_heart_straight_flush_test_hand() -> Hand {
		Hand::from_iter(["th", "jh", "qh", "kh", "ah"])
	}

	fn card_set_with_ten_to_ace_straight_flush() -> CardSet {
		CardSet::from_iter(["th", "jh", "qh", "kh", "ah", "2h", "3h", "4h"])
	}

	fn card_set_with_broken_straight() -> CardSet {
		CardSet::from_iter(["jh", "qh", "kh", "ah", "2h", "3h", "4h", "6h"])
	}

	fn card_set_with_almost_straight() -> CardSet {
		CardSet::from_iter(["ah", "2h", "3h", "4h", "3h", "2h", "ah", "6h"])
	}

	fn card_set_with_almost_flush() -> CardSet {
		CardSet::from_iter(["ah", "2h", "3h", "4h", "3s", "2s", "as", "6s"])
	}

	#[should_panic(expected = "a hand must be between 1 or 5 cards")]
	#[test]
	fn cannot_make_an_empty_hand() {
		let _ = Hand::from_iter::<[&str; 0]>([]);
	}

	#[should_panic(expected = "a hand must be between 1 or 5 cards")]
	#[test]
	fn cannot_make_a_hand_with_6_cards() {
		let _ = Hand::from_iter(["ah", "ah", "ah", "ah", "ah", "ah"]);
	}

	#[test]
	fn high_card_tests() {
		assert!(
			high_6_hand().is_poker_hand(PokerHand::HighCard),
			"high 6 card"
		);
		assert!(
			lone_ace_of_hearts().is_poker_hand(PokerHand::HighCard),
			"lone high ace"
		);
		assert!(
			!ace_pair_hand().is_poker_hand(PokerHand::HighCard),
			"pair is not a high card"
		);
		assert!(
			!ace_to_five_straight_test_hand()
				.is_poker_hand(PokerHand::HighCard),
			"straight is not a high card"
		);
		assert!(
			!heart_flush_test_hand().is_poker_hand(PokerHand::HighCard),
			"flush is not a high card"
		);
	}

	#[test]
	pub fn pair_tests() {
		assert!(ace_pair_hand().is_poker_hand(PokerHand::Pair), "ace pair");
		assert!(
			lone_ace_pair().is_poker_hand(PokerHand::Pair),
			"lone ace pair"
		);
		assert!(
			!high_6_hand().is_poker_hand(PokerHand::Pair),
			"high card is not a pair"
		);
		assert!(
			!ace_three_two_pair_hand().is_poker_hand(PokerHand::Pair),
			"two pair is not a pair"
		);
		assert!(
			!three_aces_hand().is_poker_hand(PokerHand::Pair),
			"three of a kind is not a pair"
		);
		assert!(
			!four_aces_test_hand().is_poker_hand(PokerHand::Pair),
			"four of a kind is not a pair"
		);
	}

	#[test]
	pub fn two_pair_tests() {
		assert!(
			ace_three_two_pair_hand().is_poker_hand(PokerHand::TwoPair),
			"ace & 3 two pair"
		);
		assert!(
			lone_ace_three_two_pair().is_poker_hand(PokerHand::TwoPair),
			"lone ace & 3 two pair"
		);
		assert!(
			!three_aces_hand().is_poker_hand(PokerHand::TwoPair),
			"three of a kind is not a two pair"
		);
		assert!(
			!ace_four_full_house_test_hand().is_poker_hand(PokerHand::TwoPair),
			"full house is not a two pair"
		);
		assert!(
			!four_aces_test_hand().is_poker_hand(PokerHand::TwoPair),
			"four of a kind is not a two pair"
		);
	}

	#[test]
	pub fn three_of_a_kind_tests() {
		assert!(
			three_aces_hand().is_poker_hand(PokerHand::ThreeOfAKind),
			"three aces of a kind"
		);
		assert!(
			lone_three_aces().is_poker_hand(PokerHand::ThreeOfAKind),
			"three aces"
		);
		assert!(
			!ace_pair_hand().is_poker_hand(PokerHand::ThreeOfAKind),
			"pair is not a three of a kind"
		);
		assert!(
			!ace_three_two_pair_hand().is_poker_hand(PokerHand::ThreeOfAKind),
			"two pair is not a three of a kind"
		);
		assert!(
			!ace_four_full_house_test_hand()
				.is_poker_hand(PokerHand::ThreeOfAKind),
			"full house is not a three of a kind"
		);
		assert!(
			!four_aces_test_hand().is_poker_hand(PokerHand::ThreeOfAKind),
			"four of a kind is not a three of a kind"
		);
	}

	#[test]
	pub fn straight_tests() {
		assert!(
			ace_to_five_straight_test_hand().is_poker_hand(PokerHand::Straight),
			"ace to five straight"
		);
		assert!(
			ten_to_ace_straight_test_hand().is_poker_hand(PokerHand::Straight),
			"ten to ace straight"
		);
		assert!(
			!ace_to_five_heart_straight_flush_test_hand()
				.is_poker_hand(PokerHand::Straight),
			"straight flush is not a straight"
		);
		assert!(
			!high_6_hand().is_poker_hand(PokerHand::Straight),
			"high card is not a straight"
		);
		assert!(
			!weird_straight_high_ace_hand().is_poker_hand(PokerHand::Straight),
			"weird straight (jkqa2) is not a valid straight"
		);
		assert!(
			!ace_pair_hand().is_poker_hand(PokerHand::ThreeOfAKind),
			"pair is not a straight"
		);
		assert!(
			card_set_with_ten_to_ace_straight_flush().contains_straight(),
			"card set contains ten to ace straight (flush)"
		);
		assert!(
			!card_set_with_broken_straight().contains_straight(),
			"broken ace does not make a straight in card set"
		);
		assert!(
			!card_set_with_almost_straight().contains_straight(),
			"card set has no straight though all are adjacent to next card"
		);
	}

	#[test]
	pub fn flush_tests() {
		assert!(
			heart_flush_test_hand().is_poker_hand(PokerHand::Flush),
			"heart flush"
		);
		assert!(
			spade_flush_test_hand().is_poker_hand(PokerHand::Flush),
			"spade flush"
		);
		assert!(
			!ace_to_five_heart_straight_flush_test_hand()
				.is_poker_hand(PokerHand::Flush),
			"straight flush is not a flush"
		);
		assert!(
			!high_6_hand().is_poker_hand(PokerHand::Flush),
			"high card is not a flush"
		);
		assert!(
			!ace_pair_hand().is_poker_hand(PokerHand::Flush),
			"pair is not a straight"
		);
		assert!(
			card_set_with_ten_to_ace_straight_flush().contains_flush(),
			"card set contains ten to ace (straight) flush"
		);
		assert!(
			!card_set_with_almost_flush().contains_flush(),
			"card set does not contain a flush since it only has 4 of each \
			 suit"
		)
	}

	#[test]
	pub fn full_house_tests() {
		assert!(
			ace_four_full_house_test_hand().is_poker_hand(PokerHand::FullHouse),
			"ace four full house"
		);
		assert!(
			!ace_three_two_pair_hand().is_poker_hand(PokerHand::FullHouse),
			"two pair is not a full house"
		);
		assert!(
			!three_aces_hand().is_poker_hand(PokerHand::FullHouse),
			"three of a kind is not a full house"
		);
		assert!(
			!four_aces_test_hand().is_poker_hand(PokerHand::FullHouse),
			"four of a kind is not a full house"
		);
	}

	#[test]
	pub fn four_of_a_kind_tests() {
		assert!(
			four_aces_test_hand().is_poker_hand(PokerHand::FourOfAKind),
			"four aces"
		);
		assert!(
			lone_four_aces().is_poker_hand(PokerHand::FourOfAKind),
			"lone four aces"
		);
		assert!(
			!ace_four_full_house_test_hand()
				.is_poker_hand(PokerHand::FourOfAKind),
			"full house is not a four of a kind"
		);
	}

	#[test]
	pub fn straight_flush_tests() {
		assert!(
			ace_to_five_heart_straight_flush_test_hand()
				.is_poker_hand(PokerHand::StraightFlush),
			"ace to five straight heart flush"
		);
		assert!(
			ten_to_ace_heart_straight_flush_test_hand()
				.is_poker_hand(PokerHand::StraightFlush),
			"ten to ace straight heart flush"
		);
		assert!(
			!ace_to_five_straight_test_hand()
				.is_poker_hand(PokerHand::FourOfAKind),
			"straight is not a straight flush"
		);
		assert!(
			!heart_flush_test_hand().is_poker_hand(PokerHand::StraightFlush),
			"flush is not a straight flush"
		)
	}

	#[test]
	pub fn get_poker_hand_type_test() {
		assert_eq!(
			high_6_hand().get_poker_hand_type(),
			PokerHand::HighCard,
			"high 6 is a high card"
		);
		assert_eq!(
			weird_straight_high_ace_hand().get_poker_hand_type(),
			PokerHand::HighCard,
			"weird straight is a high card"
		);
		assert_eq!(
			ace_pair_hand().get_poker_hand_type(),
			PokerHand::Pair,
			"ace pair is a pair"
		);
		assert_eq!(
			ace_three_two_pair_hand().get_poker_hand_type(),
			PokerHand::TwoPair,
			"ace & three two pair is a two pair"
		);
		assert_eq!(
			three_aces_hand().get_poker_hand_type(),
			PokerHand::ThreeOfAKind,
			"three aces is a three of a kind"
		);
		assert_eq!(
			ten_to_ace_straight_test_hand().get_poker_hand_type(),
			PokerHand::Straight,
			"ten to ace straight is a straight"
		);
		assert_eq!(
			ace_to_five_straight_test_hand().get_poker_hand_type(),
			PokerHand::Straight,
			"ace to five straight is a straight"
		);
		assert_eq!(
			heart_flush_test_hand().get_poker_hand_type(),
			PokerHand::Flush,
			"heart flush is a flush"
		);
		assert_eq!(
			spade_flush_test_hand().get_poker_hand_type(),
			PokerHand::Flush,
			"spade flush is a flush"
		);
		assert_eq!(
			ace_four_full_house_test_hand().get_poker_hand_type(),
			PokerHand::FullHouse,
			"ace & four full house is a full house"
		);
		assert_eq!(
			four_aces_test_hand().get_poker_hand_type(),
			PokerHand::FourOfAKind,
			"four aces are a four of a kind"
		);
		assert_eq!(
			ten_to_ace_heart_straight_flush_test_hand().get_poker_hand_type(),
			PokerHand::StraightFlush,
			"ten to ace heart flush is a straight flush"
		);
		assert_eq!(
			ace_to_five_heart_straight_flush_test_hand().get_poker_hand_type(),
			PokerHand::StraightFlush,
			"ace to five heart flush is a straight flush"
		);
	}

	#[test]
	fn deck_draw_works() {
		let mut deck = Deck::default();
		deck.draw(5);
		assert_eq!(
			deck.len(),
			47,
			"there are 47 cards left in deck after drawing 5"
		);
	}

	#[test]
	fn default_deck_is_standard() {
		let default_deck = Deck::default();
		let default_rank_freqs = all::<Rank>().map(|rank| (rank, 4)).collect();
		let default_suit_freqs = all::<Suit>().map(|suit| (suit, 13)).collect();
		assert_eq!(default_deck.len(), 52, "a standard deck has 52 cards");
		assert_eq!(
			default_deck.rank_frequencies(),
			default_rank_freqs,
			"a standard deck has 4 of each rank"
		);
		assert_eq!(
			default_deck.suit_frequencies(),
			default_suit_freqs,
			"a standard deck has 13 of each suit"
		);
	}
}
