use rand::{Rng, rngs::StdRng};
use uuid::Uuid;

use crate::word_list;

#[derive(Clone, Debug, PartialEq)]
pub enum OfficialRole {
    PenaltyLineupTracker,
    PenaltyWrangler,
    InsideWhiteboard,
    JamTimer,
    Scorekeeper,
    ScoreboardOperator,
    PenaltyBoxManager,
    PenaltyBoxTimer,
    InsidePackReferee,
    OutsidePackReferee,
    JammerReferee,
}

#[derive(Clone)]
pub struct Official {
    pub id: Uuid,
    pub name: String,
    pub is_head: bool,
    pub role: OfficialRole,
}

impl Official {
    pub fn random(random_source: &mut StdRng, role: OfficialRole, is_head: bool) -> Official {
        Official {
            id: Uuid::new_v4(),
            name: Self::get_random_name(random_source),
            is_head,
            role
        }
    }

    pub fn random_crew(random_source: &mut StdRng) -> Vec<Official> {
        vec![
            Self::random(random_source, OfficialRole::PenaltyLineupTracker, true),
            Self::random(random_source, OfficialRole::PenaltyLineupTracker, false),
            Self::random(random_source, OfficialRole::PenaltyWrangler, false),
            Self::random(random_source, OfficialRole::InsideWhiteboard, false),
            Self::random(random_source, OfficialRole::JamTimer, false),
            Self::random(random_source, OfficialRole::Scorekeeper, false),
            Self::random(random_source, OfficialRole::Scorekeeper, false),
            Self::random(random_source, OfficialRole::ScoreboardOperator, false),
            Self::random(random_source, OfficialRole::PenaltyBoxManager, false),
            Self::random(random_source, OfficialRole::PenaltyBoxTimer, false),
            Self::random(random_source, OfficialRole::PenaltyBoxTimer, false),
            Self::random(random_source, OfficialRole::InsidePackReferee, true),
            Self::random(random_source, OfficialRole::InsidePackReferee, false),
            Self::random(random_source, OfficialRole::OutsidePackReferee, false),
            Self::random(random_source, OfficialRole::OutsidePackReferee, false),
            Self::random(random_source, OfficialRole::OutsidePackReferee, false),
            Self::random(random_source, OfficialRole::JammerReferee, false),
            Self::random(random_source, OfficialRole::JammerReferee, false),
        ]
    }

    fn get_random_name(random_source: &mut StdRng) -> String {
        let first_name = word_list::NAME_ADJECTIVES[random_source.gen_range(0..word_list::NAME_ADJECTIVES.len())];
        let last_name = word_list::NAME_NOUNS[random_source.gen_range(0..word_list::NAME_NOUNS.len())];

        first_name.to_owned() + " " + last_name
    }
}