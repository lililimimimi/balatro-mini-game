use std::collections::HashMap;

use ortalib::{Card, Chips, JokerCard, Mult, Rank, Suit};

pub trait HandEvaluator {
    fn evaluate(&self, cards: &[Card], jokers: &[JokerCard]) -> bool;
    fn get_cards<'a>(&self, cards: &'a [Card], jokers: &[JokerCard]) -> Vec<&'a Card>;
    fn name(&self) -> &'static str;
    fn value(&self) -> (Chips, Mult);
}

/// Converts a rank to its numerical order for comparison.
fn rank_to_order(rank: Rank) -> u8 {
    match rank {
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
        Rank::Ace => 14,
    }
}
pub struct HighCard;

impl HandEvaluator for HighCard {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        !cards.is_empty()
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        if cards.is_empty() {
            return Vec::new();
        }
        let mut sorted_cards: Vec<&Card> = cards.iter().collect();
        sorted_cards.sort_by(|a, b| b.rank.cmp(&a.rank));
        vec![sorted_cards[0]]
    }

    fn name(&self) -> &'static str {
        "High Card"
    }

    fn value(&self) -> (Chips, Mult) {
        (5.0, 1.0)
    }
}
pub struct TwoPair;

impl HandEvaluator for TwoPair {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        let mut counts = HashMap::new();
        for card in cards {
            *counts.entry(card.rank).or_insert(0) += 1;
        }
        let pair_count = counts.values().filter(|&&count| count >= 2).count();
        pair_count >= 2
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        let mut groups: HashMap<Rank, Vec<&Card>> = HashMap::new();
        for card in cards {
            groups.entry(card.rank).or_default().push(card);
        }
        let mut pairs: Vec<(&Rank, &Vec<&Card>)> = groups
            .iter()
            .filter(|(_, group)| group.len() >= 2)
            .collect();
        pairs.sort_by(|(rank_a, _), (rank_b, _)| rank_b.cmp(rank_a));

        let mut result = Vec::new();

        for (_, group) in pairs.into_iter().take(2) {
            result.extend(group.iter().take(2));
        }
        result
    }

    fn name(&self) -> &'static str {
        "Two Pair"
    }

    fn value(&self) -> (Chips, Mult) {
        (20.0, 2.0)
    }
}

pub struct Pair;

impl HandEvaluator for Pair {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        let mut counts = HashMap::new();
        for card in cards {
            *counts.entry(card.rank).or_insert(0) += 1;
        }
        counts.values().any(|&count| count >= 2)
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        let mut groups: HashMap<Rank, Vec<&Card>> = HashMap::new();
        for card in cards {
            groups.entry(card.rank).or_default().push(card);
        }
        let mut pair_ranks: Vec<Rank> = groups
            .iter()
            .filter(|(_, group)| group.len() >= 2)
            .map(|(&rank, _)| rank)
            .collect();
        pair_ranks.sort_by(|a, b| b.cmp(a));
        if pair_ranks.is_empty() {
            return Vec::new();
        }
        let best_pair = pair_ranks[0];
        groups
            .get(&best_pair)
            .map(|group| group.iter().take(2).cloned().collect())
            .unwrap_or_default()
    }

    fn name(&self) -> &'static str {
        "Pair"
    }

    fn value(&self) -> (Chips, Mult) {
        (10.0, 2.0)
    }
}

pub struct ThreeOfAKind;

impl HandEvaluator for ThreeOfAKind {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        let mut counts = HashMap::new();
        for card in cards {
            *counts.entry(card.rank).or_insert(0) += 1;
        }
        counts.values().any(|&count| count >= 3)
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        let mut groups: HashMap<Rank, Vec<&Card>> = HashMap::new();
        for card in cards {
            groups.entry(card.rank).or_default().push(card);
        }

        let mut triple_ranks: Vec<Rank> = groups
            .iter()
            .filter(|(_, group)| group.len() >= 3)
            .map(|(&rank, _)| rank)
            .collect();
        triple_ranks.sort_by(|a, b| b.cmp(a));

        if triple_ranks.is_empty() {
            return Vec::new();
        }

        let best_triple = triple_ranks[0];
        groups
            .get(&best_triple)
            .map(|group| group.iter().take(3).cloned().collect())
            .unwrap_or_default()
    }

    fn name(&self) -> &'static str {
        "Three Of A Kind"
    }

    fn value(&self) -> (Chips, Mult) {
        (30.0, 3.0)
    }
}
pub struct Flush;

