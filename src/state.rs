// Implements the core poker state machine and related data structures.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::fmt;

use crate::hands::{Hand, HandType};
use crate::lookups::{Label, Lookup};
use crate::utilities::{
    clean_values, div_mod, max_or_none, min_or_none, rake, shuffled, sign, Card, Deck, RankOrder,
    Suit,
};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;
use strum_macros::{Display, EnumString};

// Enums defining game parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum BettingStructure {
    #[strum(serialize = "Fixed-limit")]
    FixedLimit,
    #[strum(serialize = "Pot-limit")]
    PotLimit,
    #[strum(serialize = "No-limit")]
    NoLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum Opening {
    Position,
    LowCard,
    HighCard,
    LowHand,
    HighHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
pub enum Automation {
    AntePosting,
    BetCollection,
    BlindOrStraddlePosting,
    CardBurning,
    HoleDealing,
    BoardDealing,
    RunoutCountSelection,
    HoleCardsShowingOrMucking,
    HandKilling,
    ChipsPushing,
    ChipsPulling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Display)]
pub enum Mode {
    Tournament,
    #[strum(serialize = "Cash-game")]
    CashGame,
}

/// Represents a single street (betting round) in a poker game.
#[derive(Debug, Clone)]
pub struct Street {
    pub card_burning_status: bool,
    pub hole_dealing_statuses: Vec<bool>,
    pub board_dealing_count: usize,
    pub draw_status: bool,
    pub opening: Opening,
    pub min_completion_betting_or_raising_amount: i64,
    pub max_completion_betting_or_raising_count: Option<usize>,
}

impl Street {
    pub fn new(
        card_burning_status: bool,
        hole_dealing_statuses: Vec<bool>,
        board_dealing_count: usize,
        draw_status: bool,
        opening: Opening,
        min_completion_betting_or_raising_amount: i64,
        max_completion_betting_or_raising_count: Option<usize>,
    ) -> Result<Self, String> {
        if !hole_dealing_statuses.is_empty() && draw_status {
            return Err("Only one of hole dealing or drawing is permitted.".to_string());
        }
        if min_completion_betting_or_raising_amount <= 0 {
            return Err("Non-positive minimum bet/raise amount supplied.".to_string());
        }
        Ok(Self {
            card_burning_status,
            hole_dealing_statuses,
            board_dealing_count,
            draw_status,
            opening,
            min_completion_betting_or_raising_amount,
            max_completion_betting_or_raising_count,
        })
    }
}

/// Represents a pot or a side pot.
#[derive(Debug, Clone)]
pub struct Pot {
    pub raked_amount: i64,
    pub unraked_amount: i64,
    pub player_indices: Vec<usize>,
}

impl Pot {
    pub fn amount(&self) -> i64 {
        self.raked_amount + self.unraked_amount
    }
}

// Represents all possible operations within a game state.
#[derive(Debug, Clone)]
pub enum Operation {
    AntePosting(AntePosting),
    BetCollection(BetCollection),
    BlindOrStraddlePosting(BlindOrStraddlePosting),
    CardBurning(CardBurning),
    HoleDealing(HoleDealing),
    BoardDealing(BoardDealing),
    StandingPatOrDiscarding(StandingPatOrDiscarding),
    Folding(Folding),
    CheckingOrCalling(CheckingOrCalling),
    BringInPosting(BringInPosting),
    CompletionBettingOrRaisingTo(CompletionBettingOrRaisingTo),
    RunoutCountSelection(RunoutCountSelection),
    HoleCardsShowingOrMucking(HoleCardsShowingOrMucking),
    HandKilling(HandKilling),
    ChipsPushing(ChipsPushing),
    ChipsPulling(ChipsPulling),
    NoOperation(NoOperation),
}

