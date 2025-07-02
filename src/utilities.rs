//! This crate, `pokerkit`, provides a collection of modules for building poker-related applications.
//!
//! The `utilities` module, in particular, offers a set of helper constants, functions, classes,
//! and methods that are used throughout the PokerKit project. These utilities are designed to
//! facilitate common poker-related tasks, such as handling cards, managing player actions,
//! and calculating game outcomes.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::str::FromStr;
use std::ops::{Div, Rem, Sub};

use chrono::{NaiveTime, Timelike};
use itertools::Itertools;
use num_bigint::BigInt;
use num_traits::{cast, Num, Signed, Zero};
use rand::seq::SliceRandom;
use rand::thread_rng;
use regex::Regex;
use rust_decimal::Decimal;
use strum_macros::{Display, EnumString};

// A placeholder for the full State struct defined in `state.rs`.
// This is needed for the function signature of `rake`.
use crate::state::State;


/// A regular expression pattern that can never be matched.
pub const UNMATCHABLE_PATTERN: &str = r"(?!)";

/// Represents the rank of a card.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumString, Display)]
pub enum Rank {
    #[strum(serialize = "A")]
    Ace,
    #[strum(serialize = "2")]
    Deuce,
    #[strum(serialize = "3")]
    Trey,
    #[strum(serialize = "4")]
    Four,
    #[strum(serialize = "5")]
    Five,
    #[strum(serialize = "6")]
    Six,
    #[strum(serialize = "7")]
    Seven,
    #[strum(serialize = "8")]
    Eight,
    #[strum(serialize = "9")]
    Nine,
    #[strum(serialize = "T")]
    Ten,
    #[strum(serialize = "J")]
    Jack,
    #[strum(serialize = "Q")]
    Queen,
    #[strum(serialize = "K")]
    King,
    #[strum(serialize = "?")]
    Unknown,
}

/// Defines the ordering of ranks for different poker variants.
pub struct RankOrder;

impl RankOrder {
    pub const STANDARD: [Rank; 13] = [
        Rank::Deuce, Rank::Trey, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
        Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];
    pub const SHORT_DECK_HOLDEM: [Rank; 9] = [
        Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack,
        Rank::Queen, Rank::King, Rank::Ace,
    ];
    pub const REGULAR: [Rank; 13] = [
        Rank::Ace, Rank::Deuce, Rank::Trey, Rank::Four, Rank::Five, Rank::Six,
        Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King,
    ];
    pub const EIGHT_OR_BETTER_LOW: [Rank; 8] = [
        Rank::Ace, Rank::Deuce, Rank::Trey, Rank::Four, Rank::Five, Rank::Six,
        Rank::Seven, Rank::Eight,
    ];
    pub const KUHN_POKER: [Rank; 3] = [Rank::Jack, Rank::Queen, Rank::King];
    pub const ROYAL_POKER: [Rank; 5] = [
        Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];
}

/// Represents the suit of a card.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumString, Display)]
pub enum Suit {
    #[strum(serialize = "c")]
    Club,
    #[strum(serialize = "d")]
    Diamond,
    #[strum(serialize = "h")]
    Heart,
    #[strum(serialize = "s")]
    Spade,
    #[strum(serialize = "?")]
    Unknown,
}