impl HandEvaluator for Flush {
    fn evaluate(&self, cards: &[Card], jokers: &[JokerCard]) -> bool {
        let has_four_fingers = has_four_fingers_joker(jokers);

        let min_cards_needed = if has_four_fingers { 4 } else { 5 };

        if cards.len() < min_cards_needed {
            return false;
        }

        let mut suit_counts = HashMap::new();
        for card in cards {
            *suit_counts.entry(card.suit).or_insert(0) += 1;
        }

        suit_counts.values().any(|&count| count >= min_cards_needed)
    }

    fn get_cards<'a>(&self, cards: &'a [Card], jokers: &[JokerCard]) -> Vec<&'a Card> {
        let has_four_fingers = has_four_fingers_joker(jokers);

        let min_cards_needed = if has_four_fingers { 4 } else { 5 };

        if cards.len() < min_cards_needed {
            return Vec::new();
        }

        let mut suit_groups: HashMap<Suit, Vec<&Card>> = HashMap::new();
        for card in cards {
            suit_groups
                .entry(card.suit)
                .or_default()
                .push(card);
        }

        let flush_suit = suit_groups
            .iter()
            .filter(|(_, group)| group.len() >= min_cards_needed)
            .max_by_key(|(_, group)| group.len());

        if let Some((_, flush_cards)) = flush_suit {
            let mut best_flush_cards = flush_cards.clone();
            best_flush_cards.sort_by(|a, b| b.rank.cmp(&a.rank));
            best_flush_cards.truncate(min_cards_needed);
            return best_flush_cards;
        }

        Vec::new()
    }

    fn name(&self) -> &'static str {
        "Flush"
    }

    fn value(&self) -> (Chips, Mult) {
        (35.0, 4.0)
    }
}
pub struct FullHouse;

impl HandEvaluator for FullHouse {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        if cards.len() < 5 {
            return false;
        }

        let mut rank_counts = HashMap::new();
        for card in cards {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        let has_three = rank_counts.values().any(|&count| count >= 3);

        let mut has_pair = false;
        for (&_rank, &count) in rank_counts.iter() {
            if count >= 2 && !(count >= 3 && rank_counts.values().filter(|&&c| c >= 3).count() == 1)
            {
                has_pair = true;
                break;
            }
        }

        has_three && has_pair
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        let mut rank_counts = HashMap::new();
        for card in cards {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        let mut triplet_ranks: Vec<Rank> = rank_counts
            .iter()
            .filter(|(_, count)| **count >= 3)
            .map(|(&rank, _)| rank)
            .collect();
        triplet_ranks.sort_by(|a, b| b.cmp(a));

        let mut pair_ranks: Vec<Rank> = rank_counts
            .iter()
            .filter(|(_, count)| **count >= 2 && **count < 3)
            .map(|(&rank, _)| rank)
            .collect();
        pair_ranks.sort_by(|a, b| b.cmp(a));

        if triplet_ranks.is_empty() || (pair_ranks.is_empty() && triplet_ranks.len() < 2) {
            return Vec::new();
        }

        let mut rank_groups: HashMap<Rank, Vec<&Card>> = HashMap::new();
        for card in cards {
            rank_groups
                .entry(card.rank)
                .or_default()
                .push(card);
        }

        let mut result = Vec::new();

        let best_triplet_rank = triplet_ranks[0];
        result.extend(rank_groups.get(&best_triplet_rank).unwrap().iter().take(3));

        if !pair_ranks.is_empty() {
            let best_pair_rank = pair_ranks[0];
            result.extend(rank_groups.get(&best_pair_rank).unwrap().iter().take(2));
        } else if triplet_ranks.len() >= 2 {
            let second_triplet_rank = triplet_ranks[1];
            result.extend(
                rank_groups
                    .get(&second_triplet_rank)
                    .unwrap()
                    .iter()
                    .take(2),
            );
        }

        result
    }

    fn name(&self) -> &'static str {
        "Full House"
    }

    fn value(&self) -> (Chips, Mult) {
        (40.0, 4.0)
    }
}

pub struct FourOfAKind;

impl HandEvaluator for FourOfAKind {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        if cards.len() < 4 {
            return false;
        }