#[derive(Debug, Clone)] pub struct AntePosting { pub player_index: usize, pub amount: i64, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct BetCollection { pub bets: Vec<i64>, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct BlindOrStraddlePosting { pub player_index: usize, pub amount: i64, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct CardBurning { pub card: Card, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct HoleDealing { pub player_index: usize, pub cards: Vec<Card>, pub statuses: Vec<bool>, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct BoardDealing { pub cards: Vec<Card>, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct StandingPatOrDiscarding { pub player_index: usize, pub cards: Vec<Card>, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct Folding { pub player_index: usize, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct CheckingOrCalling { pub player_index: usize, pub amount: i64, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct BringInPosting { pub player_index: usize, pub amount: i64, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct CompletionBettingOrRaisingTo { pub player_index: usize, pub amount: i64, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct RunoutCountSelection { pub player_index: usize, pub runout_count: Option<usize>, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct HoleCardsShowingOrMucking { pub player_index: usize, pub hole_cards: Vec<Card>, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct HandKilling { pub player_index: usize, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct ChipsPushing { pub amounts: Vec<i64>, pub pot_index: usize, pub board_index: Option<usize>, pub hand_type_index: Option<usize>, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct ChipsPulling { pub player_index: usize, pub amount: i64, pub commentary: Option<String> }
#[derive(Debug, Clone)] pub struct NoOperation { pub commentary: Option<String> }

/// The main struct representing the state of a poker game.
pub struct State {
    // Configuration
    pub automations: HashSet<Automation>,
    pub deck: Vec<Card>,
    pub hand_types: Vec<HandType>,
    pub streets: Vec<Street>,
    pub betting_structure: BettingStructure,
    pub ante_trimming_status: bool,
    pub antes: Vec<i64>,
    pub blinds_or_straddles: Vec<i64>,
    pub bring_in: i64,
    pub starting_stacks: Vec<i64>,
    pub player_count: usize,
    pub mode: Mode,
    pub starting_board_count: usize,
    pub divmod: fn(i64, i64) -> (i64, i64),
    pub rake: fn(&State, i64) -> (i64, i64),

    // Game state
    pub deck_cards: VecDeque<Card>,
    pub board_cards: Vec<Vec<Card>>,
    pub mucked_cards: Vec<Card>,
    pub burn_cards: Vec<Card>,
    pub statuses: Vec<bool>,
    pub bets: Vec<i64>,
    pub stacks: Vec<i64>,
    pub payoffs: Vec<i64>,
    pub hole_cards: Vec<Vec<Card>>,
    pub hole_card_statuses: Vec<Vec<bool>>,
    pub discarded_cards: Vec<Vec<Card>>,
    pub street_index: Option<usize>,
    pub status: bool,
    pub operations: Vec<Operation>,

    // Phase-specific state
    pub ante_posting_statuses: Vec<bool>,
    pub bet_collection_status: bool,
    pub blind_or_straddle_posting_statuses: Vec<bool>,
    pub card_burning_status: bool,
    pub hole_dealing_statuses: Vec<VecDeque<bool>>,
    pub board_dealing_counts: Vec<usize>,
    pub standing_pat_or_discarding_statuses: Vec<bool>,
    pub actor_indices: VecDeque<usize>,
    pub opener_index: Option<usize>,
    pub bring_in_status: bool,
    pub completion_status: bool,
    pub completion_betting_or_raising_amount: i64,
    pub completion_betting_or_raising_count: usize,
    pub acted_player_indices: HashSet<usize>,
    pub runout_count: Option<usize>,
    pub showdown_indices: VecDeque<usize>,
}

pub struct StateBuilder {
    automations: HashSet<Automation>,
    deck: Vec<Card>,
    hand_types: Vec<HandType>,
    streets: Vec<Street>,
    betting_structure: BettingStructure,
    ante_trimming_status: bool,
    raw_antes: BTreeMap<usize, i64>,
    raw_blinds_or_straddles: BTreeMap<usize, i64>,
    bring_in: i64,
    raw_starting_stacks: BTreeMap<usize, i64>,
    player_count: usize,
    mode: Mode,
    starting_board_count: usize,
    divmod: fn(i64, i64) -> (i64, i64),
    rake: fn(&State, i64) -> (i64, i64),
}

