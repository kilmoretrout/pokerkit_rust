//! Implements classes related to poker hand lookups.
//! Lookups are used by PokerKit's hand types to discern hand strengths.

use std::collections::{BTreeMap, HashMap};
use std::cmp::Ordering;
use num_bigint::BigUint;
use itertools::Itertools;

use crate::utilities::{Card, Rank, RankOrder}; // Assuming utilities.rs is in the same crate

// Include the generated PHF map
include!(concat!(env!("OUT_DIR"), "/rank_multipliers.rs"));

/// The enum for all hand classification labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Label {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::HighCard => write!(f, "High card"),
            Label::OnePair => write!(f, "One pair"),
            Label::TwoPair => write!(f, "Two pair"),
            Label::ThreeOfAKind => write!(f, "Three of a kind"),
            Label::Straight => write!(f, "Straight"),
            Label::Flush => write!(f, "Flush"),
            Label::FullHouse => write!(f, "Full house"),
            Label::FourOfAKind => write!(f, "Four of a kind"),
            Label::StraightFlush => write!(f, "Straight flush"),
        }
    }
}

/// An entry in a hand lookup table, representing the strength of a hand.
#[derive(Debug, Clone, Copy, Eq, Hash)]
pub struct Entry {
    /// The strength index of the hand. Stronger hands have a greater index.
    pub index: i32,
    /// The classification label of the hand.
    pub label: Label,
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

/// A trait for hand lookup tables. This is now "dyn" safe.
pub trait Lookup {
    /// Returns the rank order used by this lookup.
    fn rank_order(&self) -> &'static [Rank];

    /// Returns the internal map of entries.
    fn entries(&self) -> &HashMap<(BigUint, bool), Entry>;

    /// Populates the lookup table with hand entries.
    fn add_entries(&mut self);

    /// Hashes a collection of ranks into a unique product of primes.
    /// Changed `impl IntoIterator` to `&[Rank]` to make the trait object-safe.
    fn hash_ranks(&self, ranks: &[Rank]) -> BigUint {
        ranks.iter().map(|r| {
            let rank_char = r.to_string().chars().next().unwrap();
            *RANK_MULTIPLIERS.get(&rank_char).unwrap_or(&1)
        }).product()
    }

    /// Recursively generates hashes for all possible hands given rank multiplicities.
    fn hash_multisets(
        &self,
        ranks: &[Rank],
        counter: &mut BTreeMap<usize, usize>,
    ) -> Vec<BigUint> {
        if counter.is_empty() {
            return vec![BigUint::from(1u32)];
        }

        let mut hashes = Vec::new();
        let (multiplicity, &count) = counter.iter().next_back().unwrap();
        let multiplicity = *multiplicity;
        let count = count;
        counter.remove(&multiplicity);

        for samples in ranks.iter().rev().combinations(count) {
            let sample_ranks: Vec<Rank> = samples.iter().map(|&&r| r).collect();
            let hash_part = self.hash_ranks(&sample_ranks).pow(multiplicity as u32);
            
            let remaining_ranks: Vec<Rank> = ranks.iter().filter(|r| !sample_ranks.contains(r)).cloned().collect();

            for partial_hash in self.hash_multisets(&remaining_ranks, counter) {
                hashes.push(&hash_part * partial_hash);
            }
        }
        
        counter.insert(multiplicity, count);
        hashes
    }

    /// Gets the lookup key for a set of cards.
    fn get_key(&self, cards_str: &str) -> Result<(BigUint, bool), String> {
        let cards = Card::parse_cards(cards_str)?;
        let ranks: Vec<Rank> = Card::get_ranks(&cards).collect(); // Collect into a Vec
        let hash = self.hash_ranks(&ranks); // Pass as a slice
        let suitedness = Card::are_suited(&cards);
        Ok((hash, suitedness))
    }
    
    /// Gets the entry for a given hand.
    fn get_entry(&self, cards_str: &str) -> Result<Entry, String> {
        let key = self.get_key(cards_str)?;
        self.entries()
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("The cards '{}' form an invalid hand.", cards_str))
    }