        let mut rank_counts = HashMap::new();
        for card in cards {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        rank_counts.values().any(|count| *count >= 4)
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        let mut rank_counts = HashMap::new();
        for card in cards {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        let mut quad_ranks: Vec<Rank> = rank_counts
            .iter()
            .filter(|(_, count)| **count >= 4)
            .map(|(&rank, _)| rank)
            .collect();

        if quad_ranks.is_empty() {
            return Vec::new();
        }

        quad_ranks.sort_by(|a, b| b.cmp(a));

        let mut rank_groups: HashMap<Rank, Vec<&Card>> = HashMap::new();
        for card in cards {
            rank_groups
                .entry(card.rank)
                .or_default()
                .push(card);
        }

        let best_quad_rank = quad_ranks[0];

        let quad_cards = rank_groups.get(&best_quad_rank).unwrap();
        let mut result = Vec::new();
        result.extend(quad_cards.iter().take(4));

        result
    }

    fn name(&self) -> &'static str {
        "Four of a Kind"
    }

    fn value(&self) -> (Chips, Mult) {
        (60.0, 7.0)
    }
}

pub struct Straight;

impl HandEvaluator for Straight {
    fn evaluate(&self, cards: &[Card], jokers: &[JokerCard]) -> bool {
        let has_shortcut = has_shortcut_joker(jokers);
        let min_cards_needed = get_min_cards_needed(jokers);

        if cards.len() < min_cards_needed {
            return false;
        }

        let mut orders: Vec<u8> = cards.iter().map(|c| rank_to_order(c.rank)).collect();
        if orders.contains(&14) {
            orders.push(1);
        }
        orders.sort_unstable();
        orders.dedup();

        if is_consecutive(&orders, min_cards_needed) {
            return true;
        }

        if has_shortcut && check_shortcut_straight(&orders, min_cards_needed) {
            return true;
        }
        false
    }

    fn get_cards<'a>(&self, cards: &'a [Card], jokers: &[JokerCard]) -> Vec<&'a Card> {
        let has_shortcut = has_shortcut_joker(jokers);
        let min_cards_needed = get_min_cards_needed(jokers);

        if cards.len() < min_cards_needed {
            return Vec::new();
        }

        let mut order_to_cards: HashMap<u8, Vec<&Card>> = HashMap::new();
        for card in cards {
            let order = rank_to_order(card.rank);
            order_to_cards
                .entry(order)
                .or_default()
                .push(card);
            if card.rank == Rank::Ace {
                order_to_cards.entry(1).or_default().push(card);
            }
        }

        let mut orders: Vec<u8> = order_to_cards.keys().cloned().collect();
        orders.sort_unstable();

        for window in orders.windows(min_cards_needed) {
            if window[window.len() - 1] - window[0] == (min_cards_needed - 1) as u8 {
                let mut result = Vec::new();
                for &order in window.iter().rev().take(min_cards_needed) {
                    if let Some(card_list) = order_to_cards.get(&order) {
                        if !card_list.is_empty() {
                            result.push(card_list[0]);
                        }
                    }
                }
                if result.len() == min_cards_needed {
                    return result;
                }
            }
        }

        if has_shortcut {
            for window_size in min_cards_needed..=orders.len() {
                for window in orders.windows(window_size) {
                    let valid = window.windows(2).all(|w| w[1] - w[0] <= 2)
                        && window.windows(2).any(|w| w[1] - w[0] == 2);
                    if valid {
                        let mut result = Vec::new();
                        for &order in window.iter().rev().take(min_cards_needed) {
                            if let Some(card_list) = order_to_cards.get(&order) {
                                if !card_list.is_empty() {
                                    result.push(card_list[0]);
                                }
                            }
                        }
                        if result.len() == min_cards_needed {
                            return result;
                        }
                    }
                }
            }
        }

        Vec::new()
    }

    fn name(&self) -> &'static str {
        "Straight"
    }

    fn value(&self) -> (Chips, Mult) {
        (30.0, 4.0)
    }
}
pub struct StraightFlush;

impl HandEvaluator for StraightFlush {
    fn evaluate(&self, cards: &[Card], jokers: &[JokerCard]) -> bool {
        let has_shortcut = has_shortcut_joker(jokers);
        let min_cards_needed = get_min_cards_needed(jokers);

        if cards.len() < min_cards_needed {
            return false;
        }

        let suit_groups = group_by_suit(cards);
        for (_, suit_cards) in suit_groups
            .iter()
            .filter(|(_, cards)| cards.len() >= min_cards_needed)
        {
            let mut orders: Vec<u8> = suit_cards.iter().map(|c| rank_to_order(c.rank)).collect();
            if orders.contains(&14) {
                orders.push(1);
            }
            orders.sort_unstable();
            orders.dedup();

            if is_consecutive(&orders, min_cards_needed) {
                return true;
            }

            if has_shortcut && check_shortcut_straight(&orders, min_cards_needed) {
                return true;
            }
        }
        false
    }

