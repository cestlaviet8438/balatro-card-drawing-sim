use std::{
	collections::{
		HashMap,
		HashSet,
	},
	error::Error,
	ops::Deref,
	str::FromStr,
};

/// The rank of a playing card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl Rank {
	/// Returns if this rank is adjacent to another rank. This condition is
	/// required for a hand to count as a straight in poker.
	pub fn is_adjacent_to(self, other: Rank) -> bool {
		match (self, other) {
			(Rank::Two, Rank::Ace) | (Rank::Ace, Rank::Two) => true,
			_ => (self as u8).abs_diff(other as u8) == 1,
		}
	}
}

impl FromStr for Rank {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		assert_eq!(s.len(), 1, "rank string has to be one character");
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
	Diamond,
	Club,
	Heart,
	Spade,
}

impl FromStr for Suit {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		assert_eq!(s.len(), 1, "suit string has to be one character");
		Ok(match s.chars().next().unwrap() {
			'd' | 'D' | '♦' | '♢' => Suit::Diamond,
			'c' | 'C' | '♣' | '♧' => Suit::Club,
			'h' | 'H' | '♥' | '♡' => Suit::Heart,
			's' | 'S' | '♠' | '♤' => Suit::Spade,
			_ => panic!("unknown suit character: {s}"),
		})
	}
}

/// A playing card in Balatro/Poker, in general.
///
/// For this simulation, enhancements and editions are not included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card(Rank, Suit);

impl FromStr for Card {
	type Err = Box<dyn Error>;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		assert_eq!(s.len(), 2, "card string has to be exactly two characters");
		let mut s_chars = s.chars();
		let (rank, suit) = (
			Rank::from_str(&String::from(s_chars.next().unwrap()))?,
			Suit::from_str(&String::from(s_chars.next().unwrap()))?,
		);
		Ok(Self(rank, suit))
	}
}

/// Standard hands of poker. This assumes no card modifications and no joker
/// card in the deck itself, so there is no Flush House, Flush Five, or Five of
/// a Kind and such.
///
/// Note that Royal Flushes are not distinguished from a Straight Flush.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PokerHand {
	HighCard,
	Pair,
	TwoPair,
	ThreeOfAKind,
	Straight,
	Flush,
	FullHouse,
	FourOfAKind,
	StraightFlush,
}

pub type FiveCards = [Card; 5];

/// A hand of cards in Poker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hand(FiveCards);

impl Deref for Hand {
	type Target = FiveCards;

	fn deref(&self) -> &FiveCards {
		&self.0
	}
}

impl From<[&str; 5]> for Hand {
	fn from(cards: [&str; 5]) -> Self {
		Self::from(
			cards
				.into_iter()
				.map(|s| Card::from_str(s).unwrap())
				.collect::<Vec<_>>(),
		)
	}
}

impl From<Vec<&str>> for Hand {
	fn from(cards: Vec<&str>) -> Self {
		Self::from(
			cards
				.into_iter()
				.map(|s| Card::from_str(s).unwrap())
				.collect::<Vec<_>>(),
		)
	}
}

impl From<Vec<Card>> for Hand {
	fn from(mut cards: Vec<Card>) -> Self {
		assert_eq!(cards.len(), 5, "poker hand must be exactly 5 cards");
		cards.sort();
		Self::new(cards.as_array::<5>().unwrap())
	}
}

impl Hand {
	/// Constructs a new hand. Note that cards are sorted then stored.
	pub fn new(cards: &FiveCards) -> Self {
		let mut cards_vec = cards.to_vec();
		cards_vec.sort();
		Self(*cards_vec.as_array::<5>().unwrap())
	}

	/// Returns the set of ranks this hand contains.
	pub fn rank_set(&self) -> HashSet<Rank> {
		self.iter().map(|card| card.0).collect()
	}

	/// Returns a [`HashMap`] of ranks this hand contains, mapping each
	/// rank in the hand to how many cards shared that rank.
	pub fn rank_counts(&self) -> HashMap<Rank, u8> {
		let mut counts = HashMap::new();
		for card in self.iter() {
			counts
				.entry(card.0)
				.and_modify(|count| *count += 1)
				.or_insert(1);
		}
		counts
	}