    /// Gets the entry for a given hand, or `None` if it's invalid.
    fn get_entry_or_none(&self, cards_str: &str) -> Option<Entry> {
        self.get_key(cards_str).ok().and_then(|key| self.entries().get(&key).cloned())
    }

    /// Checks if an entry exists for the given cards.
    fn has_entry(&self, cards_str: &str) -> bool {
        self.get_entry_or_none(cards_str).is_some()
    }
}

/// A helper struct to build a lookup table.
#[derive(Default)]
pub struct LookupBuilder {
    pub entries: HashMap<(BigUint, bool), Entry>,
    entry_count: i32,
}

impl LookupBuilder {
    /// Adds an entry to the table.
    fn add_entry(&mut self, hash: BigUint, suitednesses: &[bool], label: Label) {
        let entry = Entry { index: self.entry_count, label };
        self.entry_count += 1;

        for &suitedness in suitednesses {
            self.entries.insert((hash.clone(), suitedness), entry);
        }
    }

    /// Adds all hands corresponding to a rank multiset (e.g., one pair, two pair).
    pub fn add_multisets(&mut self, lookup: &dyn Lookup, counter: BTreeMap<usize, usize>, suitednesses: &[bool], label: Label) {
        let mut counter = counter;
        let hashes = lookup.hash_multisets(lookup.rank_order(), &mut counter);
        
        for hash in hashes.into_iter().rev() {
            self.add_entry(hash, suitednesses, label);
        }
    }

    /// Adds all straight hands.
    pub fn add_straights(&mut self, lookup: &dyn Lookup, count: usize, suitednesses: &[bool], label: Label) {
        let rank_order = lookup.rank_order();
        let mut wheel_ranks = vec![rank_order[rank_order.len()-1]];
        wheel_ranks.extend_from_slice(&rank_order[..count-1]);

        // Add wheel straight (A-2-3-4-5)
        self.add_entry(lookup.hash_ranks(&wheel_ranks), suitednesses, label);
        
        // Add regular straights
        for i in 0..=(rank_order.len() - count) {
             self.add_entry(lookup.hash_ranks(&rank_order[i..i+count]), suitednesses, label);
        }
    }
    
    /// Finalizes the lookup table by re-indexing all entries to be contiguous.
    pub fn build(mut self) -> HashMap<(BigUint, bool), Entry> {
        let mut sorted_indices: Vec<i32> = self.entries.values().map(|e| e.index).collect();
        sorted_indices.sort_unstable();
        sorted_indices.dedup();

        let reset_indices: HashMap<i32, i32> = sorted_indices
            .into_iter()
            .enumerate()
            .map(|(i, old_index)| (old_index, i as i32))
            .collect();

        for entry in self.entries.values_mut() {
            entry.index = reset_indices[&entry.index];
        }

        self.entries
    }
}

// --- StandardLookup ---
pub struct StandardLookup { entries: HashMap<(BigUint, bool), Entry> }
impl Lookup for StandardLookup {
    fn rank_order(&self) -> &'static [Rank] { &RankOrder::STANDARD }
    fn entries(&self) -> &HashMap<(BigUint, bool), Entry> { &self.entries }
    fn add_entries(&mut self) {
        let mut builder = LookupBuilder::default();
        builder.add_multisets(self, BTreeMap::from([(1,5)]), &[false], Label::HighCard);
        builder.add_multisets(self, BTreeMap::from([(2,1), (1,3)]), &[false], Label::OnePair);
        builder.add_multisets(self, BTreeMap::from([(2,2), (1,1)]), &[false], Label::TwoPair);
        builder.add_multisets(self, BTreeMap::from([(3,1), (1,2)]), &[false], Label::ThreeOfAKind);
        builder.add_straights(self, 5, &[false], Label::Straight);
        builder.add_multisets(self, BTreeMap::from([(1,5)]), &[true], Label::Flush);
        builder.add_multisets(self, BTreeMap::from([(3,1), (2,1)]), &[false], Label::FullHouse);
        builder.add_multisets(self, BTreeMap::from([(4,1), (1,1)]), &[false], Label::FourOfAKind);
        builder.add_straights(self, 5, &[true], Label::StraightFlush);
        self.entries = builder.build();
    }
}
impl StandardLookup { pub fn new() -> Self { let mut lookup = Self { entries: HashMap::new() }; lookup.add_entries(); lookup } }
impl Default for StandardLookup { fn default() -> Self { Self::new() } }