    fn get_cards<'a>(&self, cards: &'a [Card], jokers: &[JokerCard]) -> Vec<&'a Card> {
        let has_shortcut = has_shortcut_joker(jokers);
        let min_cards_needed = get_min_cards_needed(jokers);

        if cards.len() < min_cards_needed {
            return Vec::new();
        }

        let suit_groups = group_by_suit(cards);
        for (_, suit_cards) in suit_groups
            .iter()
            .filter(|(_, cards)| cards.len() >= min_cards_needed)
        {
            let mut order_to_cards: HashMap<u8, Vec<&Card>> = HashMap::new();
            for &card in suit_cards.iter() {
                let order = rank_to_order(card.rank);
                order_to_cards
                    .entry(order)
                    .or_default()
                    .push(card);
                if card.rank == Rank::Ace {
                    order_to_cards.entry(1).or_default().push(card);
                }
            }

            let mut orders: Vec<u8> = order_to_cards.keys().cloned().collect();
            orders.sort_unstable();

            for window in orders.windows(min_cards_needed) {
                if window[window.len() - 1] - window[0] == (min_cards_needed - 1) as u8 {
                    let mut result = Vec::new();
                    for &order in window.iter().rev().take(min_cards_needed) {
                        if let Some(card_list) = order_to_cards.get(&order) {
                            if !card_list.is_empty() {
                                result.push(card_list[0]);
                            }
                        }
                    }
                    if result.len() == min_cards_needed {
                        return result;
                    }
                }
            }

            if has_shortcut {
                for window_size in min_cards_needed..=orders.len() {
                    for window in orders.windows(window_size) {
                        let valid = window.windows(2).all(|w| w[1] - w[0] <= 2)
                            && window.windows(2).any(|w| w[1] - w[0] == 2);
                        if valid {
                            let mut result = Vec::new();
                            for &order in window.iter().rev().take(min_cards_needed) {
                                if let Some(card_list) = order_to_cards.get(&order) {
                                    if !card_list.is_empty() {
                                        result.push(card_list[0]);
                                    }
                                }
                            }
                            if result.len() == min_cards_needed {
                                return result;
                            }
                        }
                    }
                }
            }
        }
        Vec::new()
    }

    fn name(&self) -> &'static str {
        "Straight Flush"
    }

    fn value(&self) -> (Chips, Mult) {
        (100.0, 8.0)
    }
}
pub struct FiveOfAKind;

impl HandEvaluator for FiveOfAKind {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        if cards.len() < 5 {
            return false;
        }

        let mut rank_counts = HashMap::new();
        for card in cards {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        rank_counts.values().any(|&count| count >= 5)
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        let mut rank_counts = HashMap::new();
        for card in cards {
            *rank_counts.entry(card.rank).or_insert(0) += 1;
        }

        let mut quint_ranks: Vec<Rank> = rank_counts
            .iter()
            .filter(|(_, count)| **count >= 5)
            .map(|(&rank, _)| rank)
            .collect();

        if quint_ranks.is_empty() {
            return Vec::new();
        }

        quint_ranks.sort_by(|a, b| b.cmp(a));

        let mut rank_groups: HashMap<Rank, Vec<&Card>> = HashMap::new();
        for card in cards {
            rank_groups
                .entry(card.rank)
                .or_default()
                .push(card);
        }

        let best_quint_rank = quint_ranks[0];

        let quint_cards = rank_groups.get(&best_quint_rank).unwrap();
        let mut result = Vec::new();
        result.extend(quint_cards.iter().take(5));

        result
    }

    fn name(&self) -> &'static str {
        "Five of a Kind"
    }

    fn value(&self) -> (Chips, Mult) {
        (120.0, 12.0)
    }
}

pub struct FlushHouse;

impl HandEvaluator for FlushHouse {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        if cards.len() < 5 {
            return false;
        }

        let mut suit_groups: HashMap<Suit, Vec<&Card>> = HashMap::new();
        for card in cards {
            suit_groups
                .entry(card.suit)
                .or_default()
                .push(card);
        }

