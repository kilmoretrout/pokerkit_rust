//! Implements structs and traits related to poker hands.

use std::cmp::Ordering;
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};
use itertools::Itertools;
use once_cell::sync::Lazy;

use crate::lookups::{
    BadugiLookup, EightOrBetterLookup, Entry, KuhnPokerLookup, Lookup, RegularLookup,
    ShortDeckHoldemLookup, StandardBadugiLookup, StandardLookup,
};
use crate::utilities::Card;

// Create static, lazily-initialized instances of each lookup table.
static STANDARD_LOOKUP: Lazy<StandardLookup> = Lazy::new(StandardLookup::new);
static SHORT_DECK_HOLDEM_LOOKUP: Lazy<ShortDeckHoldemLookup> = Lazy::new(ShortDeckHoldemLookup::new);
static EIGHT_OR_BETTER_LOOKUP: Lazy<EightOrBetterLookup> = Lazy::new(EightOrBetterLookup::new);
static REGULAR_LOOKUP: Lazy<RegularLookup> = Lazy::new(RegularLookup::new);
static BADUGI_LOOKUP: Lazy<BadugiLookup> = Lazy::new(BadugiLookup::new);
static STANDARD_BADUGI_LOOKUP: Lazy<StandardBadugiLookup> = Lazy::new(StandardBadugiLookup::new);
static KUHN_POKER_LOOKUP: Lazy<KuhnPokerLookup> = Lazy::new(KuhnPokerLookup::new);

/// A trait representing a poker hand.
/// Stronger hands are considered greater than weaker hands.
pub trait Hand: Sized + Clone + Eq + Hash + Ord + Display + Debug {
    /// `true` if a lower hand is better, `false` otherwise.
    const LOW: bool;
    /// The number of cards that make up this type of hand, if fixed.
    const CARD_COUNT: Option<usize>;

    /// Returns the cards that form this hand.
    fn cards(&self) -> &[Card];
    /// Gets the lookup entry for this hand.
    fn entry(&self) -> Entry;

    /// Creates a new hand from a vector of cards, using a specific lookup.
    fn new(cards: Vec<Card>, lookup: &dyn Lookup) -> Result<Self, String>;

    /// Determines the best possible hand from a set of hole and board cards.
    fn from_game(hole_cards_str: &str, board_cards_str: &str, lookup: &dyn Lookup) -> Result<Self, String>;
}

/// An enum to act as a factory for different hand types.
#[derive(Debug, Clone, Copy)]
pub enum HandType {
    StandardHighHand,
    StandardLowHand,
    ShortDeckHoldemHand,
    EightOrBetterLowHand,
    RegularLowHand,
    OmahaHoldemHand,
    OmahaEightOrBetterLowHand,
    BadugiHand,
    StandardBadugiHand,
    KuhnPokerHand,
}

impl HandType {
    /// Creates the best possible hand of the corresponding type from game cards.
    pub fn from_game(&self, hole_cards_str: &str, board_cards_str: &str) -> Result<Box<impl Hand>, String> {
        match self {
            HandType::StandardHighHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*STANDARD_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::StandardLowHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*STANDARD_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::ShortDeckHoldemHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*SHORT_DECK_HOLDEM_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::EightOrBetterLowHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*EIGHT_OR_BETTER_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::RegularLowHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*REGULAR_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::OmahaHoldemHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*STANDARD_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::OmahaEightOrBetterLowHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*EIGHT_OR_BETTER_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::BadugiHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*BADUGI_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::StandardBadugiHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*STANDARD_BADUGI_LOOKUP)?;
                Ok(Box::new(hand))
            }
            HandType::KuhnPokerHand => {
                let hand = StandardHighHand::from_game(hole_cards_str, board_cards_str, &*KUHN_POKER_LOOKUP)?;
                Ok(Box::new(hand))
            }
        }
    }
}


/// A macro to implement common traits (`PartialEq`, `Ord`, `Hash`, `Display`, `Debug`) for a hand struct.
macro_rules! impl_hand_boilerplate {
    ($hand_type:ident) => {
        impl PartialEq for $hand_type {
            fn eq(&self, other: &Self) -> bool {
                self.entry() == other.entry()
            }
        }
        impl Eq for $hand_type {}

        impl PartialOrd for $hand_type {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $hand_type {
            fn cmp(&self, other: &Self) -> Ordering {
                if <Self as Hand>::LOW {
                    other.entry().cmp(&self.entry())
                } else {
                    self.entry().cmp(&other.entry())
                }
            }
        }

        impl Hash for $hand_type {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.entry().hash(state);
            }
        }

        impl Display for $hand_type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let cards_str = self.cards().iter().map(|c| c.to_string()).collect::<String>();
                write!(f, "{} ({})", self.entry().label, cards_str)
            }
        }

        impl Debug for $hand_type {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let cards_str = self.cards().iter().map(|c| c.to_string()).collect::<String>();
                write!(f, "{}", cards_str)
            }
        }
    };
}

