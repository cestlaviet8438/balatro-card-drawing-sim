use crate::cards::Card;

trait Strategy {
	/// Returns which cards to discard.
	fn get_cards_to_discard(
		held: &[Card],
		discard_pile: &[Card],
		draw_pile: &[Card],
	) -> Vec<Card>;
}