        for (_, suit_cards) in suit_groups.iter().filter(|(_, cards)| cards.len() >= 5) {
            let mut rank_counts = HashMap::new();
            for card in suit_cards {
                *rank_counts.entry(card.rank).or_insert(0) += 1;
            }

            let has_three = rank_counts.values().any(|&count| count >= 3);
            let pair_count = rank_counts.values().filter(|count| **count >= 2).count();

            if has_three
                && (pair_count >= 2
                    || rank_counts.values().filter(|count| **count >= 3).count() >= 2)
            {
                return true;
            }
        }

        false
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        if cards.len() < 5 {
            return Vec::new();
        }

        let mut suit_groups: HashMap<Suit, Vec<&Card>> = HashMap::new();
        for card in cards {
            suit_groups
                .entry(card.suit)
                .or_default()
                .push(card);
        }

        for (_, suit_cards) in suit_groups.iter().filter(|(_, cards)| cards.len() >= 5) {
            let mut rank_counts = HashMap::new();
            for card in suit_cards {
                *rank_counts.entry(card.rank).or_insert(0) += 1;
            }

            let mut triplet_ranks: Vec<Rank> = rank_counts
                .iter()
                .filter(|(_, count)| **count >= 3)
                .map(|(&rank, _)| rank)
                .collect();
            triplet_ranks.sort_by(|a, b| b.cmp(a));

            let mut pair_ranks: Vec<Rank> = rank_counts
                .iter()
                .filter(|(_, count)| **count >= 2 && **count < 3)
                .map(|(&rank, _)| rank)
                .collect();
            pair_ranks.sort_by(|a, b| b.cmp(a));

            if triplet_ranks.is_empty() || (pair_ranks.is_empty() && triplet_ranks.len() < 2) {
                continue;
            }

            let mut rank_groups: HashMap<Rank, Vec<&Card>> = HashMap::new();
            for card in suit_cards {
                rank_groups
                    .entry(card.rank)
                    .or_default()
                    .push(card);
            }

            let mut result = Vec::new();

            let best_triplet_rank = triplet_ranks[0];
            result.extend(rank_groups.get(&best_triplet_rank).unwrap().iter().take(3));

            if !pair_ranks.is_empty() {
                let best_pair_rank = pair_ranks[0];
                result.extend(rank_groups.get(&best_pair_rank).unwrap().iter().take(2));
            } else if triplet_ranks.len() >= 2 {
                let second_triplet_rank = triplet_ranks[1];
                result.extend(
                    rank_groups
                        .get(&second_triplet_rank)
                        .unwrap()
                        .iter()
                        .take(2),
                );
            }

            return result;
        }

        Vec::new()
    }

    fn name(&self) -> &'static str {
        "Flush House"
    }

    fn value(&self) -> (Chips, Mult) {
        (140.0, 14.0)
    }
}

pub struct FlushFive;

impl HandEvaluator for FlushFive {
    fn evaluate(&self, cards: &[Card], _jokers: &[JokerCard]) -> bool {
        if cards.len() < 5 {
            return false;
        }

        let mut suit_groups: HashMap<Suit, Vec<&Card>> = HashMap::new();
        for card in cards {
            suit_groups
                .entry(card.suit)
                .or_default()
                .push(card);
        }

        for (_, suit_cards) in suit_groups.iter() {
            let mut rank_counts = HashMap::new();
            for card in suit_cards {
                *rank_counts.entry(card.rank).or_insert(0) += 1;
            }

            if rank_counts.values().any(|&count| count >= 5) {
                return true;
            }
        }

        false
    }

    fn get_cards<'a>(&self, cards: &'a [Card], _jokers: &[JokerCard]) -> Vec<&'a Card> {
        if cards.len() < 5 {
            return Vec::new();
        }

        let mut suit_groups: HashMap<Suit, Vec<&Card>> = HashMap::new();
        for card in cards {
            suit_groups
                .entry(card.suit)
                .or_default()
                .push(card);
        }

        for (_, suit_cards) in suit_groups.iter() {
            let mut rank_counts = HashMap::new();
            for &card in suit_cards {
                *rank_counts.entry(card.rank).or_insert(0) += 1;
            }

            let mut quint_ranks: Vec<Rank> = rank_counts
                .iter()
                .filter(|(_, count)| **count >= 5)
                .map(|(&rank, _)| rank)
                .collect();

            if !quint_ranks.is_empty() {
                quint_ranks.sort_by(|a, b| b.cmp(a));

                let mut rank_groups: HashMap<Rank, Vec<&Card>> = HashMap::new();
                for &card in suit_cards {
                    rank_groups
                        .entry(card.rank)
                        .or_default()
                        .push(card);
                }

                let best_quint_rank = quint_ranks[0];
                let quint_cards = rank_groups.get(&best_quint_rank).unwrap();

                return quint_cards.iter().take(5).cloned().collect();
            }
        }

        Vec::new()
    }

    fn name(&self) -> &'static str {
        "Flush Five"
    }

    fn value(&self) -> (Chips, Mult) {
        (160.0, 16.0)
    }
}