impl StateBuilder {
    pub fn new(player_count: usize) -> Self {
        Self {
            automations: HashSet::new(),
            deck: Deck::standard(),
            hand_types: vec![HandType::StandardHighHand],
            streets: Vec::new(),
            betting_structure: BettingStructure::NoLimit,
            ante_trimming_status: false,
            raw_antes: BTreeMap::new(),
            raw_blinds_or_straddles: BTreeMap::new(),
            bring_in: 0,
            raw_starting_stacks: BTreeMap::new(),
            player_count,
            mode: Mode::Tournament,
            starting_board_count: 1,
            divmod: div_mod,
            rake,
        }
    }

    pub fn automations(mut self, automations: &[Automation]) -> Self { self.automations = automations.iter().cloned().collect(); self }
    pub fn deck(mut self, deck: Vec<Card>) -> Self { self.deck = deck; self }
    pub fn hand_types(mut self, hand_types: Vec<HandType>) -> Self { self.hand_types = hand_types; self }
    pub fn streets(mut self, streets: Vec<Street>) -> Self { self.streets = streets; self }
    pub fn betting_structure(mut self, betting_structure: BettingStructure) -> Self { self.betting_structure = betting_structure; self }
    pub fn ante_trimming_status(mut self, ante_trimming_status: bool) -> Self { self.ante_trimming_status = ante_trimming_status; self }
    pub fn raw_antes(mut self, raw_antes: BTreeMap<usize, i64>) -> Self { self.raw_antes = raw_antes; self }
    pub fn raw_blinds_or_straddles(mut self, raw_blinds_or_straddles: BTreeMap<usize, i64>) -> Self { self.raw_blinds_or_straddles = raw_blinds_or_straddles; self }
    pub fn bring_in(mut self, bring_in: i64) -> Self { self.bring_in = bring_in; self }
    pub fn raw_starting_stacks(mut self, raw_starting_stacks: BTreeMap<usize, i64>) -> Self { self.raw_starting_stacks = raw_starting_stacks; self }
    pub fn mode(mut self, mode: Mode) -> Self { self.mode = mode; self }

    pub fn build(self) -> Result<State, String> {
        if self.player_count < 2 { return Err("Player count must be at least 2".to_string()); }
        if self.streets.is_empty() { return Err("Streets cannot be empty".to_string()); }
        
        let antes = clean_values(&self.raw_antes, self.player_count);
        let blinds_or_straddles = clean_values(&self.raw_blinds_or_straddles, self.player_count);
        let starting_stacks = clean_values(&self.raw_starting_stacks, self.player_count);

        let mut state = State {
            automations: self.automations,
            deck: self.deck.clone(),
            hand_types: self.hand_types,
            streets: self.streets,
            betting_structure: self.betting_structure,
            ante_trimming_status: self.ante_trimming_status,
            antes,
            blinds_or_straddles,
            bring_in: self.bring_in,
            starting_stacks: starting_stacks.clone(),
            player_count: self.player_count,
            mode: self.mode,
            starting_board_count: self.starting_board_count,
            divmod: self.divmod,
            rake: self.rake,
            deck_cards: VecDeque::from(shuffled(&self.deck)),
            board_cards: vec![Vec::new(); self.starting_board_count],
            mucked_cards: Vec::new(),
            burn_cards: Vec::new(),
            statuses: vec![true; self.player_count],
            bets: vec![0; self.player_count],
            stacks: starting_stacks,
            payoffs: vec![0; self.player_count],
            hole_cards: vec![Vec::new(); self.player_count],
            hole_card_statuses: vec![Vec::new(); self.player_count],
            discarded_cards: vec![Vec::new(); self.player_count],
            street_index: None,
            status: true,
            operations: Vec::new(),
            ante_posting_statuses: vec![false; self.player_count],
            bet_collection_status: false,
            blind_or_straddle_posting_statuses: vec![false; self.player_count],
            card_burning_status: false,
            hole_dealing_statuses: vec![VecDeque::new(); self.player_count],
            board_dealing_counts: vec![0; self.starting_board_count],
            standing_pat_or_discarding_statuses: vec![false; self.player_count],
            actor_indices: VecDeque::new(),
            opener_index: None,
            bring_in_status: false,
            completion_status: false,
            completion_betting_or_raising_amount: 0,
            completion_betting_or_raising_count: 0,
            acted_player_indices: HashSet::new(),
            runout_count: None,
            showdown_indices: VecDeque::new(),
        };

        state.begin();
        Ok(state)
    }
}


