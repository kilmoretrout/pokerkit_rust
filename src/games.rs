//! Implements various poker game definitions, acting as factories for `State`.

use std::collections::{BTreeMap, HashSet};

use crate::hands::HandType;
use crate::state::{
    Automation, BettingStructure, Mode, Opening, State, StateBuilder, Street,
};
use crate::utilities::{div_mod, rake, Deck};

// A helper type for raw values like antes, blinds, and stacks.
type RawValues = BTreeMap<usize, i64>;

// Each struct here represents a specific poker game variant.
// They don't hold data themselves but provide a `create_state` method
// to construct a fully configured `State`.

pub struct FixedLimitTexasHoldem;

impl FixedLimitTexasHoldem {
    pub fn create_state(
        automations: &[Automation],
        ante_trimming_status: bool,
        raw_antes: RawValues,
        raw_blinds_or_straddles: RawValues,
        small_bet: i64,
        big_bet: i64,
        raw_starting_stacks: RawValues,
        player_count: usize,
        mode: Mode,
    ) -> Result<State, String> {
        let streets = vec![
            Street::new(false, vec![false; 2], 0, false, Opening::Position, small_bet, Some(4))?,
            Street::new(true, vec![], 3, false, Opening::Position, small_bet, Some(4))?,
            Street::new(true, vec![], 1, false, Opening::Position, big_bet, Some(4))?,
            Street::new(true, vec![], 1, false, Opening::Position, big_bet, Some(4))?,
        ];

        StateBuilder::new(player_count)
            .automations(automations)
            .streets(streets)
            .deck(Deck::standard())
            .hand_types(vec![HandType::StandardHighHand])
            .betting_structure(BettingStructure::FixedLimit)
            .ante_trimming_status(ante_trimming_status)
            .raw_antes(raw_antes)
            .raw_blinds_or_straddles(raw_blinds_or_straddles)
            .bring_in(0)
            .raw_starting_stacks(raw_starting_stacks)
            .mode(mode)
            .build()
    }
}

pub struct NoLimitTexasHoldem;

impl NoLimitTexasHoldem {
    pub fn create_state(
        automations: &[Automation],
        ante_trimming_status: bool,
        raw_antes: RawValues,
        raw_blinds_or_straddles: RawValues,
        min_bet: i64,
        raw_starting_stacks: RawValues,
        player_count: usize,
        mode: Mode,
    ) -> Result<State, String> {
        let streets = vec![
            Street::new(false, vec![false; 2], 0, false, Opening::Position, min_bet, None)?,
            Street::new(true, vec![], 3, false, Opening::Position, min_bet, None)?,
            Street::new(true, vec![], 1, false, Opening::Position, min_bet, None)?,
            Street::new(true, vec![], 1, false, Opening::Position, min_bet, None)?,
        ];

        StateBuilder::new(player_count)
            .automations(automations)
            .streets(streets)
            .deck(Deck::standard())
            .hand_types(vec![HandType::StandardHighHand])
            .betting_structure(BettingStructure::NoLimit)
            .ante_trimming_status(ante_trimming_status)
            .raw_antes(raw_antes)
            .raw_blinds_or_straddles(raw_blinds_or_straddles)
            .bring_in(0)
            .raw_starting_stacks(raw_starting_stacks)
            .mode(mode)
            .build()
    }
}

pub struct PotLimitOmahaHoldem;

impl PotLimitOmahaHoldem {
     pub fn create_state(
        automations: &[Automation],
        ante_trimming_status: bool,
        raw_antes: RawValues,
        raw_blinds_or_straddles: RawValues,
        min_bet: i64,
        raw_starting_stacks: RawValues,
        player_count: usize,
        mode: Mode,
    ) -> Result<State, String> {
        let streets = vec![
            Street::new(false, vec![false; 4], 0, false, Opening::Position, min_bet, None)?,
            Street::new(true, vec![], 3, false, Opening::Position, min_bet, None)?,
            Street::new(true, vec![], 1, false, Opening::Position, min_bet, None)?,
            Street::new(true, vec![], 1, false, Opening::Position, min_bet, None)?,
        ];

        StateBuilder::new(player_count)
            .automations(automations)
            .streets(streets)
            .deck(Deck::standard())
            .hand_types(vec![HandType::OmahaHoldemHand])
            .betting_structure(BettingStructure::PotLimit)
            .ante_trimming_status(ante_trimming_status)
            .raw_antes(raw_antes)
            .raw_blinds_or_straddles(raw_blinds_or_straddles)
            .bring_in(0)
            .raw_starting_stacks(raw_starting_stacks)
            .mode(mode)
            .build()
    }
}

// ... Implementations for other game types like Razz, Stud, Draw games etc. would follow a similar pattern.