	/// Returns the set of suits this hand contains.
	pub fn suit_set(&self) -> HashSet<Suit> {
		self.iter().map(|card| card.1).collect()
	}

	/// Returns a [`HashMap`] of suits this hand contains, mapping each
	/// suit in the hand to how many cards shared that rank.
	pub fn suit_counts(&self) -> HashMap<Suit, u8> {
		let mut counts = HashMap::new();
		for card in self.iter() {
			counts
				.entry(card.1)
				.and_modify(|count| *count += 1)
				.or_insert(1);
		}
		counts
	}

	/// Checks if the provided hand contains a pair, i.e. at least two cards
	/// share the same rank.
	pub fn contains_pair(&self) -> bool {
		self.rank_set().len() <= 4
	}

	/// Checks if the provided hand contains a two pair, i.e. there are at least
	/// two distinct pairs in the hand.
	pub fn contains_two_pair(&self) -> bool {
		let rank_set = self.rank_set();
		// a two pair has 3 unique ranks (2 if it is a full house).
		if rank_set.len() != 2 && rank_set.len() != 3 {
			return false;
		}

		let rank_counts = self.rank_counts();
		// a two pair/full house's rank counts looks like: [2, 2, 1] or [2, 3].
		rank_counts.values().collect::<HashSet<_>>().contains(&2)
	}

	/// Checks if the provided hand contains a three of a kind, i.e. at least
	/// three cards share the same rank.
	pub fn contains_three_of_a_kind(&self) -> bool {
		*self.rank_counts().values().max().unwrap() == 3
	}

	/// Checks if the provided hand contains a straight, i.e. all of the
	/// cards' ranks can be arranged in strictly increasing order, by steps of
	/// 1, and the ace (if there is one) is not simultaneously a high or low
	/// ace. For example, JKQA2 is not a straight.
	pub fn contains_straight(&self) -> bool {
		fn verify_adjacency(ranks: &[Rank]) -> bool {
			ranks
				.windows(2)
				.map(|pair| (pair[0], pair[1]))
				.all(|(rank_1, rank_2)| rank_1.is_adjacent_to(rank_2))
		}

		let mut ranks: Vec<_> = self.rank_set().into_iter().collect();
		if ranks.len() < 5 {
			return false;
		}
		ranks.sort();

		if ranks.contains(&Rank::Ace) {
			let ace_straights = [
				[Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Ace],
				[Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace],
			];
			return ace_straights.contains(ranks.as_array().unwrap());
		}

		verify_adjacency(&ranks)
	}

	/// Checks if the provided hand contains a flush, i.e. all of the cards'
	/// suits are the same.
	pub fn contains_flush(&self) -> bool {
		self.suit_set().len() == 1
	}

	/// Checks if the provided hand is a full house, i.e. there is a
	/// three of a kind and pair, distinct in rank.
	pub fn is_full_house(&self) -> bool {
		self.rank_set().len() == 2 && !self.is_four_of_a_kind()
	}

	/// Checks if the provided hand is a four of a kind, i.e. all four
	/// suits of the same rank are present in the hand.
	pub fn is_four_of_a_kind(&self) -> bool {
		*self.rank_counts().values().max().unwrap() == 4
	}

	/// Checks if the provided hand is a straight flush, i.e. there is
	/// both a straight and a flush in the hand.
	pub fn is_straight_flush(&self) -> bool {
		self.contains_straight() && self.contains_flush()
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
					&& !self.contains_three_of_a_kind()
					&& !self.is_four_of_a_kind()
			},
			PokerHand::TwoPair => {
				self.contains_two_pair() && !self.is_full_house()
			},
			PokerHand::ThreeOfAKind => {
				self.contains_three_of_a_kind() && !self.contains_two_pair()
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
}

#[cfg(test)]
mod test {
	use crate::cards::{
		Hand,
		PokerHand,
	};

	fn high_6_test_hand() -> Hand {
		Hand::from(["ah", "2h", "3h", "4h", "6s"])
	}