pub struct PokerHand {
    evaluators: Vec<Box<dyn HandEvaluator>>,
}

impl Default for PokerHand {
    fn default() -> Self {
        Self::new()
    }
}

impl PokerHand {
    pub fn new() -> Self {
        let mut hand = PokerHand {
            evaluators: Vec::new(),
        };
        hand.evaluators.push(Box::new(FlushFive));
        hand.evaluators.push(Box::new(FlushHouse));
        hand.evaluators.push(Box::new(FiveOfAKind));
        hand.evaluators.push(Box::new(StraightFlush));
        hand.evaluators.push(Box::new(FourOfAKind));
        hand.evaluators.push(Box::new(FullHouse));
        hand.evaluators.push(Box::new(Flush));
        hand.evaluators.push(Box::new(Straight));
        hand.evaluators.push(Box::new(ThreeOfAKind));
        hand.evaluators.push(Box::new(TwoPair));
        hand.evaluators.push(Box::new(Pair));
        hand.evaluators.push(Box::new(HighCard));

        hand
    }
    /// Finds the best hand type and its cards from the given set.
    pub fn find_best_hand<'a>(
        &self,
        cards: &'a [Card],
        jokers: &[JokerCard],
    ) -> Option<(&dyn HandEvaluator, Vec<&'a Card>)> {
        for evaluator in &self.evaluators {
            if evaluator.evaluate(cards, jokers) {
                let hand_cards = evaluator.get_cards(cards, jokers);
                return Some((&**evaluator, hand_cards));
            }
        }

        None
    }
    /// Gets the name of the best hand type.
    pub fn get_name(&self, cards: &[Card], jokers: &[JokerCard]) -> Option<&'static str> {
        self.find_best_hand(cards, jokers)
            .map(|(evaluator, _)| evaluator.name())
    }
    /// Gets the base value of the best hand type.
    pub fn get_value(&self, cards: &[Card], jokers: &[JokerCard]) -> Option<(Chips, Mult)> {
        self.find_best_hand(cards, jokers)
            .map(|(evaluator, _)| evaluator.value())
    }
}

/// Creates a default poker hand evaluator instance.
pub fn create_poker_hand() -> PokerHand {
    PokerHand::new()
}

/// Checks if the Four Fingers joker is present.
fn has_four_fingers_joker(jokers: &[JokerCard]) -> bool {
    jokers
        .iter()
        .any(|joker| matches!(joker.joker, ortalib::Joker::FourFingers))
}

/// Checks if the Shortcut joker is present.
fn has_shortcut_joker(jokers: &[JokerCard]) -> bool {
    let has_shortcut = jokers
        .iter()
        .any(|joker| matches!(joker.joker, ortalib::Joker::Shortcut));
    has_shortcut
}

/// Determines the minimum cards needed based on jokers.
fn get_min_cards_needed(jokers: &[JokerCard]) -> usize {
    if has_four_fingers_joker(jokers) { 4 } else { 5 }
}

/// Groups cards by suit.
fn group_by_suit(cards: &[Card]) -> HashMap<Suit, Vec<&Card>> {
    let mut suit_groups: HashMap<Suit, Vec<&Card>> = HashMap::new();
    for card in cards {
        suit_groups
            .entry(card.suit)
            .or_default()
            .push(card);
    }
    suit_groups
}

/// Checks if a sequence of ranks is consecutive.
fn is_consecutive(orders: &[u8], min_cards_needed: usize) -> bool {
    for window in orders.windows(min_cards_needed) {
        if window[window.len() - 1] - window[0] == (min_cards_needed - 1) as u8 {
            return true;
        }
    }
    false
}

/// Checks for a shortcut straight (allowing gaps) in ranks.
fn check_shortcut_straight(orders: &[u8], min_cards_needed: usize) -> bool {
    for window_size in min_cards_needed..=orders.len() {
        for window in orders.windows(window_size) {
            let valid = window.windows(2).all(|w| w[1] - w[0] <= 2)
                && window.windows(2).any(|w| w[1] - w[0] == 2);
            if valid {
                return true;
            }
        }
    }
    false
}
