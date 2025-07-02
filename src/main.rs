use std::collections::BTreeMap;
use pokerkit::games::NoLimitTexasHoldem;
use pokerkit::state::{Automation, Mode, State};

/// Creates a new no-limit Texas Hold'em game state.
fn create_nolimit(n_players: usize) -> Result<State, String> {
    let automations = vec![
        Automation::AntePosting,
        Automation::BetCollection,
        Automation::BlindOrStraddlePosting,
        Automation::HoleDealing, // Automate the dealing
        Automation::HoleCardsShowingOrMucking,
        Automation::HandKilling,
        Automation::ChipsPushing,
        Automation::ChipsPulling,
    ];

    let mut blinds = BTreeMap::new();
    blinds.insert(0, 4); // Small blind
    blinds.insert(1, 8); // Big blind

    let mut starting_stacks = BTreeMap::new();
    for i in 0..n_players {
        starting_stacks.insert(i, 800);
    }
    
    let antes = BTreeMap::new();

    NoLimitTexasHoldem::create_state(
        &automations,
        true,
        antes,
        blinds,
        8,
        starting_stacks,
        n_players,
        Mode::CashGame,
    )
}

fn main() {
    println!("Creating a 6-player No-Limit Texas Hold'em game...");

    match create_nolimit(6) {
        Ok(mut state) => {
            println!("\nGame state created successfully!");
            println!("---------------------------------");
            
            println!("\nInitial State after blinds and dealing:");
            println!("Stacks: {:?}", state.stacks);
            println!("Bets: {:?}", state.bets);
            println!("Hole Cards: {:?}", state.hole_cards);
            println!("Next to act: Player {:?}", state.actor_indices.front().unwrap());
            println!("---------------------------------");

            // --- Pre-flop Betting Round ---
            println!("\n--- Pre-flop Betting ---");

            println!("\nPlayer 2 (UTG) folds.");
            state.fold(None).unwrap();
            println!("Stacks: {:?}", state.stacks);
            println!("Next to act: Player {:?}", state.actor_indices.front().unwrap());

            println!("\nPlayer 3 folds.");
            state.fold(None).unwrap();
            println!("Stacks: {:?}", state.stacks);
            println!("Next to act: Player {:?}", state.actor_indices.front().unwrap());
            
            println!("\nPlayer 4 raises to 25.");
            state.complete_bet_or_raise_to(25, None).unwrap();
            println!("Stacks: {:?}", state.stacks);
            println!("Bets: {:?}", state.bets);
            println!("Next to act: Player {:?}", state.actor_indices.front().unwrap());

            println!("\nPlayer 5 (Button) folds.");
            state.fold(None).unwrap();
            println!("Stacks: {:?}", state.stacks);
            println!("Next to act: Player {:?}", state.actor_indices.front().unwrap());

            println!("\nPlayer 0 (SB) calls.");
            state.check_or_call(None).unwrap();
            println!("Stacks: {:?}", state.stacks);
            println!("Bets: {:?}", state.bets);
            println!("Next to act: Player {:?}", state.actor_indices.front().unwrap());
            
            println!("\nPlayer 1 (BB) re-raises to 75.");
            state.complete_bet_or_raise_to(75, None).unwrap();
            println!("Stacks: {:?}", state.stacks);
            println!("Bets: {:?}", state.bets);
            println!("Next to act: Player {:?}", state.actor_indices.front().unwrap());

            println!("\nPlayer 4 calls.");
            state.check_or_call(None).unwrap();
            println!("Stacks: {:?}", state.stacks);
            println!("Bets: {:?}", state.bets);
            println!("Next to act: Player {:?}", state.actor_indices.front().unwrap());

            println!("\nPlayer 0 calls.");
            state.check_or_call(None).unwrap();
            
            println!("\n--- End of Pre-flop Betting ---");
            println!("Final Stacks: {:?}", state.stacks);
            println!("Total Pot: {}", state.pots().iter().map(|p| p.amount()).sum::<i64>());
            println!("---------------------------------");
        }
        Err(e) => {
            eprintln!("Failed to create game state: {}", e);
        }
    }
}
