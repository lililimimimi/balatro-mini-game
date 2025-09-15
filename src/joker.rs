use ortalib::{Card, Chips, JokerCard, Mult, Suit};
use std::collections::HashMap;

use crate::modifiers;

pub enum JokerActivation {
    OnScored,
    OnHeld,
    Independent,
}

#[derive(Debug, PartialEq)]
pub enum ScoringScope {
    BestHand,
    AllPlayed,
    Custom(Vec<Card>),
}
pub trait JokerEffect {
    fn name(&self) -> &'static str;
    fn activation_type(&self) -> JokerActivation;
    fn apply(
        &self,
        chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        context: &JokerContext,
    ) -> bool;
    fn scoring_scope(&self, _context: &JokerContext) -> ScoringScope {
        ScoringScope::BestHand
    }
    fn supports_retrigger(&self) -> bool {
        false
    }
    fn is_passive(&self) -> bool {
        false
    }
    fn copy_effect(
        &self,
        _chips: &mut Chips,
        _mult: &mut Mult,
        _context: &JokerContext,
    ) -> Option<String> {
        None
    }
    fn is_copyable(&self) -> bool {
        true
    }
    fn preferred_scoring_scope(&self, _context: &JokerContext) -> Option<ScoringScope> {
        None
    }
}

pub struct JokerContext<'a> {
    pub cards_played: &'a [Card],
    pub cards_in_hand: &'a [Card],
    pub best_hand_name: Option<&'a str>,
    pub all_jokers: &'a [JokerCard],
}
impl<'a> JokerContext<'a> {
    pub fn new(
        cards_played: &'a [Card],
        cards_in_hand: &'a [Card],
        best_hand_name: Option<&'a str>,
        all_jokers: &'a [JokerCard],
    ) -> Self {
        JokerContext {
            cards_played,
            cards_in_hand,
            best_hand_name,
            all_jokers,
        }
    }
    pub fn is_face_card(&self, card: &Card) -> bool {
        if self
            .all_jokers
            .iter()
            .any(|joker| matches!(joker.joker, ortalib::Joker::Pareidolia))
        {
            return true;
        }

        card.rank.is_face()
    }
    pub fn with_modified_suits(&self) -> Vec<Card> {
        let has_smeared_joker = self
            .all_jokers
            .iter()
            .any(|j| matches!(j.joker, ortalib::Joker::SmearedJoker));

        if has_smeared_joker {
            self.cards_played
                .iter()
                .map(|card| {
                    let mut new_card = *card;
                    match new_card.suit {
                        Suit::Diamonds => new_card.suit = Suit::Hearts,
                        Suit::Clubs => new_card.suit = Suit::Spades,
                        _ => {}
                    }
                    new_card
                })
                .collect()
        } else {
            self.cards_played.to_vec()
        }
    }
}
pub struct BasicJoker;

impl JokerEffect for BasicJoker {
    fn name(&self) -> &'static str {
        "Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        *mult += 4.0;
        false
    }
}

pub struct JollyJoker;

impl JokerEffect for JollyJoker {
    fn name(&self) -> &'static str {
        "Jolly Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        let mut rank_counts = HashMap::new();
        for card in context.cards_played {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        if rank_counts.values().any(|&count| count >= 2) {
            *mult += 8.0;
            return true;
        }

        false
    }
}

pub struct ZanyJoker;

impl JokerEffect for ZanyJoker {
    fn name(&self) -> &'static str {
        "Zany Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        let mut rank_counts = HashMap::new();
        for card in context.cards_played {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        if rank_counts.values().any(|&count| count >= 3) {
            *mult += 12.0;
            return true;
        }

        false
    }
}

pub struct MadJoker;

impl JokerEffect for MadJoker {
    fn name(&self) -> &'static str {
        "Mad Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        let mut rank_counts = HashMap::new();
        for card in context.cards_played {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        let pairs_count = rank_counts.values().filter(|&&count| count >= 2).count();

        if pairs_count >= 2 {
            *mult += 10.0;
            return true;
        }

        false
    }
}

