mod game;
mod game_json;
mod official;
mod skater;
mod team;
mod word_list;

use std::path::PathBuf;

use crate::skater::Skater;
use clap::Parser;
use game::Game;
use official::Official;
use rand::{rngs::StdRng, SeedableRng};
use team::Team;

#[derive(Parser, Debug)]
struct CommandLineArguments {
    /// The seed used to generate the game
    #[arg(short = 's', long = "seed")]
    random_seed: Option<u64>,

    /// The file path to output the game JSON to
    #[arg(short = 'j', long = "gameJson")]
    json_output_path: Option<PathBuf>,

    /// The file path to output the events YAML to
    #[arg(short = 'y', long = "eventsYaml")]
    yaml_output_path: Option<PathBuf>,
}

fn print_skater(skater: &Skater) {
    println!("{} ({}) - {:?}", skater.name, skater.number, skater.favored_position);
}

fn print_team(team: &Team) {
    println!("{} - {}", team.name, team.color);
    for skater in team.roster.iter() {
        print_skater(&skater);
    }
}

fn print_official(official: &Official) {
    if official.is_head {
        println!("{} - {:?} (Head)", official.name, official.role);
    } else {
        println!("{} - {:?}", official.name, official.role);
    }
}

fn main() {
    let arguments = CommandLineArguments::parse();

    let random = match arguments.random_seed {
        None => {
            StdRng::from_entropy()
        },
        Some(seed) => {
            let seed_bytes = u64::to_le_bytes(seed);
            let mut seed_buffer = [0; 32];
            seed_buffer[..seed_bytes.len()].copy_from_slice(&seed_bytes);
        
            StdRng::from_seed(seed_buffer)
        }
    };

    let mut game = Game::random(random);

    println!("Home");
    println!("----");
    print_team(&game.home_team.details);

    println!();
    println!("Away");
    println!("----");
    print_team(&game.away_team.details);

    println!();
    println!("Officials");
    println!("---------");
    for o in game.officials.iter() {
        print_official(&o);
    }

    println!();

    game.run();
    
    if let Some(json_path) = arguments.json_output_path {
        match std::fs::write(&json_path, game.game_json.to_string()) {
            Ok(_) => {
                println!("Game JSON written to {}", json_path.to_str().unwrap());
            },
            Err(e) => {
                println!("Error writing game JSON: {}", e);
            }
        }
    }

}
