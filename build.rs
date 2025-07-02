use phf_codegen::Map;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("rank_multipliers.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41];
    let ranks = [
        ("Ace", 'A'), ("Deuce", '2'), ("Trey", '3'), ("Four", '4'),
        ("Five", '5'), ("Six", '6'), ("Seven", '7'), ("Eight", '8'),
        ("Nine", '9'), ("Ten", 'T'), ("Jack", 'J'), ("Queen", 'Q'),
        ("King", 'K'),
    ];

    let mut map = Map::new();
    for (i, &(_name, val)) in ranks.iter().enumerate() {
        map.entry(val, &primes[i].to_string());
    }

    writeln!(
        &mut file,
        "static RANK_MULTIPLIERS: phf::Map<char, u64> = {};",
        map.build()
    )
    .unwrap();
}