pub struct CrazyJoker;

impl JokerEffect for CrazyJoker {
    fn name(&self) -> &'static str {
        "Crazy Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if has_straight(context.cards_played) {
            *mult += 12.0;
            return true;
        }

        false
    }
}
/// Checks if a straight is present in the cards.
fn has_straight(cards: &[Card]) -> bool {
    if cards.len() < 5 {
        return false;
    }

    let mut ranks: Vec<u8> = cards
        .iter()
        .map(|card| card.rank.rank_value() as u8)
        .collect();

    ranks.sort_unstable();
    ranks.dedup();

    let mut consecutive_count = 1;
    let mut max_consecutive = 1;

    for i in 1..ranks.len() {
        if ranks[i] == ranks[i - 1] + 1 {
            consecutive_count += 1;
            max_consecutive = max_consecutive.max(consecutive_count);
        } else if ranks[i] != ranks[i - 1] {
            consecutive_count = 1;
        }
    }

    if ranks.contains(&14) {
        let mut low_ace_ranks = vec![1];
        low_ace_ranks.extend(ranks.iter().filter(|&&r| r <= 5).copied());
        low_ace_ranks.sort_unstable();
        low_ace_ranks.dedup();

        consecutive_count = 1;
        for i in 1..low_ace_ranks.len() {
            if low_ace_ranks[i] == low_ace_ranks[i - 1] + 1 {
                consecutive_count += 1;
                max_consecutive = max_consecutive.max(consecutive_count);
            } else if low_ace_ranks[i] != low_ace_ranks[i - 1] {
                consecutive_count = 1;
            }
        }
    }

    max_consecutive >= 5
}

pub struct DrollJoker;

impl JokerEffect for DrollJoker {
    fn name(&self) -> &'static str {
        "Droll Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if has_flush(context.cards_played) {
            *mult += 10.0;
            return true;
        }

        false
    }
}
/// Checks if a flush is present in the cards.
fn has_flush(cards: &[Card]) -> bool {
    if cards.len() < 5 {
        return false;
    }

    let mut suit_counts = HashMap::new();
    for card in cards {
        *suit_counts.entry(card.suit).or_insert(0) += 1;
    }

    suit_counts.values().any(|&count| count >= 5)
}

pub struct SlyJoker;

impl JokerEffect for SlyJoker {
    fn name(&self) -> &'static str {
        "Sly Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        let mut rank_counts = HashMap::new();
        for card in context.cards_played {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        if rank_counts.values().any(|&count| count >= 2) {
            *chips += 50.0;
            return true;
        }

        false
    }
}
pub struct WilyJoker;

impl JokerEffect for WilyJoker {
    fn name(&self) -> &'static str {
        "Wily Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        let mut rank_counts = HashMap::new();
        for card in context.cards_played {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        if rank_counts.values().any(|&count| count >= 3) {
            *chips += 100.0;
            return true;
        }

        false
    }
}

pub struct CleverJoker;

impl JokerEffect for CleverJoker {
    fn name(&self) -> &'static str {
        "Clever Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        let mut rank_counts = HashMap::new();
        for card in context.cards_played {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        let pairs_count = rank_counts.values().filter(|&&count| count == 2).count();

        if pairs_count >= 2 {
            *chips += 80.0;
            return true;
        }

        false
    }
}

pub struct DeviousJoker;

impl JokerEffect for DeviousJoker {
    fn name(&self) -> &'static str {
        "Devious Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if has_straight(context.cards_played) {
            *chips += 100.0;
            return true;
        }

        false
    }
}
pub struct CraftyJoker;

impl JokerEffect for CraftyJoker {
    fn name(&self) -> &'static str {
        "Crafty Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if has_flush(context.cards_played) {
            *chips += 80.0;
            return true;
        }

        false
    }
}

pub struct AbstractJoker;