	fn weird_straight_high_ace_test_hand() -> Hand {
		Hand::from(["jh", "qh", "kh", "ah", "2s"])
	}

	fn ace_pair_test_hand() -> Hand {
		Hand::from(["ah", "as", "3h", "4h", "6s"])
	}

	fn ace_three_two_pair_test_hand() -> Hand {
		Hand::from(["ah", "as", "3h", "3s", "6s"])
	}

	fn three_aces_test_hand() -> Hand {
		Hand::from(["ah", "as", "ac", "4h", "5h"])
	}

	fn ace_to_five_straight_test_hand() -> Hand {
		Hand::from(["as", "2h", "3h", "4h", "5h"])
	}

	fn ten_to_ace_straight_test_hand() -> Hand {
		Hand::from(["ts", "jh", "qh", "kh", "ah"])
	}

	fn heart_flush_test_hand() -> Hand {
		Hand::from(["ah", "2h", "3h", "4h", "6h"])
	}

	fn spade_flush_test_hand() -> Hand {
		Hand::from(["as", "3s", "5s", "7s", "9s"])
	}

	fn ace_four_full_house_test_hand() -> Hand {
		Hand::from(["ah", "as", "ac", "4h", "4s"])
	}

	fn four_aces_test_hand() -> Hand {
		Hand::from(["ah", "as", "ac", "ad", "6h"])
	}

	fn ace_to_five_heart_straight_flush_test_hand() -> Hand {
		Hand::from(["ah", "2h", "3h", "4h", "5h"])
	}

	fn ten_to_ace_heart_straight_flush_test_hand() -> Hand {
		Hand::from(["th", "jh", "qh", "kh", "ah"])
	}

	#[test]
	fn high_card_tests() {
		assert!(
			high_6_test_hand().is_poker_hand(PokerHand::HighCard),
			"high 6 card"
		);
		assert!(
			!ace_pair_test_hand().is_poker_hand(PokerHand::HighCard),
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
		assert!(
			ace_pair_test_hand().is_poker_hand(PokerHand::Pair),
			"ace pair"
		);
		assert!(
			!high_6_test_hand().is_poker_hand(PokerHand::Pair),
			"high card is not a pair"
		);
		assert!(
			!three_aces_test_hand().is_poker_hand(PokerHand::Pair),
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
			ace_three_two_pair_test_hand().is_poker_hand(PokerHand::TwoPair),
			"ace & 3 two pair"
		);
		assert!(
			!three_aces_test_hand().is_poker_hand(PokerHand::TwoPair),
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
			three_aces_test_hand().is_poker_hand(PokerHand::ThreeOfAKind),
			"three aces of a kind"
		);
		assert!(
			!ace_pair_test_hand().is_poker_hand(PokerHand::ThreeOfAKind),
			"pair is not a three of a kind"
		);
		assert!(
			!ace_three_two_pair_test_hand()
				.is_poker_hand(PokerHand::ThreeOfAKind),
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
			!high_6_test_hand().is_poker_hand(PokerHand::Straight),
			"high card is not a straight"
		);
		assert!(
			!weird_straight_high_ace_test_hand()
				.is_poker_hand(PokerHand::Straight),
			"weird straight (jkqa2) is not a valid straight"
		);
		assert!(
			!ace_pair_test_hand().is_poker_hand(PokerHand::ThreeOfAKind),
			"pair is not a straight"
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
			!high_6_test_hand().is_poker_hand(PokerHand::Flush),
			"high card is not a flush"
		);
		assert!(
			!ace_pair_test_hand().is_poker_hand(PokerHand::Flush),
			"pair is not a straight"
		);
	}

	#[test]
	pub fn full_house_tests() {
		assert!(
			ace_four_full_house_test_hand().is_poker_hand(PokerHand::FullHouse),
			"ace four full house"
		);
		assert!(
			!ace_three_two_pair_test_hand().is_poker_hand(PokerHand::FullHouse),
			"two pair is not a full house"
		);
		assert!(
			!three_aces_test_hand().is_poker_hand(PokerHand::FullHouse),
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
}