// --- ShortDeckHoldemLookup ---
pub struct ShortDeckHoldemLookup { entries: HashMap<(BigUint, bool), Entry> }
impl Lookup for ShortDeckHoldemLookup {
    fn rank_order(&self) -> &'static [Rank] { &RankOrder::SHORT_DECK_HOLDEM }
    fn entries(&self) -> &HashMap<(BigUint, bool), Entry> { &self.entries }
    fn add_entries(&mut self) {
        let mut builder = LookupBuilder::default();
        builder.add_multisets(self, BTreeMap::from([(1,5)]), &[false], Label::HighCard);
        builder.add_multisets(self, BTreeMap::from([(2,1), (1,3)]), &[false], Label::OnePair);
        builder.add_multisets(self, BTreeMap::from([(2,2), (1,1)]), &[false], Label::TwoPair);
        builder.add_multisets(self, BTreeMap::from([(3,1), (1,2)]), &[false], Label::ThreeOfAKind);
        builder.add_straights(self, 5, &[false], Label::Straight);
        builder.add_multisets(self, BTreeMap::from([(3,1), (2,1)]), &[false], Label::FullHouse);
        builder.add_multisets(self, BTreeMap::from([(1,5)]), &[true], Label::Flush);
        builder.add_multisets(self, BTreeMap::from([(4,1), (1,1)]), &[false], Label::FourOfAKind);
        builder.add_straights(self, 5, &[true], Label::StraightFlush);
        self.entries = builder.build();
    }
}
impl ShortDeckHoldemLookup { pub fn new() -> Self { let mut lookup = Self { entries: HashMap::new() }; lookup.add_entries(); lookup } }
impl Default for ShortDeckHoldemLookup { fn default() -> Self { Self::new() } }

// --- EightOrBetterLookup ---
pub struct EightOrBetterLookup { entries: HashMap<(BigUint, bool), Entry> }
impl Lookup for EightOrBetterLookup {
    fn rank_order(&self) -> &'static [Rank] { &RankOrder::EIGHT_OR_BETTER_LOW }
    fn entries(&self) -> &HashMap<(BigUint, bool), Entry> { &self.entries }
    fn add_entries(&mut self) {
        let mut builder = LookupBuilder::default();
        builder.add_multisets(self, BTreeMap::from([(1,5)]), &[false, true], Label::HighCard);
        self.entries = builder.build();
    }
}
impl EightOrBetterLookup { pub fn new() -> Self { let mut lookup = Self { entries: HashMap::new() }; lookup.add_entries(); lookup } }
impl Default for EightOrBetterLookup { fn default() -> Self { Self::new() } }

// --- RegularLookup ---
pub struct RegularLookup { entries: HashMap<(BigUint, bool), Entry> }
impl Lookup for RegularLookup {
    fn rank_order(&self) -> &'static [Rank] { &RankOrder::REGULAR }
    fn entries(&self) -> &HashMap<(BigUint, bool), Entry> { &self.entries }
    fn add_entries(&mut self) {
        let mut builder = LookupBuilder::default();
        builder.add_multisets(self, BTreeMap::from([(1, 5)]), &[false, true], Label::HighCard);
        builder.add_multisets(self, BTreeMap::from([(2, 1), (1, 3)]), &[false], Label::OnePair);
        builder.add_multisets(self, BTreeMap::from([(2, 2), (1, 1)]), &[false], Label::TwoPair);
        builder.add_multisets(self, BTreeMap::from([(3, 1), (1, 2)]), &[false], Label::ThreeOfAKind);
        builder.add_multisets(self, BTreeMap::from([(3, 1), (2, 1)]), &[false], Label::FullHouse);
        builder.add_multisets(self, BTreeMap::from([(4, 1), (1, 1)]), &[false], Label::FourOfAKind);
        self.entries = builder.build();
    }
}
impl RegularLookup { pub fn new() -> Self { let mut lookup = Self { entries: HashMap::new() }; lookup.add_entries(); lookup } }
impl Default for RegularLookup { fn default() -> Self { Self::new() } }