impl JokerEffect for AbstractJoker {
    fn name(&self) -> &'static str {
        "Abstract Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        let joker_count = context.all_jokers.len();
        *mult += 3.0 * joker_count as f64;
        true
    }
}

pub struct RaisedFistJoker;

impl JokerEffect for RaisedFistJoker {
    fn name(&self) -> &'static str {
        "Raised Fist"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnHeld
    }
    fn supports_retrigger(&self) -> bool {
        false
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if let Some(lowest_card) = find_lowest_rank_card(context.cards_in_hand) {
                if card == lowest_card {
                    let bonus_value = match card.rank {
                        ortalib::Rank::Jack | ortalib::Rank::Queen | ortalib::Rank::King => 20.0,
                        ortalib::Rank::Ace => 22.0,
                        _ => card.rank.rank_value() * 2.0,
                    };
                    *mult += bonus_value;
                    return true;
                }
            }
        }
        false
    }
}

/// Finds the card with the lowest rank in a set.
fn find_lowest_rank_card(cards: &[Card]) -> Option<&Card> {
    if cards.is_empty() {
        return None;
    }

    let mut lowest_cards = Vec::new();
    let mut lowest_value = f64::MAX;

    for card in cards {
        let value = get_card_numeric_value(card);

        if value < lowest_value {
            lowest_value = value;
            lowest_cards.clear();
            lowest_cards.push(card);
        } else if value == lowest_value {
            lowest_cards.push(card);
        }
    }

    lowest_cards.last().copied()
}

/// Gets the numeric value of a card's rank.
fn get_card_numeric_value(card: &Card) -> f64 {
    card.rank.rank_value()
}
pub struct BlackboardJoker;

impl JokerEffect for BlackboardJoker {
    fn name(&self) -> &'static str {
        "Blackboard"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if context.cards_in_hand.is_empty() {
            *mult *= 3.0;
            return true;
        }

        let all_black = context.cards_in_hand.iter().all(|card| {
            matches!(card.suit, ortalib::Suit::Spades | ortalib::Suit::Clubs)
                || matches!(card.enhancement, Some(ortalib::Enhancement::Wild))
        });

        if all_black {
            *mult *= 3.0;
            return true;
        }

        false
    }
}

pub struct BaronJoker;

impl JokerEffect for BaronJoker {
    fn name(&self) -> &'static str {
        "Baron"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnHeld
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if context.is_face_card(card) && card.rank == ortalib::Rank::King {
                *mult *= 1.5;
                return true;
            }
        }
        false
    }
}

pub struct GreedyJoker;

impl JokerEffect for GreedyJoker {
    fn name(&self) -> &'static str {
        "Greedy Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if matches!(card.suit, ortalib::Suit::Diamonds)
                || matches!(card.enhancement, Some(ortalib::Enhancement::Wild))
            {
                *mult += 3.0;
                return true;
            }
        }

        false
    }
}

pub struct LustyJoker;

impl JokerEffect for LustyJoker {
    fn name(&self) -> &'static str {
        "Lusty Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if matches!(card.suit, ortalib::Suit::Hearts)
                || matches!(card.enhancement, Some(ortalib::Enhancement::Wild))
            {
                *mult += 3.0;
                return true;
            }
        }

        false
    }
}

pub struct WrathfulJoker;

impl JokerEffect for WrathfulJoker {
    fn name(&self) -> &'static str {
        "Wrathful Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if matches!(card.suit, ortalib::Suit::Spades)
                || matches!(card.enhancement, Some(ortalib::Enhancement::Wild))
            {
                *mult += 3.0;
                return true;
            }
        }

        false
    }
}

pub struct GluttonousJoker;

impl JokerEffect for GluttonousJoker {
    fn name(&self) -> &'static str {
        "Gluttonous Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if matches!(card.suit, ortalib::Suit::Clubs)
                || matches!(card.enhancement, Some(ortalib::Enhancement::Wild))
            {
                *mult += 3.0;
                return true;
            }
        }

        false
    }
}