/// A macro for hands made from the best combination of a fixed number of cards.
macro_rules! impl_combination_hand {
    ($hand_type:ident, $is_low:expr, $num_cards:expr, $hand_name:expr) => {
        #[derive(Clone)]
        pub struct $hand_type {
            cards: Vec<Card>,
            entry: Entry,
        }
        impl_hand_boilerplate!($hand_type);

        impl Hand for $hand_type {
            const LOW: bool = $is_low;
            const CARD_COUNT: Option<usize> = Some($num_cards);
            
            fn cards(&self) -> &[Card] { &self.cards }
            fn entry(&self) -> Entry { self.entry }

            fn new(cards: Vec<Card>, lookup: &dyn Lookup) -> Result<Self, String> {
                let cards_str = cards.iter().map(|c| c.to_string()).collect::<String>();
                if cards.len() != Self::CARD_COUNT.unwrap() {
                    return Err(format!("Invalid card count for {}", $hand_name));
                }
                let entry = lookup.get_entry(&cards_str)
                    .map_err(|e| format!("Invalid {} hand: {}", $hand_name, e))?;
                Ok(Self { cards, entry })
            }

            fn from_game(hole_cards_str: &str, board_cards_str: &str, lookup: &dyn Lookup) -> Result<Self, String> {
                let hole_cards = Card::parse_cards(hole_cards_str)?;
                let board_cards = Card::parse_cards(board_cards_str)?;
                let all_cards: Vec<Card> = hole_cards.into_iter().chain(board_cards.into_iter()).collect();

                all_cards
                    .into_iter()
                    .combinations(Self::CARD_COUNT.unwrap())
                    .filter_map(|combo| Self::new(combo, lookup).ok())
                    .max()
                    .ok_or_else(|| format!("No valid {} hand can be formed.", $hand_name))
            }
        }
    };
}

impl_combination_hand!(StandardHighHand, false, 5, "StandardHighHand");
impl_combination_hand!(StandardLowHand, true, 5, "StandardLowHand");
impl_combination_hand!(ShortDeckHoldemHand, false, 5, "ShortDeckHoldemHand");
impl_combination_hand!(EightOrBetterLowHand, true, 5, "EightOrBetterLowHand");
impl_combination_hand!(RegularLowHand, true, 5, "RegularLowHand");

/// A macro for hands that must use a specific number of hole and board cards.
macro_rules! impl_hole_board_combination_hand {
    (
        $hand_type:ident,
        $is_low:expr,
        $total_cards:expr,
        $hole_cards_to_use:expr,
        $board_cards_to_use:expr,
        $hand_name:expr
    ) => {
        #[derive(Clone)]
        pub struct $hand_type {
            cards: Vec<Card>,
            entry: Entry,
        }
        impl_hand_boilerplate!($hand_type);

        impl Hand for $hand_type {
            const LOW: bool = $is_low;
            const CARD_COUNT: Option<usize> = Some($total_cards);
            
            fn cards(&self) -> &[Card] { &self.cards }
            fn entry(&self) -> Entry { self.entry }

            fn new(cards: Vec<Card>, lookup: &dyn Lookup) -> Result<Self, String> {
                let cards_str = cards.iter().map(|c| c.to_string()).collect::<String>();
                if cards.len() != Self::CARD_COUNT.unwrap() {
                    return Err(format!("Invalid card count for {}", $hand_name));
                }
                let entry = lookup.get_entry(&cards_str)
                    .map_err(|e| format!("Invalid {} hand: {}", $hand_name, e))?;
                Ok(Self { cards, entry })
            }

            fn from_game(hole_cards_str: &str, board_cards_str: &str, lookup: &dyn Lookup) -> Result<Self, String> {
                let hole_cards = Card::parse_cards(hole_cards_str)?;
                let board_cards = Card::parse_cards(board_cards_str)?;

                hole_cards
                    .into_iter()
                    .combinations($hole_cards_to_use)
                    .cartesian_product(board_cards.into_iter().combinations($board_cards_to_use))
                    .filter_map(|(h, b)| {
                        let all_cards: Vec<Card> = h.into_iter().chain(b.into_iter()).collect();
                        Self::new(all_cards, lookup).ok()
                    })
                    .max()
                    .ok_or_else(|| format!("No valid {} hand can be formed.", $hand_name))
            }
        }
    };
}

