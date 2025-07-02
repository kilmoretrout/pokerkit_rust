# pokerkit_rust

Rust port of Juho Kims Python package pokerkit (https://github.com/uoftcprg/pokerkit/tree/main).  

In development. Some functionality may not yet be working.  If you find this useful please consider donating some XMR to: 

4AFKKNFaCuuLv1BtirNDqqbRirwxV6MhoAtbcB9fmso3gqRe3WW6dthfcd8Rym5dqbQBT1pDMmwRtjchyzCzbCKcMkYnpRp

```
// main.rs:

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
```

```
cargo build
target/debug/pokerkit 
Creating a 6-player No-Limit Texas Hold'em game...

Game state created successfully!
---------------------------------

Initial State after blinds and dealing:
Stacks: [796, 792, 800, 800, 800, 800]
Bets: [4, 8, 0, 0, 0, 0]
Hole Cards: [[Card { rank: Jack, suit: Heart }, Card { rank: Jack, suit: Diamond }], [Card { rank: Ten, suit: Heart }, Card { rank: Four, suit: Diamond }], [Card { rank: Ten, suit: Club }, Card { rank: Deuce, suit: Club }], [Card { rank: Five, suit: Club }, Card { rank: King, suit: Spade }], [Card { rank: King, suit: Heart }, Card { rank: Eight, suit: Heart }], [Card { rank: Nine, suit: Spade }, Card { rank: Jack, suit: Spade }]]
Next to act: Player 2
---------------------------------

--- Pre-flop Betting ---

Player 2 (UTG) folds.
Stacks: [796, 792, 800, 800, 800, 800]
Next to act: Player 3

Player 3 folds.
Stacks: [796, 792, 800, 800, 800, 800]
Next to act: Player 4

Player 4 raises to 25.
Stacks: [796, 792, 800, 800, 775, 800]
Bets: [4, 8, 0, 0, 25, 0]
Next to act: Player 5

Player 5 (Button) folds.
Stacks: [796, 792, 800, 800, 775, 800]
Next to act: Player 0

Player 0 (SB) calls.
Stacks: [775, 792, 800, 800, 775, 800]
Bets: [25, 8, 0, 0, 25, 0]
Next to act: Player 1

Player 1 (BB) re-raises to 75.
Stacks: [775, 725, 800, 800, 775, 800]
Bets: [25, 75, 0, 0, 25, 0]
Next to act: Player 4

Player 4 calls.
Stacks: [775, 725, 800, 800, 725, 800]
Bets: [25, 75, 0, 0, 75, 0]
Next to act: Player 0

Player 0 calls.

--- End of Pre-flop Betting ---
Final Stacks: [725, 725, 800, 800, 725, 800]
Total Pot: 225
---------------------------------
```