pub struct FibonacciJoker;

impl JokerEffect for FibonacciJoker {
    fn name(&self) -> &'static str {
        "Fibonacci"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            let is_fibonacci = matches!(
                card.rank,
                ortalib::Rank::Ace
                    | ortalib::Rank::Two
                    | ortalib::Rank::Three
                    | ortalib::Rank::Five
                    | ortalib::Rank::Eight
            );

            if is_fibonacci {
                *mult += 8.0;
                return true;
            }
        }

        false
    }
}

pub struct ScaryFaceJoker;

impl JokerEffect for ScaryFaceJoker {
    fn name(&self) -> &'static str {
        "Scary Face"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        chips: &mut Chips,
        _mult: &mut Mult,
        card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if context.is_face_card(card) {
                *chips += 30.0;
                return true;
            }
        }

        false
    }
}

pub struct EvenStevenJoker;

impl JokerEffect for EvenStevenJoker {
    fn name(&self) -> &'static str {
        "Even Steven"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            let rank_value = card.rank.rank_value() as u8;
            if rank_value % 2 == 0 && (2..=10).contains(&rank_value) {
                let bonus = match rank_value {
                    10 => 4.0,
                    8 => 4.0,
                    6 => 4.0,
                    4 => 4.0,
                    2 => 4.0,
                    _ => 0.0,
                };
                *mult += bonus;
                return true;
            }
        }

        false
    }
}

pub struct OddToddJoker;

impl JokerEffect for OddToddJoker {
    fn name(&self) -> &'static str {
        "Odd Todd"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        chips: &mut Chips,
        _mult: &mut Mult,
        card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            let rank_value = card.rank.rank_value() as u8;

            if (rank_value % 2 == 1 && (3..=9).contains(&rank_value))
                || matches!(card.rank, ortalib::Rank::Ace)
            {
                *chips += 31.0;
                return true;
            }
        }

        false
    }
}

pub struct PhotographJoker;

impl JokerEffect for PhotographJoker {
    fn name(&self) -> &'static str {
        "Photograph"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if context.is_face_card(card) {
                if let Some(card_index) = context.cards_played.iter().position(|c| c == card) {
                    let previous_face_cards = context
                        .cards_played
                        .iter()
                        .take(card_index)
                        .filter(|c| c.rank.is_face())
                        .count();

                    if previous_face_cards == 0 {
                        *mult *= 2.0;
                        return true;
                    }
                }
            }
        }

        false
    }
}

pub struct SmileyFaceJoker;

impl JokerEffect for SmileyFaceJoker {
    fn name(&self) -> &'static str {
        "Smiley Face"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if context.is_face_card(card) {
                *mult += 5.0;
                return true;
            }
        }

        false
    }
}

pub struct FlowerPotJoker;
impl JokerEffect for FlowerPotJoker {
    fn name(&self) -> &'static str {
        "Flower Pot"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if context.cards_played.len() < 4 {
            return false;
        }

        let has_smeared_joker = context
            .all_jokers
            .iter()
            .any(|j| matches!(j.joker, ortalib::Joker::SmearedJoker));

        let cards = context.cards_played;
        let mut diamonds_count = 0;
        let mut clubs_count = 0;
        let mut hearts_count = 0;
        let mut spades_count = 0;
        let mut _wild_cards = 0;

        for card in cards {
            match card.suit {
                ortalib::Suit::Diamonds => diamonds_count += 1,
                ortalib::Suit::Clubs => clubs_count += 1,
                ortalib::Suit::Hearts => hearts_count += 1,
                ortalib::Suit::Spades => spades_count += 1,
            }
            if matches!(card.enhancement, Some(ortalib::Enhancement::Wild)) {
                _wild_cards += 1;
            }
        }

        let has_red_suits = diamonds_count > 0 || hearts_count > 0;
        let has_black_suits = clubs_count > 0 || spades_count > 0;