/// Represents a playing card with a rank and a suit.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub const UNKNOWN: Card = Card {
        rank: Rank::Unknown,
        suit: Suit::Unknown,
    };

    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self { rank, suit }
    }

    pub fn get_ranks(cards: &[Card]) -> impl Iterator<Item = Rank> + '_ {
        cards.iter().map(|c| c.rank)
    }

    pub fn get_suits(cards: &[Card]) -> impl Iterator<Item = Suit> + '_ {
        cards.iter().map(|c| c.suit)
    }

    pub fn are_paired(cards: &[Card]) -> bool {
        let ranks: Vec<Rank> = Self::get_ranks(cards).collect();
        ranks.iter().unique().count() != ranks.len()
    }

    pub fn are_suited(cards: &[Card]) -> bool {
        Self::get_suits(cards).unique().count() <= 1
    }

    pub fn are_rainbow(cards: &[Card]) -> bool {
        let suits: Vec<Suit> = Self::get_suits(cards).collect();
        suits.iter().unique().count() == suits.len()
    }

    pub fn parse_cards(s: &str) -> Result<Vec<Card>, String> {
        let s = s.replace("10", "T").replace(',', "");
        let mut cards = Vec::new();
        for content in s.split_whitespace() {
            if content.len() % 2 != 0 {
                return Err(format!(
                    "The length of a card string must be a multiple of 2, but got '{}'",
                    content
                ));
            }
            for i in (0..content.len()).step_by(2) {
                let rank_str = &content[i..i + 1];
                let suit_str = &content[i + 1..i + 2];
                let rank = Rank::from_str(rank_str)
                    .map_err(|_| format!("Invalid rank: '{}'", rank_str))?;
                let suit = Suit::from_str(suit_str)
                    .map_err(|_| format!("Invalid suit: '{}'", suit_str))?;
                cards.push(Card::new(rank, suit));
            }
        }
        Ok(cards)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl FromStr for Card {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cards = Card::parse_cards(s)?;
        if cards.len() == 1 {
            Ok(cards[0])
        } else {
            Err("Expected a single card".to_string())
        }
    }
}

/// Represents a deck of cards.
pub struct Deck;

impl Deck {
    pub fn standard() -> Vec<Card> {
        RankOrder::STANDARD
            .iter()
            .cartesian_product(&[Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade])
            .map(|(&rank, &suit)| Card::new(rank, suit))
            .collect()
    }

    pub fn short_deck_holdem() -> Vec<Card> {
        RankOrder::SHORT_DECK_HOLDEM
            .iter()
            .cartesian_product(&[Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade])
            .map(|(&rank, &suit)| Card::new(rank, suit))
            .collect()
    }
}

pub fn min_or_none<T: Ord>(values: impl IntoIterator<Item = Option<T>>) -> Option<T> {
    values.into_iter().filter_map(|x| x).min()
}

pub fn max_or_none<T: Ord>(values: impl IntoIterator<Item = Option<T>>) -> Option<T> {
    values.into_iter().filter_map(|x| x).max()
}

/// "Cleans" a collection of values into a vector of a fixed size.
pub fn clean_values(values: &BTreeMap<usize, i64>, count: usize) -> Vec<i64> {
    let mut cleaned = vec![0; count];
    for (&k, &v) in values {
        if k < count {
            cleaned[k] = v;
        }
    }
    cleaned
}

pub fn shuffled<T: Clone>(values: &[T]) -> Vec<T> {
    let mut rng = thread_rng();
    let mut shuffled_values = values.to_vec();
    shuffled_values.shuffle(&mut rng);
    shuffled_values
}

pub fn rotated<T: Clone>(values: &[T], count: isize) -> VecDeque<T> {
    let mut deque: VecDeque<T> = values.iter().cloned().collect();
    if count > 0 {
        deque.rotate_right(count as usize);
    } else {
        deque.rotate_left(count.abs() as usize);
    }
    deque
}

/// The default divmod function, using standard integer division.
pub fn div_mod(dividend: i64, divisor: i64) -> (i64, i64) {
    (dividend / divisor, dividend % divisor)
}

/// The default rake function, which takes no rake.
pub fn rake(_state: &State, amount: i64) -> (i64, i64) {
    (0, amount)
}

pub fn parse_value(raw_value: &str) -> Result<Box<impl Num>, String> {
    let raw_value = raw_value.replace(',', "");
    if let Ok(val) = raw_value.parse::<BigInt>() {
        Ok(Box::new(val))
    } else if let Ok(_val) = Decimal::from_str(&raw_value) {
        Err("Decimal parsing is not fully supported in this context.".to_string())
    } else {
        Err(format!("Could not parse '{}' as a number", raw_value))
    }
}

pub fn parse_time(raw_time: &str) -> Result<NaiveTime, chrono::ParseError> {
    NaiveTime::parse_from_str(raw_time, "%H:%M:%S")
}

pub fn sign<T: Signed>(value: T) -> T {
    if value.is_positive() {
        T::one()
    } else if value.is_negative() {
        -T::one()
    } else {
        T::zero()
    }
}