impl State {
    // Core state machine logic
    fn begin(&mut self) { self.begin_ante_posting(); }
    fn end(&mut self) { self.status = false; }
    
    // Game flow state transitions
    fn begin_ante_posting(&mut self) { 
        for i in 0..self.player_count {
            self.ante_posting_statuses[i] = self.get_effective_ante(i) > 0;
        }
        self.run_ante_posting_automation();
    }
    fn run_ante_posting_automation(&mut self) {
        if self.automations.contains(&Automation::AntePosting) {
            let indices: Vec<usize> = self.ante_poster_indices().collect();
            for i in indices {
                self.post_ante(Some(i), None).unwrap();
            }
        }
        if !self.ante_posting_statuses.iter().any(|&s| s) {
            self.end_ante_posting();
        }
    }
    fn end_ante_posting(&mut self) { self.begin_bet_collection(); }

    fn begin_bet_collection(&mut self) {
        self.bet_collection_status = self.bets.iter().any(|&b| b > 0);
        self.run_bet_collection_automation();
    }
    fn run_bet_collection_automation(&mut self) {
        if self.automations.contains(&Automation::BetCollection) && self.bet_collection_status {
            self.collect_bets(None).unwrap();
        }
        if !self.bet_collection_status {
            self.end_bet_collection();
        }
    }
    fn end_bet_collection(&mut self) {
        if self.statuses.iter().filter(|&&s| s).count() <= 1 {
            self.begin_chips_pushing();
        } else if self.street_index.is_none() {
            self.begin_blind_or_straddle_posting();
        } else if self.street_index == Some(self.streets.len() - 1) { // is last street
            self.begin_showdown();
        } else {
            self.begin_dealing();
        }
    }

    fn begin_blind_or_straddle_posting(&mut self) {
        for i in 0..self.player_count {
            self.blind_or_straddle_posting_statuses[i] = self.get_effective_blind_or_straddle(i) > 0;
        }
        self.run_blind_or_straddle_posting_automation();
    }
    fn run_blind_or_straddle_posting_automation(&mut self) {
        if self.automations.contains(&Automation::BlindOrStraddlePosting) {
            let indices: Vec<usize> = self.blind_or_straddle_poster_indices().collect();
            for i in indices {
                self.post_blind_or_straddle(Some(i), None).unwrap();
            }
        }
        if !self.blind_or_straddle_posting_statuses.iter().any(|&s| s) {
            self.end_blind_or_straddle_posting();
        }
    }
    fn end_blind_or_straddle_posting(&mut self) { self.begin_dealing(); }