        let mut unique_suit_groups = 0;
        if has_red_suits {
            unique_suit_groups += 1;
        }
        if has_black_suits {
            unique_suit_groups += 1;
        }

        let mut unique_suits = 0;
        if diamonds_count > 0 {
            unique_suits += 1;
        }
        if clubs_count > 0 {
            unique_suits += 1;
        }
        if hearts_count > 0 {
            unique_suits += 1;
        }
        if spades_count > 0 {
            unique_suits += 1;
        }

        if has_smeared_joker {
            if hearts_count > 0 && unique_suit_groups >= 2 {
                *mult *= 3.0;
                return true;
            }
        } else if unique_suits >= 4 {
                *mult *= 3.0;
                return true; 
            
        }

        false
    }
}

pub struct FourFingersJoker;

impl JokerEffect for FourFingersJoker {
    fn name(&self) -> &'static str {
        "Four Fingers"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        false
    }

    fn is_copyable(&self) -> bool {
        false
    }
    fn preferred_scoring_scope(&self, context: &JokerContext) -> Option<ScoringScope> {
        let has_royal_cards = context
            .cards_played
            .iter()
            .any(|card| card.rank == ortalib::Rank::Ace)
            && context
                .cards_played
                .iter()
                .any(|card| card.rank == ortalib::Rank::King)
            && context
                .cards_played
                .iter()
                .any(|card| card.rank == ortalib::Rank::Queen)
            && context
                .cards_played
                .iter()
                .any(|card| card.rank == ortalib::Rank::Jack)
            && context
                .cards_played
                .iter()
                .any(|card| card.rank == ortalib::Rank::Ten);

        let has_shortcut = context
            .all_jokers
            .iter()
            .any(|joker| matches!(joker.joker, ortalib::Joker::Shortcut));

        if context.best_hand_name.is_some() && (has_royal_cards || has_shortcut) {
            return Some(ScoringScope::AllPlayed);
        }

        None
    }
}

pub struct ShortcutJoker;

impl JokerEffect for ShortcutJoker {
    fn name(&self) -> &'static str {
        "Shortcut"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        _context: &JokerContext,
    ) -> bool {
        true
    }
    fn is_copyable(&self) -> bool {
        false
    }
}
pub struct MimeJoker;

impl JokerEffect for MimeJoker {
    fn name(&self) -> &'static str {
        "Mime"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnHeld
    }

    fn supports_retrigger(&self) -> bool {
        false
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            let mut processed_types = std::collections::HashSet::new();
            processed_types.insert(get_joker_id(&ortalib::Joker::Mime));

            for joker in context.all_jokers {
                let joker_id = get_joker_id(&joker.joker);

                if !processed_types.insert(joker_id) {
                    continue;
                }

                let effect = JokerFactory::create_joker(&joker.joker);

                if matches!(effect.activation_type(), JokerActivation::OnHeld) {
                    effect.apply(_chips, mult, Some(card), context);
                }
            }
            return true;
        }
        false
    }
}
pub struct PareidoliaJoker;

impl JokerEffect for PareidoliaJoker {
    fn name(&self) -> &'static str {
        "Pareidolia"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        !context.cards_played.is_empty() || !context.cards_in_hand.is_empty()
    }

    fn is_copyable(&self) -> bool {
        false
    }
}

pub struct SplashJoker;

impl JokerEffect for SplashJoker {
    fn name(&self) -> &'static str {
        "Splash"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        !context.cards_played.is_empty()
    }

    fn scoring_scope(&self, context: &JokerContext) -> ScoringScope {
        if !context.cards_played.is_empty() {
            ScoringScope::AllPlayed
        } else {
            ScoringScope::BestHand
        }
    }
    fn is_copyable(&self) -> bool {
        false
    }
}