// --- BadugiLookup ---
pub struct BadugiLookup { entries: HashMap<(BigUint, bool), Entry> }
impl Lookup for BadugiLookup {
    fn rank_order(&self) -> &'static [Rank] { &RankOrder::REGULAR }
    fn entries(&self) -> &HashMap<(BigUint, bool), Entry> { &self.entries }
    fn add_entries(&mut self) {
        let mut builder = LookupBuilder::default();
        for i in (1..=4).rev() {
            builder.add_multisets(self, BTreeMap::from([(1, i)]), &[i == 1], Label::HighCard);
        }
        self.entries = builder.build();
    }
    // Override get_key for Badugi-specific validation
    fn get_key(&self, cards_str: &str) -> Result<(BigUint, bool), String> {
        let cards = Card::parse_cards(cards_str)?;
        if !Card::are_rainbow(&cards) {
            return Err("Badugi hands must be rainbow".to_string());
        }
        let ranks: Vec<Rank> = Card::get_ranks(&cards).collect();
        let hash = self.hash_ranks(&ranks);
        let suitedness = Card::are_suited(&cards);
        Ok((hash, suitedness))
    }
}
impl BadugiLookup { pub fn new() -> Self { let mut lookup = Self { entries: HashMap::new() }; lookup.add_entries(); lookup } }
impl Default for BadugiLookup { fn default() -> Self { Self::new() } }

// --- StandardBadugiLookup ---
pub struct StandardBadugiLookup { entries: HashMap<(BigUint, bool), Entry> }
impl Lookup for StandardBadugiLookup {
    fn rank_order(&self) -> &'static [Rank] { &RankOrder::STANDARD }
    fn entries(&self) -> &HashMap<(BigUint, bool), Entry> { &self.entries }
    fn add_entries(&mut self) {
        let mut builder = LookupBuilder::default();
        for i in (1..=4).rev() {
            builder.add_multisets(self, BTreeMap::from([(1, i)]), &[i == 1], Label::HighCard);
        }
        self.entries = builder.build();
    }
    fn get_key(&self, cards_str: &str) -> Result<(BigUint, bool), String> {
        let cards = Card::parse_cards(cards_str)?;
        if !Card::are_rainbow(&cards) {
            return Err("Badugi hands must be rainbow".to_string());
        }
        let ranks: Vec<Rank> = Card::get_ranks(&cards).collect();
        let hash = self.hash_ranks(&ranks);
        let suitedness = Card::are_suited(&cards);
        Ok((hash, suitedness))
    }
}
impl StandardBadugiLookup { pub fn new() -> Self { let mut lookup = Self { entries: HashMap::new() }; lookup.add_entries(); lookup } }
impl Default for StandardBadugiLookup { fn default() -> Self { Self::new() } }

// --- KuhnPokerLookup ---
pub struct KuhnPokerLookup { entries: HashMap<(BigUint, bool), Entry> }
impl Lookup for KuhnPokerLookup {
    fn rank_order(&self) -> &'static [Rank] { &RankOrder::KUHN_POKER }
    fn entries(&self) -> &HashMap<(BigUint, bool), Entry> { &self.entries }
    fn add_entries(&mut self) {
        let mut builder = LookupBuilder::default();
        builder.add_multisets(self, BTreeMap::from([(1, 1)]), &[true], Label::HighCard);
        self.entries = builder.build();
    }
}
impl KuhnPokerLookup { pub fn new() -> Self { let mut lookup = Self { entries: HashMap::new() }; lookup.add_entries(); lookup } }
impl Default for KuhnPokerLookup { fn default() -> Self { Self::new() } }