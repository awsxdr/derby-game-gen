use rand::{Rng, rngs::StdRng};
use uuid::Uuid;

use crate::word_list;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position {
    Jammer,
    Pivot,
    Blocker,
}

#[derive(Clone, Debug)]
pub struct Skater {
    pub id: Uuid,
    pub name: String,
    pub number: String,
    pub favored_position: Position,
    pub base_speed: f32,
    pub penalty_chance: f64,
}

impl Skater {
    pub fn random(random_source: &mut StdRng) -> Skater {
        Skater {
            id: Uuid::new_v4(),
            name: Self::get_random_name(random_source),
            number: Self::get_random_number(random_source),
            favored_position: Self::get_random_position(random_source),
            base_speed: Self::get_random_speed(random_source),
            penalty_chance: Self::get_random_penalty_chance(random_source),
        }
    }

    fn get_random_name(random_source: &mut StdRng) -> String {
        let first_name = word_list::NAME_ADJECTIVES[random_source.gen_range(0..word_list::NAME_ADJECTIVES.len())];
        let last_name = word_list::NAME_NOUNS[random_source.gen_range(0..word_list::NAME_NOUNS.len())];

        first_name.to_owned() + " " + last_name
    }

    fn get_random_number(random_source: &mut StdRng) -> String {
        let characters: Vec<String> = vec![0; random_source.gen_range(1..=4)].iter().map(|_| {
            random_source.gen_range(0..=9).to_string()
        }).collect();
        characters.join("")
    }

    fn get_random_position(random_source: &mut StdRng) -> Position {
        match random_source.gen_range(0..3) {
            0 => Position::Jammer,
            1 => Position::Pivot,
            _ => Position::Blocker
        }
    }

    fn get_random_speed(random_source: &mut StdRng) -> f32 {
        random_source.gen_range(15.0..20.0)
    }

    fn get_random_penalty_chance(random_source: &mut StdRng) -> f64 {
        random_source.gen_range(1.0/2000.0..1.0/1000.0)
    }
}