pub struct SockAndBuskinJoker;
impl JokerEffect for SockAndBuskinJoker {
    fn name(&self) -> &'static str {
        "Sock and Buskin"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::OnScored
    }

    fn apply(
        &self,
        chips: &mut Chips,
        mult: &mut Mult,
        card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if let Some(card) = card {
            if context.is_face_card(card) {
                let value = card.rank.rank_value();
                *chips += value;

                if let Some(enhancement_type) = &card.enhancement {
                    let enhancement = modifiers::create_enhancement_handler(enhancement_type);
                    enhancement.apply(chips, mult, card, false);
                }

                if let Some(edition_type) = &card.edition {
                    let edition = modifiers::create_edition_handler(edition_type);
                    edition.apply(chips, mult, card);
                }

                for joker in context.all_jokers {
                    let joker_effect = JokerFactory::create_joker(&joker.joker);
                    if matches!(joker_effect.activation_type(), JokerActivation::OnScored)
                        && joker_effect.name() != self.name()
                    {
                        joker_effect.apply(chips, mult, Some(card), context);
                    }
                }
                return true;
            }
        }
        false
    }
    fn supports_retrigger(&self) -> bool {
        false
    }
}

pub struct SmearedJoker;

impl JokerEffect for SmearedJoker {
    fn name(&self) -> &'static str {
        "Smeared Joker"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        _chips: &mut Chips,
        _mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        !context.cards_played.is_empty()
    }

    fn is_copyable(&self) -> bool {
        false
    }
}

pub struct BlueprintJoker;

impl JokerEffect for BlueprintJoker {
    fn name(&self) -> &'static str {
        "Blueprint"
    }

    fn activation_type(&self) -> JokerActivation {
        JokerActivation::Independent
    }

    fn apply(
        &self,
        chips: &mut Chips,
        mult: &mut Mult,
        _card: Option<&Card>,
        context: &JokerContext,
    ) -> bool {
        if let Some(_copied_joker_name) = self.copy_effect(chips, mult, context) {
            !context.cards_played.is_empty()
        } else {
            false
        }
    }
    /// Copies the effect of the next applicable joker.   
    fn copy_effect(
        &self,
        chips: &mut Chips,
        mult: &mut Mult,
        context: &JokerContext,
    ) -> Option<String> {
        let mut current_index = None;
        for (i, joker) in context.all_jokers.iter().enumerate() {
            let joker_effect = JokerFactory::create_joker(&joker.joker);
            if joker_effect.name() == self.name() {
                current_index = Some(i);
                break;
            }
        }

        if let Some(mut index) = current_index {
            let mut target_joker_effect = None;
            while index + 1 < context.all_jokers.len() {
                index += 1;
                let next_joker = &context.all_jokers[index];
                let next_joker_effect = JokerFactory::create_joker(&next_joker.joker);

                if next_joker_effect.is_passive() {
                    continue;
                }

                if next_joker_effect.name() == self.name() {
                    continue;
                }

                target_joker_effect = Some(next_joker_effect);
                break;
            }

            if let Some(joker_effect) = target_joker_effect {
                match joker_effect.activation_type() {
                    JokerActivation::OnScored => {
                        let mut applied_any = false;
                        for card in context.cards_played {
                            if joker_effect.apply(chips, mult, Some(card), context) {
                                applied_any = true;
                            }
                        }
                        if applied_any {
                            return Some(joker_effect.name().to_string());
                        }
                    }
                    JokerActivation::OnHeld => {
                        let mut applied_any = false;
                        for card in context.cards_in_hand {
                            if joker_effect.apply(chips, mult, Some(card), context) {
                                applied_any = true;
                            }
                        }
                        if applied_any {
                            return Some(joker_effect.name().to_string());
                        }
                    }
                    JokerActivation::Independent => {
                        let applied = joker_effect.apply(chips, mult, None, context);
                        if applied {
                            return Some(joker_effect.name().to_string());
                        }
                    }
                }
            }
        }
        None
    }
}

pub struct JokerFactory;

