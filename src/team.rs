use rand::{Rng, rngs::StdRng};
use uuid::Uuid;

use crate::{skater::Skater, word_list};

#[derive(Clone)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub roster: Vec<Skater>,
    pub color: String,
}

impl Team {
    pub fn random(random_source: &mut StdRng) -> Team {
        Team {
            id: Uuid::new_v4(),
            name: Self::get_random_name(random_source),
            roster: Self::get_random_roster(random_source),
            color: Self::get_random_color(random_source),
        }
    }

    fn get_random_name(random_source: &mut StdRng) -> String {
        word_list::PLACE_NAMES[random_source.gen_range(0..word_list::PLACE_NAMES.len())].to_owned() + " Roller Derby"
    }

    fn get_random_roster(random_source: &mut StdRng) -> Vec<Skater> {
        let roster_size = random_source.gen_range(8..=15);
        let mut roster: Vec<Skater> = Vec::new();

        for _ in 0..roster_size {
            loop {
                let skater = Skater::random(random_source);

                if !roster.iter().any(|i| i.number == skater.number) {
                    roster.push(skater);
                    break;
                }
            }
        };

        roster.sort_by(|a, b| a.number.cmp(&b.number));

        roster
    }

    fn get_random_color(random_source: &mut StdRng) -> String {
        word_list::COLORS[random_source.gen_range(0..word_list::COLORS.len())].to_string()
    }
}