    fn begin_dealing(&mut self) {
        let new_street_index = self.street_index.map_or(0, |i| i + 1);
        self.street_index = Some(new_street_index);
        let street = self.streets[new_street_index].clone();

        self.card_burning_status = street.card_burning_status;
        for i in 0..self.player_count {
            if self.statuses[i] {
                self.hole_dealing_statuses[i].extend(street.hole_dealing_statuses.iter());
                self.standing_pat_or_discarding_statuses[i] = street.draw_status;
            }
        }
        self.board_dealing_counts = vec![street.board_dealing_count; self.starting_board_count];
        self.run_dealing_automation();
    }
    fn run_dealing_automation(&mut self) {
        let dealing_done = !self.card_burning_status 
            && !self.hole_dealing_statuses.iter().any(|q| !q.is_empty())
            && !self.board_dealing_counts.iter().any(|&c| c > 0)
            && !self.standing_pat_or_discarding_statuses.iter().any(|&s| s);
        
        if dealing_done {
            self.end_dealing();
        } else if self.automations.contains(&Automation::CardBurning) && self.can_burn_card(None) {
            self.burn_card(None, None).unwrap();
        } else if self.automations.contains(&Automation::HoleDealing) && self.hole_dealee_index().is_some() {
            while self.hole_dealee_index().is_some() {
                self.deal_hole(None, None, None).unwrap();
            }
        } // ... and so on for other dealing automations
    }
    fn end_dealing(&mut self) { self.begin_betting(); }

    fn begin_betting(&mut self) {
        self.opener_index = None;
        self.acted_player_indices.clear();
        self.completion_betting_or_raising_amount = 0;
        self.completion_betting_or_raising_count = 0;
    
        let street = self.streets[self.street_index.unwrap()].clone();
    
        // Determine the first player to act.
        let opener_index = match street.opening {
            Opening::Position => {
                if self.street_index == Some(0) { // Pre-flop
                    let bb_index = self.blinds_or_straddles.iter().rposition(|&b| b > 0).unwrap_or(self.player_count - 1);
                    let mut current = (bb_index + 1) % self.player_count;
                    // Find the next active player
                    while !self.statuses[current] {
                        current = (current + 1) % self.player_count;
                    }
                    current
                } else { // Post-flop
                    (0..self.player_count).find(|&i| self.statuses[i]).unwrap_or(0)
                }
            }
            _ => unimplemented!("Opening type {:?} is not yet implemented", street.opening),
        };
    
        self.opener_index = Some(opener_index);
    
        // Set up the actor queue.
        self.actor_indices = (0..self.player_count)
            .cycle()
            .skip(opener_index)
            .take(self.player_count)
            .filter(|&i| self.statuses[i] && self.stacks[i] > 0)
            .collect();
    
        self.run_betting_automation();
    }
    fn run_betting_automation(&mut self) {
        let active_players: Vec<usize> = (0..self.player_count).filter(|&i| self.statuses[i]).collect();
        if active_players.len() <= 1 {
            self.end_betting();
            return;
        }
    
        let max_bet = self.bets.iter().max().cloned().unwrap_or(0);
        let all_acted = active_players.iter().all(|i| self.acted_player_indices.contains(i));
        let bets_settled = active_players.iter().all(|&i| self.bets[i] == max_bet || self.stacks[i] == 0);
    
        if all_acted && bets_settled {
            self.end_betting();
        }
    }
    
    fn end_betting(&mut self) {
        self.actor_indices.clear();
        self.begin_bet_collection();
    }

    fn begin_showdown(&mut self) { /* ... */ }
    fn begin_chips_pushing(&mut self) { /* ... */ }

    // Helper methods
    pub fn get_effective_ante(&self, player_index: usize) -> i64 {
        let ante = if self.player_count == 2 { self.antes[1 - player_index] } else { self.antes[player_index] };
        ante.min(self.starting_stacks[player_index])
    }
    
    pub fn get_effective_blind_or_straddle(&self, player_index: usize) -> i64 {
        let blind = if self.player_count == 2 { self.blinds_or_straddles[1 - player_index].abs() } else { self.blinds_or_straddles[player_index].abs() };
        blind.min(self.starting_stacks[player_index] - self.get_effective_ante(player_index))
    }

