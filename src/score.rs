use crate::joker::{JokerActivation, JokerContext, JokerFactory, ScoringScope, get_joker_id};
use crate::modifiers::{self, handle_wild};
use crate::pokerhand::create_poker_hand;
use ortalib::{Card, Chips, JokerCard, Mult, Round};

pub struct ScoreManager {
    cards_played: Vec<Card>,
    cards_in_hand: Vec<Card>,
    chips: Chips,
    mult: Mult,
    best_hand_name: Option<String>,
    best_hand_cards: Vec<Card>,
    base_chips: Chips,
    base_mult: Mult,
    jokers: Vec<JokerCard>,
}

impl ScoreManager {
    /// Creates a new `ScoreManager` instance from a given round, initializing scoring state.
    pub fn from_round(round: &Round) -> Self {
        ScoreManager {
            cards_played: round.cards_played.clone(),
            cards_in_hand: round.cards_held_in_hand.clone(),
            chips: 0.0,
            mult: 0.0,
            best_hand_name: None,
            best_hand_cards: Vec::new(),
            base_chips: 0.0,
            base_mult: 0.0,
            jokers: round.jokers.clone(),
        }
    }

    /// Calculates the total score by evaluating the best poker hand and applying effects.
    pub fn calculate_score(&mut self) -> f64 {
        let poker_hand = create_poker_hand();
        let mut cards_to_evaluate = handle_wild(&self.cards_played);

        let context = JokerContext {
            cards_played: &self.cards_played,
            cards_in_hand: &self.cards_in_hand,
            best_hand_name: self.best_hand_name.as_deref(),
            all_jokers: &self.jokers,
        };

        let mut joker_effects = std::collections::HashMap::new();
        for joker in &self.jokers {
            let joker_id = get_joker_id(&joker.joker);
            let joker_effect = JokerFactory::create_joker(&joker.joker);
            joker_effects.insert(joker_id, joker_effect);
        }

        if !self
            .cards_played
            .iter()
            .any(|card| matches!(card.enhancement, Some(ortalib::Enhancement::Wild)))
        {
            cards_to_evaluate = context.with_modified_suits();
        }

        let mut scoring_scope = ScoringScope::BestHand;
        for joker in &self.jokers {
            let joker_effect = joker_effects.get(&get_joker_id(&joker.joker)).unwrap();
            let scope = joker_effect.scoring_scope(&context);
            if matches!(scope, ScoringScope::AllPlayed) {
                scoring_scope = scope;
                break;
            }
        }

        if let Some((evaluator, hand_cards)) =
            poker_hand.find_best_hand(&cards_to_evaluate, &self.jokers)
        {
            self.best_hand_name = Some(evaluator.name().to_string());
            self.best_hand_cards = hand_cards.into_iter().cloned().collect();

            let updated_context = JokerContext {
                cards_played: &self.cards_played,
                cards_in_hand: &self.cards_in_hand,
                best_hand_name: self.best_hand_name.as_deref(),
                all_jokers: &self.jokers,
            };

            for joker in &self.jokers {
                let joker_effect = joker_effects.get(&get_joker_id(&joker.joker)).unwrap();
                if let Some(preferred_scope) =
                    joker_effect.preferred_scoring_scope(&updated_context)
                {
                    scoring_scope = preferred_scope;
                    break;
                }
            }

            let (base_chips, base_mult) = evaluator.value();
            self.base_chips = base_chips;
            self.base_mult = base_mult;

            self.chips = base_chips;
            self.mult = base_mult;

            let cards_to_score = match scoring_scope {
                ScoringScope::AllPlayed => &self.cards_played,
                ScoringScope::BestHand => &self.best_hand_cards,
                ScoringScope::Custom(ref _cards) => {
                    panic!("Custom scoring scope not yet supported");
                }
            };

            for card in cards_to_score {
                let card_value = card.rank.rank_value();
                self.chips += card_value;

                if let Some(enhancement_type) = &card.enhancement {
                    let enhancement = modifiers::create_enhancement_handler(enhancement_type);
                    enhancement.apply(&mut self.chips, &mut self.mult, card, false);
                }

                if let Some(edition_type) = &card.edition {
                    let edition = modifiers::create_edition_handler(edition_type);
                    edition.apply(&mut self.chips, &mut self.mult, card);
                }

                let context = JokerContext {
                    cards_played: &self.cards_played,
                    cards_in_hand: &self.cards_in_hand,
                    best_hand_name: self.best_hand_name.as_deref(),
                    all_jokers: &self.jokers,
                };

                for joker in &self.jokers {
                    let joker_effect = joker_effects.get(&get_joker_id(&joker.joker)).unwrap();
                    if matches!(joker_effect.activation_type(), JokerActivation::OnScored) {
                        let applied = joker_effect.apply(
                            &mut self.chips,
                            &mut self.mult,
                            Some(card),
                            &context,
                        );

                        if applied && joker_effect.supports_retrigger() {
                            let card_value = card.rank.rank_value();
                            self.chips += card_value;

                            if let Some(enhancement_type) = &card.enhancement {
                                let enhancement =
                                    modifiers::create_enhancement_handler(enhancement_type);
                                enhancement.apply(&mut self.chips, &mut self.mult, card, false);
                            }

                            if let Some(edition_type) = &card.edition {
                                let edition = modifiers::create_edition_handler(edition_type);
                                edition.apply(&mut self.chips, &mut self.mult, card);
                            }

                            for retrigger_joker in &self.jokers {
                                let retrigger_effect = joker_effects
                                    .get(&get_joker_id(&retrigger_joker.joker))
                                    .unwrap();
                                if matches!(
                                    retrigger_effect.activation_type(),
                                    JokerActivation::OnScored
                                ) {
                                    retrigger_effect.apply(
                                        &mut self.chips,
                                        &mut self.mult,
                                        Some(card),
                                        &context,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            for card in &self.cards_in_hand {
                if let Some(enhancement_type) = &card.enhancement {
                    if matches!(enhancement_type, ortalib::Enhancement::Steel) {
                        let enhancement = modifiers::create_enhancement_handler(enhancement_type);
                        enhancement.apply(&mut self.chips, &mut self.mult, card, true);
                    }
                }
            }

            {
                let context = JokerContext {
                    cards_played: &self.cards_played,
                    cards_in_hand: &self.cards_in_hand,
                    best_hand_name: self.best_hand_name.as_deref(),
                    all_jokers: &self.jokers,
                };

                for card in &self.cards_in_hand {
                    let mut processed_joker_indices = std::collections::HashSet::new();

                    for (index, joker) in self.jokers.iter().enumerate() {
                        if !processed_joker_indices.insert(index) {
                            continue;
                        }

                        let joker_id = get_joker_id(&joker.joker);
                        let joker_effect = joker_effects.get(&joker_id).unwrap();

                        if matches!(joker_effect.activation_type(), JokerActivation::OnHeld)
                            && !matches!(joker.joker, ortalib::Joker::Mime)
                        {
                            joker_effect.apply(
                                &mut self.chips,
                                &mut self.mult,
                                Some(card),
                                &context,
                            );
                        }
                    }

                    processed_joker_indices.clear();
                    for (index, joker) in self.jokers.iter().enumerate() {
                        if !processed_joker_indices.insert(index) {
                            continue;
                        }

                        let joker_id = get_joker_id(&joker.joker);
                        let joker_effect = joker_effects.get(&joker_id).unwrap();

                        if matches!(joker_effect.activation_type(), JokerActivation::OnHeld)
                            && matches!(joker.joker, ortalib::Joker::Mime)
                        {
                            joker_effect.apply(
                                &mut self.chips,
                                &mut self.mult,
                                Some(card),
                                &context,
                            );
                        }
                    }

                    processed_joker_indices.clear();
                    for (index, joker) in self.jokers.iter().enumerate() {
                        if !processed_joker_indices.insert(index) {
                            continue;
                        }

                        let joker_id = get_joker_id(&joker.joker);
                        let joker_effect = joker_effects.get(&joker_id).unwrap();

                        if joker_effect.supports_retrigger()
                            && matches!(joker_effect.activation_type(), JokerActivation::OnHeld)
                        {
                            let retrigger_context = JokerContext {
                                cards_played: &self.cards_played,
                                cards_in_hand: &self.cards_in_hand,
                                best_hand_name: self.best_hand_name.as_deref(),
                                all_jokers: &self.jokers,
                            };

                            joker_effect.apply(
                                &mut self.chips,
                                &mut self.mult,
                                Some(card),
                                &retrigger_context,
                            );
                        }
                    }
                }
            }

            {
                let context = JokerContext {
                    cards_played: &self.cards_played,
                    cards_in_hand: &self.cards_in_hand,
                    best_hand_name: self.best_hand_name.as_deref(),
                    all_jokers: &self.jokers,
                };

                let mut processed_joker_indices = std::collections::HashSet::new();

                for joker in &self.jokers {
                    if let Some(edition_type) = &joker.edition {
                        if matches!(edition_type, ortalib::Edition::Foil)
                            || matches!(edition_type, ortalib::Edition::Holographic)
                        {
                            modifiers::apply_edition_effect(
                                edition_type,
                                &mut self.chips,
                                &mut self.mult,
                            );
                        }
                    }
                }

                for (index, joker) in self.jokers.iter().enumerate() {
                    if !processed_joker_indices.insert(index) {
                        continue;
                    }

                    let joker_id = get_joker_id(&joker.joker);
                    let joker_effect = joker_effects.get(&joker_id).unwrap();

                    if matches!(joker_effect.activation_type(), JokerActivation::Independent) {
                        joker_effect.apply(&mut self.chips, &mut self.mult, None, &context);
                    }
                }

                for joker in &self.jokers {
                    if let Some(edition_type) = &joker.edition {
                        if matches!(edition_type, ortalib::Edition::Polychrome) {
                            modifiers::apply_edition_effect(
                                edition_type,
                                &mut self.chips,
                                &mut self.mult,
                            );
                        }
                    }
                }
            }
        } else if matches!(scoring_scope, ScoringScope::AllPlayed) {
            self.chips = 0.0;
            self.mult = 1.0;
        } else {
            return 0.0;
        }
        (self.chips * self.mult).floor()
    }

    /// Computes the score for a round and provides an explanation of the result.
    pub fn score_with_explanation(round: &Round) -> (Chips, Mult, String) {
        let mut manager = ScoreManager::from_round(round);
        let final_score = manager.calculate_score();
        let explanation = if let Some(ref hand_name) = manager.best_hand_name {
            format!("{} (Final Score: {})", hand_name, final_score)
        } else {
            "No valid poker hand identified".to_string()
        };

        (manager.chips, manager.mult, explanation)
    }
}