impl_hole_board_combination_hand!(OmahaHoldemHand, false, 5, 2, 3, "OmahaHoldemHand");
impl_hole_board_combination_hand!(OmahaEightOrBetterLowHand, true, 5, 2, 3, "OmahaEightOrBetterLowHand");

#[derive(Clone)]
pub struct BadugiHand { cards: Vec<Card>, entry: Entry }
impl_hand_boilerplate!(BadugiHand);
impl Hand for BadugiHand {
    const LOW: bool = true;
    const CARD_COUNT: Option<usize> = None; // Variable card count
    fn cards(&self) -> &[Card] { &self.cards }
    fn entry(&self) -> Entry { self.entry }

    fn new(cards: Vec<Card>, lookup: &dyn Lookup) -> Result<Self, String> {
        let cards_str = cards.iter().map(|c| c.to_string()).collect::<String>();
        let entry = lookup.get_entry(&cards_str)
            .map_err(|_| format!("The cards '{}' form an invalid BadugiHand hand.", cards_str))?;
        Ok(Self { cards, entry })
    }
    fn from_game(hole_cards_str: &str, board_cards_str: &str, lookup: &dyn Lookup) -> Result<Self, String> {
        let hole_cards = Card::parse_cards(hole_cards_str)?;
        let all_cards: Vec<Card> = hole_cards.into_iter().chain(Card::parse_cards(board_cards_str)?).collect();
        (1..=4).rev()
            .flat_map(|count| all_cards.iter().cloned().combinations(count))
            .filter_map(|combo| Self::new(combo, lookup).ok())
            .max()
            .ok_or_else(|| "No valid BadugiHand hand can be formed".to_string())
    }
}

#[derive(Clone)]
pub struct StandardBadugiHand { cards: Vec<Card>, entry: Entry }
impl_hand_boilerplate!(StandardBadugiHand);
impl Hand for StandardBadugiHand {
    const LOW: bool = true;
    const CARD_COUNT: Option<usize> = None;
    fn cards(&self) -> &[Card] { &self.cards }
    fn entry(&self) -> Entry { self.entry }

    fn new(cards: Vec<Card>, lookup: &dyn Lookup) -> Result<Self, String> {
        let cards_str = cards.iter().map(|c| c.to_string()).collect::<String>();
         let entry = lookup.get_entry(&cards_str)
            .map_err(|_| format!("The cards '{}' form an invalid StandardBadugiHand hand.", cards_str))?;
        Ok(Self { cards, entry })
    }
    fn from_game(hole_cards_str: &str, board_cards_str: &str, lookup: &dyn Lookup) -> Result<Self, String> {
        let hole_cards = Card::parse_cards(hole_cards_str)?;
        let all_cards: Vec<Card> = hole_cards.into_iter().chain(Card::parse_cards(board_cards_str)?).collect();
        (1..=4).rev()
            .flat_map(|count| all_cards.iter().cloned().combinations(count))
            .filter_map(|combo| Self::new(combo, lookup).ok())
            .max()
            .ok_or_else(|| "No valid StandardBadugiHand hand can be formed".to_string())
    }
}

#[derive(Clone)]
pub struct KuhnPokerHand { cards: Vec<Card>, entry: Entry }
impl_hand_boilerplate!(KuhnPokerHand);
impl Hand for KuhnPokerHand {
    const LOW: bool = false;
    const CARD_COUNT: Option<usize> = Some(1);
    fn cards(&self) -> &[Card] { &self.cards }
    fn entry(&self) -> Entry { self.entry }

    fn new(cards: Vec<Card>, lookup: &dyn Lookup) -> Result<Self, String> {
        let cards_str = cards.iter().map(|c| c.to_string()).collect::<String>();
        let entry = lookup.get_entry(&cards_str)
            .map_err(|_| format!("The cards '{}' form an invalid KuhnPokerHand hand.", cards_str))?;
        Ok(Self { cards, entry })
    }
    fn from_game(hole_cards_str: &str, board_cards_str: &str, lookup: &dyn Lookup) -> Result<Self, String> {
        let hole_cards = Card::parse_cards(hole_cards_str)?;
        hole_cards
            .into_iter()
            .chain(Card::parse_cards(board_cards_str)?)
            .filter_map(|card| Self::new(vec![card], lookup).ok())
            .max()
            .ok_or_else(|| "No valid KuhnPokerHand hand can be formed".to_string())
    }
}
