use ortalib::{
    Card, Chips, Edition as EditionType, Enhancement as EnhancementType, Mult, Rank, Suit,
};
use std::collections::HashMap;

pub trait Enhancement {
    fn apply(&self, chips: &mut Chips, mult: &mut Mult, card: &Card, is_held: bool);
    fn name(&self) -> &'static str;
}

pub struct BonusEnhancement;

impl Enhancement for BonusEnhancement {
    fn apply(&self, chips: &mut Chips, _mult: &mut Mult, _card: &Card, _is_held: bool) {
        *chips += 30.0;
    }

    fn name(&self) -> &'static str {
        "Bonus Card"
    }
}

pub struct MultEnhancement;

impl Enhancement for MultEnhancement {
    fn apply(&self, _chips: &mut Chips, mult: &mut Mult, _card: &Card, _is_held: bool) {
        *mult += 4.0;
    }

    fn name(&self) -> &'static str {
        "Mult Card"
    }
}

pub struct WildEnhancement;

impl Enhancement for WildEnhancement {
    fn apply(&self, _chips: &mut Chips, _mult: &mut Mult, _card: &Card, _is_held: bool) {}

    fn name(&self) -> &'static str {
        "Wild Card"
    }
}

pub struct GlassEnhancement;

impl Enhancement for GlassEnhancement {
    fn apply(&self, _chips: &mut Chips, mult: &mut Mult, _card: &Card, _is_held: bool) {
        *mult *= 2.0;
    }

    fn name(&self) -> &'static str {
        "Glass Card"
    }
}

pub struct SteelEnhancement;

impl Enhancement for SteelEnhancement {
    fn apply(&self, _chips: &mut Chips, mult: &mut Mult, _card: &Card, is_held: bool) {
        if is_held {
            *mult *= 1.5;
        }
    }

    fn name(&self) -> &'static str {
        "Steel Card"
    }
}

/// Creates an enhancement handler based on the enhancement type.
pub fn create_enhancement_handler(enhancement_type: &EnhancementType) -> Box<dyn Enhancement> {
    match enhancement_type {
        EnhancementType::Bonus => Box::new(BonusEnhancement),
        EnhancementType::Mult => Box::new(MultEnhancement),
        EnhancementType::Wild => Box::new(WildEnhancement),
        EnhancementType::Glass => Box::new(GlassEnhancement),
        EnhancementType::Steel => Box::new(SteelEnhancement),
    }
}

pub trait Edition {
    fn apply(&self, chips: &mut Chips, mult: &mut Mult, card: &Card);
    fn name(&self) -> &'static str;
}

pub struct FoilEdition;

impl Edition for FoilEdition {
    fn apply(&self, chips: &mut Chips, _mult: &mut Mult, _card: &Card) {
        *chips += 50.0;
    }

    fn name(&self) -> &'static str {
        "Foil"
    }
}

pub struct HolographicEdition;

impl Edition for HolographicEdition {
    fn apply(&self, _chips: &mut Chips, mult: &mut Mult, _card: &Card) {
        *mult += 10.0;
    }

    fn name(&self) -> &'static str {
        "Holographic"
    }
}

pub struct PolychromeEdition;

impl Edition for PolychromeEdition {
    fn apply(&self, _chips: &mut Chips, mult: &mut Mult, _card: &Card) {
        *mult *= 1.5;
    }

    fn name(&self) -> &'static str {
        "Polychrome"
    }
}

/// Creates an edition handler based on the edition type.
pub fn create_edition_handler(edition_type: &EditionType) -> Box<dyn Edition> {
    match edition_type {
        EditionType::Foil => Box::new(FoilEdition),
        EditionType::Holographic => Box::new(HolographicEdition),
        EditionType::Polychrome => Box::new(PolychromeEdition),
    }
}

/// Applies enhancements and editions to a set of cards, modifying chips and mult.
pub fn apply_enhancements(cards: &Vec<Card>, chips: &mut Chips, mult: &mut Mult, is_held: bool) {
    for card in cards {
        if let Some(enhancement_type) = &card.enhancement {
            let enhancement = create_enhancement_handler(enhancement_type);
            enhancement.apply(chips, mult, card, is_held);
        }

        if let Some(edition_type) = &card.edition {
            let edition = create_edition_handler(edition_type);
            edition.apply(chips, mult, card);
        }
    }
}