    pub fn ante_poster_indices(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.player_count).filter(move |&i| self.ante_posting_statuses[i])
    }
    
    pub fn blind_or_straddle_poster_indices(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.player_count).filter(move |&i| self.blind_or_straddle_posting_statuses[i])
    }
    
    pub fn hole_dealee_index(&self) -> Option<usize> {
        (0..self.player_count).filter(|&i| !self.hole_dealing_statuses[i].is_empty()).max_by_key(|&i| (self.hole_dealing_statuses[i].len(), -(i as isize)))
    }

    // Public API for actions
    pub fn post_ante(&mut self, player_index: Option<usize>, commentary: Option<String>) -> Result<AntePosting, String> {
        let player_index = player_index.unwrap_or_else(|| self.ante_poster_indices().next().unwrap());
        if !self.ante_posting_statuses[player_index] { return Err("Player cannot post ante".to_string()); }
        
        let amount = self.get_effective_ante(player_index);
        self.ante_posting_statuses[player_index] = false;
        self.bets[player_index] = amount;
        self.stacks[player_index] -= amount;
        self.payoffs[player_index] -= amount;
        
        let op = AntePosting { player_index, amount, commentary };
        self.operations.push(Operation::AntePosting(op.clone()));
        Ok(op)
    }
    
    pub fn collect_bets(&mut self, commentary: Option<String>) -> Result<BetCollection, String> {
        if !self.bet_collection_status { return Err("No bets to collect".to_string()); }
        self.bet_collection_status = false;
        let bets = self.bets.clone();
        self.bets.iter_mut().for_each(|b| *b = 0);
        let op = BetCollection { bets, commentary };
        self.operations.push(Operation::BetCollection(op.clone()));
        Ok(op)
    }
    
    pub fn post_blind_or_straddle(&mut self, player_index: Option<usize>, commentary: Option<String>) -> Result<BlindOrStraddlePosting, String> {
        let player_index = player_index.unwrap_or_else(|| self.blind_or_straddle_poster_indices().next().unwrap());
        if !self.blind_or_straddle_posting_statuses[player_index] { return Err("Player cannot post blind/straddle".to_string()); }

        let amount = self.get_effective_blind_or_straddle(player_index);
        self.blind_or_straddle_posting_statuses[player_index] = false;
        self.bets[player_index] += amount;
        self.stacks[player_index] -= amount;
        self.payoffs[player_index] -= amount;
        
        let op = BlindOrStraddlePosting { player_index, amount, commentary };
        self.operations.push(Operation::BlindOrStraddlePosting(op.clone()));
        Ok(op)
    }
    
    pub fn can_burn_card(&self, _card: Option<Card>) -> bool { self.card_burning_status }
    
    pub fn burn_card(&mut self, card: Option<Card>, commentary: Option<String>) -> Result<CardBurning, String> {
        if !self.can_burn_card(card) { return Err("Cannot burn card now".to_string()); }
        let card_to_burn = card.unwrap_or_else(|| self.deck_cards.pop_front().unwrap());
        self.card_burning_status = false;
        self.burn_cards.push(card_to_burn);
        let op = CardBurning { card: card_to_burn, commentary };
        self.operations.push(Operation::CardBurning(op.clone()));
        self.run_dealing_automation();
        Ok(op)
    }

    pub fn deal_hole(&mut self, cards: Option<Vec<Card>>, player_index: Option<usize>, commentary: Option<String>) -> Result<HoleDealing, String> {
        let player_index = player_index.or_else(|| self.hole_dealee_index()).ok_or("No player to deal to")?;
        let num_to_deal = cards.as_ref().map_or(1, |c| c.len());
        if self.hole_dealing_statuses[player_index].len() < num_to_deal { return Err("Not enough hole cards to be dealt to player".to_string()); }

        let dealt_cards = cards.unwrap_or_else(|| self.deck_cards.drain(..num_to_deal).collect());
        let mut statuses = Vec::new();
        for card in &dealt_cards {
            let status = self.hole_dealing_statuses[player_index].pop_front().unwrap();
            self.hole_cards[player_index].push(*card);
            self.hole_card_statuses[player_index].push(status);
            statuses.push(status);
        }
        
        let op = HoleDealing { player_index, cards: dealt_cards, statuses, commentary };
        self.operations.push(Operation::HoleDealing(op.clone()));
        self.run_dealing_automation();
        Ok(op)
    }

    fn actor_index(&self) -> Result<usize, String> {
        self.actor_indices.front().cloned().ok_or_else(|| "There is no player to act.".to_string())
    }

    fn advance_actor(&mut self) {
        if let Some(player_index) = self.actor_indices.pop_front() {
            self.acted_player_indices.insert(player_index);
        }
    }

    pub fn fold(&mut self, commentary: Option<String>) -> Result<Folding, String> {
        let player_index = self.actor_index()?;
        self.advance_actor();
        self.statuses[player_index] = false;
        self.mucked_cards.append(&mut self.hole_cards[player_index]);
        let op = Folding { player_index, commentary };
        self.operations.push(Operation::Folding(op.clone()));
        self.run_betting_automation();
        Ok(op)
    }

    pub fn check_or_call(&mut self, commentary: Option<String>) -> Result<CheckingOrCalling, String> {
        let player_index = self.actor_index()?;
        let max_bet = *self.bets.iter().max().unwrap_or(&0);
        let amount_to_call = (max_bet - self.bets[player_index]).min(self.stacks[player_index]);
        
        self.advance_actor();
        self.bets[player_index] += amount_to_call;
        self.stacks[player_index] -= amount_to_call;
        self.payoffs[player_index] -= amount_to_call;

        let op = CheckingOrCalling { player_index, amount: amount_to_call, commentary };
        self.operations.push(Operation::CheckingOrCalling(op.clone()));
        self.run_betting_automation();
        Ok(op)
    }

    pub fn complete_bet_or_raise_to(&mut self, amount: i64, commentary: Option<String>) -> Result<CompletionBettingOrRaisingTo, String> {
        let player_index = self.actor_index()?;
        let delta = amount - self.bets[player_index];
        
        self.bets[player_index] = amount;
        self.stacks[player_index] -= delta;
        self.payoffs[player_index] -= delta;
        
        self.opener_index = Some(player_index);
        self.completion_betting_or_raising_count += 1;
        
        // Action re-opens for all other active players.
        self.actor_indices = (0..self.player_count)
            .cycle()
            .skip(player_index + 1)
            .take(self.player_count)
            .filter(|&i| self.statuses[i] && self.stacks[i] > 0)
            .collect();
        self.acted_player_indices.clear();
        self.acted_player_indices.insert(player_index);

        let op = CompletionBettingOrRaisingTo { player_index, amount, commentary };
        self.operations.push(Operation::CompletionBettingOrRaisingTo(op.clone()));
        self.run_betting_automation();
        Ok(op)
    }

    pub fn pots(&self) -> Vec<Pot> {
        let mut contributions: Vec<i64> = self.payoffs.iter().map(|p| -p).collect();
        let mut pots = Vec::new();

        for i in 0..self.player_count {
            contributions[i] += self.bets[i];
        }

        let mut last_contribution = 0;
        let mut unique_contributions: Vec<i64> = contributions
            .iter()
            .cloned()
            .filter(|&c| c > 0)
            .collect();
        unique_contributions.sort_unstable();
        unique_contributions.dedup();

        for &contribution in &unique_contributions {
            let mut pot_amount = 0;
            let mut pot_player_indices = Vec::new();

            for i in 0..self.player_count {
                if contributions[i] >= contribution {
                    pot_amount += contribution - last_contribution;
                }
                if self.statuses[i] && contributions[i] >= contribution {
                    pot_player_indices.push(i);
                }
            }
            
            if pot_amount > 0 {
                let (raked, unraked) = (self.rake)(self, pot_amount);
                pots.push(Pot {
                    raked_amount: raked,
                    unraked_amount: unraked,
                    player_indices: pot_player_indices,
                });
            }
            last_contribution = contribution;
        }
        pots
    }
}