/// Creates a joker effect instance based on joker type.
impl JokerFactory {
    pub fn create_joker(joker_type: &ortalib::Joker) -> Box<dyn JokerEffect> {
        match joker_type {
            ortalib::Joker::Joker => Box::new(BasicJoker),
            ortalib::Joker::JollyJoker => Box::new(JollyJoker),
            ortalib::Joker::ZanyJoker => Box::new(ZanyJoker),
            ortalib::Joker::MadJoker => Box::new(MadJoker),
            ortalib::Joker::CrazyJoker => Box::new(CrazyJoker),
            ortalib::Joker::DrollJoker => Box::new(DrollJoker),
            ortalib::Joker::SlyJoker => Box::new(SlyJoker),
            ortalib::Joker::WilyJoker => Box::new(WilyJoker),
            ortalib::Joker::CleverJoker => Box::new(CleverJoker),
            ortalib::Joker::DeviousJoker => Box::new(DeviousJoker),
            ortalib::Joker::CraftyJoker => Box::new(CraftyJoker),
            ortalib::Joker::AbstractJoker => Box::new(AbstractJoker),
            ortalib::Joker::RaisedFist => Box::new(RaisedFistJoker),
            ortalib::Joker::Blackboard => Box::new(BlackboardJoker),
            ortalib::Joker::Baron => Box::new(BaronJoker),
            ortalib::Joker::GreedyJoker => Box::new(GreedyJoker),
            ortalib::Joker::LustyJoker => Box::new(LustyJoker),
            ortalib::Joker::WrathfulJoker => Box::new(WrathfulJoker),
            ortalib::Joker::GluttonousJoker => Box::new(GluttonousJoker),
            ortalib::Joker::Fibonacci => Box::new(FibonacciJoker),
            ortalib::Joker::ScaryFace => Box::new(ScaryFaceJoker),
            ortalib::Joker::EvenSteven => Box::new(EvenStevenJoker),
            ortalib::Joker::OddTodd => Box::new(OddToddJoker),
            ortalib::Joker::Photograph => Box::new(PhotographJoker),
            ortalib::Joker::SmileyFace => Box::new(SmileyFaceJoker),
            ortalib::Joker::FlowerPot => Box::new(FlowerPotJoker),
            ortalib::Joker::FourFingers => Box::new(FourFingersJoker),
            ortalib::Joker::Shortcut => Box::new(ShortcutJoker),
            ortalib::Joker::Mime => Box::new(MimeJoker),
            ortalib::Joker::Pareidolia => Box::new(PareidoliaJoker),
            ortalib::Joker::Splash => Box::new(SplashJoker),
            ortalib::Joker::SockAndBuskin => Box::new(SockAndBuskinJoker),
            ortalib::Joker::SmearedJoker => Box::new(SmearedJoker),
            ortalib::Joker::Blueprint => Box::new(BlueprintJoker),
        }
    }
}

/// Applies independent joker effects to chips and mult.
pub fn apply_joker_effects(
    jokers: &[JokerCard],
    chips: &mut Chips,
    mult: &mut Mult,
    context: &JokerContext,
) {
    let mut processed_joker_types = std::collections::HashSet::new();

    for joker in jokers {
        let joker_id = get_joker_id(&joker.joker);

        if !processed_joker_types.insert(joker_id) {
            continue;
        }

        let joker_effect = JokerFactory::create_joker(&joker.joker);
        if matches!(joker_effect.activation_type(), JokerActivation::Independent) {
            joker_effect.apply(chips, mult, None, context);
        }
    }
}