/// Handles wild cards by adjusting the card set, potentially forming a straight.
pub fn handle_wild(cards: &[Card]) -> Vec<Card> {
    let has_wild = cards
        .iter()
        .any(|card| matches!(card.enhancement, Some(EnhancementType::Wild)));

    if let Some(all_wild_result) = handle_all_wild_cards(cards) {
        return all_wild_result;
    }

    if let Some(wild_straight) = try_form_wild_straight(cards) {
        return wild_straight;
    }

    if !has_wild {
        return cards.to_vec();
    }

    cards.to_vec()
}

/// Generates all possible straight sequences of ranks.
fn get_possible_straight_sequences() -> Vec<Vec<Rank>> {
    let all_ranks = [Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace];
    let mut sequences = Vec::new();

    for i in 0..=all_ranks.len().saturating_sub(5) {
        sequences.push(all_ranks[i..i + 5].to_vec());
    }

    let low_straight = vec![Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five];
    if !sequences.contains(&low_straight) {
        sequences.push(low_straight);
    }

    sequences
}

/// Selects the best straight sequence from possible options.
fn select_best_straight_sequence() -> Option<Vec<Rank>> {
    let sequences = get_possible_straight_sequences();
    let mut best: Option<Vec<Rank>> = None;

    for seq in sequences {
        let mut desc = seq.clone();

        desc.sort_by(|a, b| b.cmp(a));

        best = match best {
            None => Some(desc),
            Some(current) => {
                if desc > current {
                    Some(desc)
                } else {
                    Some(current)
                }
            }
        }
    }

    best
}

/// Determines the most common suit among the cards.
fn select_best_suit(cards: &[Card]) -> Suit {
    let mut suit_counts = HashMap::new();
    for card in cards {
        *suit_counts.entry(card.suit).or_insert(0) += 1;
    }

    if !suit_counts.is_empty() {
        suit_counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(suit, _)| suit)
            .unwrap()
    } else {
        let all_suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

        all_suits[0]
    }
}

/// Converts a set of all wild cards into a straight sequence if possible.
pub fn handle_all_wild_cards(cards: &[Card]) -> Option<Vec<Card>> {
    let all_wild = cards
        .iter()
        .all(|card| matches!(card.enhancement, Some(EnhancementType::Wild)));

    if all_wild {
        if let Some(target_seq) = select_best_straight_sequence() {
            let selected_suit = select_best_suit(cards);
            let mut result = Vec::new();

            for r in target_seq.iter() {
                let mut card = cards[0];
                card.rank = *r;
                card.suit = selected_suit;
                card.enhancement = None;
                result.push(card);
            }

            if result.len() == 5 {
                return Some(result);
            }
        }
    }

    None
}

/// Attempts to form a straight using wild cards to fill gaps.
pub fn try_form_wild_straight(cards: &[Card]) -> Option<Vec<Card>> {
    let normal_cards: Vec<Card> = cards
        .iter()
        .filter(|card| !matches!(card.enhancement, Some(EnhancementType::Wild)))
        .cloned()
        .collect();

    let wild_cards: Vec<Card> = cards
        .iter()
        .filter(|card| matches!(card.enhancement, Some(EnhancementType::Wild)))
        .cloned()
        .collect();

    let expected = select_best_straight_sequence()?;
    let target_suit = select_best_suit(&normal_cards);

    let mut result = Vec::new();
    let mut missing = Vec::new();

    let mut normals = normal_cards.clone();
    for exp in expected.iter() {
        if let Some(pos) = normals.iter().position(|card| card.rank == *exp) {
            let chosen = normals.remove(pos);
            if chosen.suit == target_suit {
                result.push(chosen);
            } else {
                missing.push(*exp);
            }
        } else {
            missing.push(*exp);
        }
    }

    if missing.len() <= wild_cards.len() {
        for (i, exp) in missing.iter().enumerate() {
            let mut wild_replacement = wild_cards[i];
            wild_replacement.rank = *exp;
            wild_replacement.suit = target_suit;
            wild_replacement.enhancement = None;
            result.push(wild_replacement);
        }

        if result.len() == 5 {
            result.sort_by(|a, b| b.rank.cmp(&a.rank));
            return Some(result);
        }
    }

    None
}

/// Applies an edition effect directly to chips and mult based on the edition type.
pub fn apply_edition_effect(edition_type: &EditionType, chips: &mut Chips, mult: &mut Mult) {
    match edition_type {
        EditionType::Foil => *chips += 50.0,
        EditionType::Holographic => *mult += 10.0,
        EditionType::Polychrome => *mult *= 1.5,
    }
}