/// Returns a unique ID for a joker type.
pub fn get_joker_id(joker: &ortalib::Joker) -> u32 {
    match joker {
        ortalib::Joker::Joker => 1,
        ortalib::Joker::JollyJoker => 2,
        ortalib::Joker::ZanyJoker => 3,
        ortalib::Joker::MadJoker => 4,
        ortalib::Joker::CrazyJoker => 5,
        ortalib::Joker::DrollJoker => 6,
        ortalib::Joker::SlyJoker => 7,
        ortalib::Joker::WilyJoker => 8,
        ortalib::Joker::CleverJoker => 9,
        ortalib::Joker::DeviousJoker => 10,
        ortalib::Joker::CraftyJoker => 11,
        ortalib::Joker::AbstractJoker => 12,
        ortalib::Joker::RaisedFist => 13,
        ortalib::Joker::Blackboard => 14,
        ortalib::Joker::Baron => 15,
        ortalib::Joker::GreedyJoker => 16,
        ortalib::Joker::LustyJoker => 17,
        ortalib::Joker::WrathfulJoker => 18,
        ortalib::Joker::GluttonousJoker => 19,
        ortalib::Joker::Fibonacci => 20,
        ortalib::Joker::ScaryFace => 21,
        ortalib::Joker::EvenSteven => 22,
        ortalib::Joker::OddTodd => 23,
        ortalib::Joker::Photograph => 24,
        ortalib::Joker::SmileyFace => 25,
        ortalib::Joker::FlowerPot => 26,
        ortalib::Joker::FourFingers => 27,
        ortalib::Joker::Shortcut => 28,
        ortalib::Joker::Mime => 29,
        ortalib::Joker::Pareidolia => 30,
        ortalib::Joker::Splash => 31,
        ortalib::Joker::SockAndBuskin => 32,
        ortalib::Joker::SmearedJoker => 33,
        ortalib::Joker::Blueprint => 34,
    }
}

/// Applies OnHeld joker effects for a specific card.
pub fn apply_onheld_joker_effects(
    card: &Card,
    jokers: &[JokerCard],
    chips: &mut Chips,
    mult: &mut Mult,
    context: &JokerContext,
) {
    let mut processed_joker_types = std::collections::HashSet::new();

    for joker in jokers {
        let joker_id = get_joker_id(&joker.joker);

        if !processed_joker_types.insert(joker_id) {
            continue;
        }

        let joker_effect = JokerFactory::create_joker(&joker.joker);
        if matches!(joker_effect.activation_type(), JokerActivation::OnHeld) {
            joker_effect.apply(chips, mult, Some(card), context);
        }
    }
}

/// Applies OnScored joker effects for a specific card.
pub fn apply_onscored_joker_effects(
    card: &Card,
    jokers: &[JokerCard],
    chips: &mut Chips,
    mult: &mut Mult,
    context: &JokerContext,
) {
    for joker in jokers {
        let joker_effect = JokerFactory::create_joker(&joker.joker);
        if matches!(joker_effect.activation_type(), JokerActivation::OnScored) {
            joker_effect.apply(chips, mult, Some(card), context);
        }
    }
}

/// Applies retriggerable joker effects for cards in hand.
pub fn apply_jokers_retrigger(
    jokers: &[JokerCard],
    cards_in_hand: &[Card],
    chips: &mut Chips,
    mult: &mut Mult,
    context: &JokerContext,
) {
    if cards_in_hand.is_empty() {
        return;
    }

    for card in cards_in_hand {
        let mut processed_joker_types = std::collections::HashSet::new();

        for joker_card in jokers {
            if matches!(joker_card.joker, ortalib::Joker::Mime) {
                continue;
            }

            let joker_id = get_joker_id(&joker_card.joker);

            if !processed_joker_types.insert(joker_id) {
                continue;
            }

            let effect = JokerFactory::create_joker(&joker_card.joker);

            if matches!(effect.activation_type(), JokerActivation::OnHeld) {
                effect.apply(chips, mult, Some(card), context);
            }
        }

        processed_joker_types.clear();
        for joker_card in jokers {
            if !matches!(joker_card.joker, ortalib::Joker::Mime) {
                continue;
            }

            let joker_id = get_joker_id(&joker_card.joker);

            if !processed_joker_types.insert(joker_id) {
                continue;
            }

            let effect = JokerFactory::create_joker(&joker_card.joker);

            if matches!(effect.activation_type(), JokerActivation::OnHeld) {
                effect.apply(chips, mult, Some(card), context);
            }
        }
    }
}
