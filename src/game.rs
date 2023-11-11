use std::{cmp::Ordering, borrow::BorrowMut};

use rand::{Rng, rngs::StdRng};

use crate::{team::Team, official::Official, skater::{Skater, Position}, game_json::{GameJson, TeamJamFielding, FieldingSkaterJson, TeamJamJson}};

const PENALTY_SIT_DURATION: u64 = 30 * 1000;
const PERIOD_DURATION: u64 = 30 * 60 * 1000;
const JAM_DURATION: u64 = 2 * 60 * 1000;
const LINEUP_DURATION: u64 = 30 * 1000;

const RETURN_CUT_PENALTY_CHANCE: f64 = 1.0 / 100.0;
const EXIT_PACK_NO_PASS_CHANCE: f64 = 1.0 / 50.0;
const EXIT_PACK_CALL_CHANCE: f64 = 1.0 / 2.0;

type OnTrackTeam = Vec<JamSkater>;

#[derive(Clone, Debug)]
struct PreGame {
}

#[derive(Clone, Debug)]
enum LeadJammerTeam {
    None,
    Home,
    Away,
}

#[derive(Clone, Debug)]
struct JamInProgress {
    start_tick: u64,
    home_skaters: OnTrackTeam,
    away_skaters: OnTrackTeam,
    lead_jammer_team: LeadJammerTeam,
}

#[derive(Clone, Debug)]
struct LineupInProgress {
    start_tick: u64,
}

#[derive(Clone, Debug)]
enum TimeoutType {
    Official,
    Team,
    Review,
}

#[derive(Clone, Debug)]
struct TimeoutInProgress {
    start_tick: u64,
    timeout_type: TimeoutType,
}

#[derive(Clone, Debug)]
struct IntervalInProgress {
    start_tick: u64,
}

#[derive(Clone, Debug)]
struct PostGame {
    start_tick: u64,
}

#[derive(Clone, Debug)]
enum GameState {
    PreGame(PreGame),
    JamInProgress(JamInProgress),
    LineupInProgress(LineupInProgress),
    TimeoutInProgress(TimeoutInProgress),
    IntervalInProgress(IntervalInProgress),
    PostGame(PostGame),
}

type PenaltyBox = Vec<JamSkater>;

#[derive(Clone)]
pub struct GameTeam {
    pub details: Team,
    timeouts_remaining: u8,
    has_official_review: bool,
    official_review_retained: bool,
    roster: Vec<GameSkater>,
}

#[derive(Clone, Debug)]
struct GameSkater {
    details: Skater,
    penalties: Vec<Penalty>,
    last_jam_tick: u64,
}

#[derive(Clone, Debug)]
struct Penalty {
    code: String,
    received_tick: u64,
}

#[derive(Clone, Debug)]
struct SkatingOnTrack {
    location: f32,
}

#[derive(Clone, Debug)]
struct SkatingToBox {
    distance_remaining: f32,
    penalties_to_sit: u8,
}

#[derive(Clone, Debug)]
struct SatInBox {
    start_tick: u64,
    penalty_count: u8,
}

#[derive(Clone, Debug)]
struct ReturningFromBox {
    distance_remaining: f32,
}

#[derive(Clone, Debug)]
struct HeldInBox {
    ticks_expired: u64,
    penalty_count: u8,
}

#[derive(Clone, Debug)]
enum SkaterActivity {
    SkatingOnTrack(SkatingOnTrack),
    SkatingToBox(SkatingToBox),
    SatInBox(SatInBox),
    ReturningFromBox(ReturningFromBox),
    HeldInBox(HeldInBox),
}

#[derive(Clone, Debug)]
struct JamSkater {
    details: Skater,
    position: Position,
    activity: SkaterActivity,
    can_receive_lead: bool,
    is_lead: bool,
}

impl From<Vec<JamSkater>> for TeamJamFielding {
    fn from(value: Vec<JamSkater>) -> Self {
        let blockers: Vec<&JamSkater> = value.iter().filter(|s| s.position == Position::Blocker).collect();
        let jammer = value.iter().filter(|s| s.position == Position::Jammer).next().unwrap();
        let pivot = value.iter().filter(|s| s.position == Position::Pivot).next();

        TeamJamFielding {
            blocker1: blockers[0].clone().into(),
            blocker2: blockers[1].clone().into(),
            blocker3: blockers[2].clone().into(),
            jammer: jammer.clone().into(),
            pivot: if let Some(p) = pivot { p.clone().into() } else { blockers[3].clone().into() },
        }
    }
}

impl From<JamSkater> for FieldingSkaterJson {
    fn from(value: JamSkater) -> Self {
        FieldingSkaterJson {
            skater_id: value.details.id,
            number: "".to_string(),
        }
    }
}

struct JamStartEvent {
    tick: u64,
}

pub struct Game {
    random_source: StdRng,
    pub home_team: GameTeam,
    pub away_team: GameTeam,
    pub officials: Vec<Official>,
    pub game_json: GameJson,
    state: GameState,
    current_tick: u64,
    period_clock: u64,
    penalty_box: PenaltyBox,
    lead_is_open: bool,
    jam_called: bool,
}

impl Game {
    pub fn random(mut random_source: StdRng) -> Game {
        let home_team = Self::get_random_team(&mut random_source);
        let away_team = Self::get_random_team(&mut random_source);
        let officials = Official::random_crew(&mut random_source);

        let mut game = Game {
            random_source,
            home_team,
            away_team,
            officials,
            game_json: GameJson::new(),
            state: GameState::PreGame(PreGame {}),
            current_tick: 0,
            period_clock: 0,
            penalty_box: PenaltyBox::default(),
            lead_is_open: false,
            jam_called: false,
        };

        for official in game.officials.iter() {
            game.game_json.add_official(official);
        }

        game
    }

    pub fn run(&mut self) {
        loop {
            if let GameState::PostGame(_) = self.state {
                break;
            }

            self.tick();
        }
    }

    fn get_random_team(random_source: &mut StdRng) -> GameTeam {
        let team = Team::random(random_source);

        GameTeam {
            details: team.clone(),
            timeouts_remaining: 3,
            has_official_review: true,
            official_review_retained: false,
            roster: team.clone().roster.iter().map(|s| GameSkater {
                details: s.clone(),
                penalties: vec![],
                last_jam_tick: 0,
            }).collect(),
        }
    }

    fn tick(&mut self) {
        self.current_tick += 1000;
        self.state = match self.state.clone() {
            GameState::PreGame(_) => self.start_jam(),
            GameState::JamInProgress(jam) => self.tick_jam(&jam),
            GameState::LineupInProgress(lineup) => self.tick_lineup(&lineup),
            _ => self.end_game()
        };
    }

    fn start_jam(&mut self) -> GameState {
        let jam_start_tick = self.get_random_current_tick();

        if self.period_clock == 0 {
            self.game_json.add_period(jam_start_tick);
            self.period_clock = 30 * 60 * 1000 + jam_start_tick;
        }

        let home_skaters = self.get_random_jam_team(&self.home_team.clone());
        let away_skaters = self.get_random_jam_team(&self.away_team.clone());

        let jam = JamInProgress { 
            start_tick: jam_start_tick,
            home_skaters,
            away_skaters,
            lead_jammer_team: LeadJammerTeam::None,
        };

        self.game_json.current_period_mut().unwrap().add_jam(jam_start_tick, &jam.home_skaters.clone().into(), &jam.away_skaters.clone().into());
        self.game_json.current_period_mut().unwrap().current_jam_mut().unwrap().home_team_jam.add_trip(jam_start_tick);
        self.game_json.current_period_mut().unwrap().current_jam_mut().unwrap().away_team_jam.add_trip(jam_start_tick);

        println!("Jam {} started", self.game_json.current_period_mut().unwrap().jam_count());

        self.lead_is_open = true;
        self.jam_called = false;

        GameState::JamInProgress(jam)
    }

    fn end_game(&self) -> GameState {
        GameState::PostGame(PostGame { start_tick: self.current_tick })
    }

    fn end_jam(&mut self, jam: &JamInProgress, jam_end_tick: u64, was_called: bool) -> GameState {
        let jam_json = self.game_json.current_period_mut().unwrap().current_jam_mut().unwrap();
        jam_json.end_tick = jam_end_tick;

        let mut end_team_jam = |team_jam: &mut TeamJamJson| {
            let trip_count = team_jam.trip_count();
            let trip = team_jam.current_trip_mut().unwrap();
            println!("{} - {}", jam_end_tick, trip.start_tick);
            trip.duration = if trip.start_tick < jam_end_tick { jam_end_tick - trip.start_tick } else { 0 };
            trip.score = if trip_count > 1 { self.random_source.gen_range(0..=4) } else { 0 };
        };

        end_team_jam(&mut jam_json.home_team_jam);

        let period_has_expired = self.period_clock == 0;

        for skater in self.penalty_box.iter_mut() {
            let track_skater = jam.home_skaters.iter().chain(jam.away_skaters.iter()).filter(|s| s.details.id == skater.details.id).next().unwrap();
            skater.activity = match &track_skater.activity {
                SkaterActivity::SkatingToBox(skating_to_box) => {
                    SkaterActivity::HeldInBox(HeldInBox { 
                        ticks_expired: 0,
                        penalty_count: skating_to_box.penalties_to_sit,
                    })
                },
                SkaterActivity::SatInBox(sat_in_box) => {
                    SkaterActivity::HeldInBox(HeldInBox { 
                        ticks_expired: jam_end_tick - sat_in_box.start_tick,
                        penalty_count: sat_in_box.penalty_count,
                    })
                },
                _ => {
                    track_skater.activity.clone()
                }
            }
        }

        if period_has_expired {
            let period = self.game_json.current_period_mut().unwrap();
            period.duration = jam_end_tick - period.start_tick;

            GameState::IntervalInProgress(IntervalInProgress { start_tick: jam_end_tick })
        } else {
            GameState::LineupInProgress(LineupInProgress { start_tick: jam_end_tick })
        }
    }

    fn tick_jam(&mut self, jam: &JamInProgress) -> GameState {
        let jam_has_expired = self.current_tick - jam.start_tick >= 120000;

        self.period_clock = if self.period_clock >= 1000 { self.period_clock - 1000 } else { 0 };

        if jam_has_expired {
            let jam_end_tick = jam.start_tick + 120000;

            println!("Jam expired");
            self.end_jam(jam, jam_end_tick, false)
        } else {
            let mut home_skaters = jam.home_skaters.clone();
            for skater in home_skaters.iter_mut() {
                self.tick_skater(skater, true);
            }

            let mut away_skaters = jam.away_skaters.clone();
            for skater in away_skaters.iter_mut() {
                self.tick_skater(skater, false);
            }

            if self.jam_called {
                let jam_end_tick = self.get_random_current_tick();

                println!("Jam called");
                self.end_jam(jam, jam_end_tick, true)
            } else {
                GameState::JamInProgress(JamInProgress {
                    start_tick: jam.start_tick,
                    home_skaters: home_skaters,
                    away_skaters: away_skaters.clone(),
                    lead_jammer_team: jam.lead_jammer_team.clone(),
                })
            }
        }
    }

    fn give_skater_penalty(&mut self, skater: &mut JamSkater) -> SkaterActivity {
        println!("Penalty for {}", skater.details.name);
        skater.is_lead = false;
        skater.can_receive_lead = false;
        self.penalty_box.push(skater.clone());

        SkaterActivity::SkatingToBox(SkatingToBox { 
            distance_remaining: self.random_source.gen_range(1.0..60.0),
            penalties_to_sit: 1
        })
    }

    fn tick_on_track_skater(&mut self, on_track: &SkatingOnTrack, skater: &mut JamSkater, is_home_team: bool) -> SkaterActivity {
        let has_commited_penalty = self.random_source.gen_bool(skater.details.penalty_chance);

        let set_is_lead = |game_json: &mut GameJson, is_lead: bool| {
            let team_json = if is_home_team { 
                game_json.current_period_mut().unwrap().current_jam_mut().unwrap().home_team_jam.borrow_mut()
            } else {
                game_json.current_period_mut().unwrap().current_jam_mut().unwrap().away_team_jam.borrow_mut()
            };
            team_json.is_lead = is_lead;
        };

        if has_commited_penalty {
            set_is_lead(&mut self.game_json, false);
            self.give_skater_penalty(skater)
        } else {
            if skater.position == Position::Jammer {
                let is_in_pack = on_track.location < 20.0;

                if is_in_pack {
                    let new_location = on_track.location + self.random_source.gen_range(-2.0..skater.details.base_speed / 4.0);

                    let has_exited_pack = new_location >= 20.0;

                    if has_exited_pack {
                        let pass_completion_tick = self.get_random_current_tick();

                        let team = if is_home_team {
                            self.game_json.current_period_mut().unwrap().current_jam_mut().unwrap().home_team_jam.borrow_mut()
                        } else {
                            self.game_json.current_period_mut().unwrap().current_jam_mut().unwrap().away_team_jam.borrow_mut()
                        };

                        if let Some(trip) = team.current_trip_mut() {
                            trip.score = 4;
                            trip.duration = pass_completion_tick - trip.start_tick;
                        }

                        if skater.is_lead {
                            self.jam_called = self.random_source.gen_bool(EXIT_PACK_CALL_CHANCE);
                        }

                        let could_receive_lead = self.lead_is_open && skater.can_receive_lead;
                        if could_receive_lead {
                            let lead_earned = !self.random_source.gen_bool(EXIT_PACK_NO_PASS_CHANCE);
                            if lead_earned {
                                set_is_lead(&mut self.game_json, true);

                                self.lead_is_open = false;
                                skater.is_lead = true;
                            }
                            skater.can_receive_lead = false;
                        }
                    }

                    let has_received_penalty = self.random_source.gen_bool(skater.details.penalty_chance);
                    if has_received_penalty {
                        self.give_skater_penalty(skater)
                    } else {
                        SkaterActivity::SkatingOnTrack(SkatingOnTrack {
                            location: new_location,
                        })
                    }
                } else {
                    let mut new_location = on_track.location + skater.details.base_speed;
                    if new_location > 100.0 {
                        new_location = 0.0;

                        let pass_start_tick = self.get_random_current_tick();
                        let team = if is_home_team {
                            self.game_json.current_period_mut().unwrap().current_jam_mut().unwrap().home_team_jam.borrow_mut()
                        } else {
                            self.game_json.current_period_mut().unwrap().current_jam_mut().unwrap().away_team_jam.borrow_mut()
                        };

                        team.add_trip(pass_start_tick);
                    }
                    SkaterActivity::SkatingOnTrack(SkatingOnTrack {
                        location: new_location,
                    })
                }
            } else {
                let has_received_penalty = self.random_source.gen_bool(skater.details.penalty_chance);
                if has_received_penalty {
                    self.give_skater_penalty(skater)
                } else {
                    skater.activity.clone()
                }
            }
        }
    }

    fn tick_skating_to_box_skater(&mut self, to_box: &SkatingToBox, skater: &mut JamSkater) -> SkaterActivity {
        let distance_covered = skater.details.base_speed + self.random_source.gen_range(-1.0..1.0);
        if to_box.distance_remaining > distance_covered {
            let should_get_second_penalty = to_box.penalties_to_sit == 1 && self.random_source.gen_bool(1.0 / 20.0);

            SkaterActivity::SkatingToBox(SkatingToBox {
                distance_remaining: to_box.distance_remaining - distance_covered,
                penalties_to_sit: if should_get_second_penalty { 2 } else { 1 },
            })
        } else {
            SkaterActivity::SatInBox(SatInBox {
                start_tick: self.get_random_current_tick(),
                penalty_count: to_box.penalties_to_sit,
            })
        }
    }

    fn tick_sat_in_box_skater(&mut self, sat_in_box: &SatInBox, skater: &mut JamSkater) -> SkaterActivity {
        let has_completed_penalties = self.current_tick - sat_in_box.start_tick > PENALTY_SIT_DURATION * sat_in_box.penalty_count as u64;
                
        if has_completed_penalties {
            println!("Releasing {}", skater.details.name);
            self.penalty_box.retain(|s| s.details.id != skater.details.id);

            SkaterActivity::ReturningFromBox(ReturningFromBox { distance_remaining: self.random_source.gen_range(1.0..60.0) })
        } else {
            skater.activity.clone()
        }
    }

    fn tick_returning_from_box_skater(&mut self, returning: &ReturningFromBox, skater: &mut JamSkater) -> SkaterActivity {
        let distance_covered = skater.details.base_speed + self.random_source.gen_range(-1.0..1.0);
        if returning.distance_remaining > distance_covered {
            SkaterActivity::ReturningFromBox(ReturningFromBox { distance_remaining: returning.distance_remaining - distance_covered })
        } else {
            let should_get_cut_penalty = self.random_source.gen_bool(RETURN_CUT_PENALTY_CHANCE);

            if should_get_cut_penalty {
                self.give_skater_penalty(skater)
            } else {
                SkaterActivity::SkatingOnTrack(SkatingOnTrack { 
                    location: 0.0
                })
            }
        }
    }

    fn tick_held_in_box_skater(&mut self, held_in_box: &HeldInBox) -> SkaterActivity {
        SkaterActivity::SatInBox(SatInBox { 
            start_tick: self.current_tick - held_in_box.ticks_expired,
            penalty_count: held_in_box.penalty_count,
        })
    }

    fn tick_skater(&mut self, skater: &mut JamSkater, is_home_team: bool) {
        skater.activity = match &skater.activity.clone() {
            SkaterActivity::SkatingOnTrack(on_track) => self.tick_on_track_skater(on_track, skater, is_home_team),
            SkaterActivity::SkatingToBox(to_box) => self.tick_skating_to_box_skater(to_box, skater),
            SkaterActivity::SatInBox(sat_in_box) => self.tick_sat_in_box_skater(sat_in_box, skater),
            SkaterActivity::ReturningFromBox(returning) => self.tick_returning_from_box_skater(returning, skater),
            SkaterActivity::HeldInBox(held_in_box) => self.tick_held_in_box_skater(held_in_box),
        };
    }

    fn tick_lineup(&mut self, lineup: &LineupInProgress) -> GameState {
        self.period_clock = if self.period_clock >= 1000 { self.period_clock - 1000 } else { 0 };
        let period_has_expired = self.period_clock == 0;

        if period_has_expired {
            GameState::IntervalInProgress(IntervalInProgress { start_tick: self.get_random_current_tick() })
        } else {
            let should_start_new_jam = self.current_tick - lineup.start_tick >= 30000;

            if should_start_new_jam {
                self.start_jam()
            } else {
                GameState::LineupInProgress(LineupInProgress { start_tick: lineup.start_tick })
            }
        }
    }

    fn get_random_current_tick(&mut self) -> u64 {
        self.current_tick - self.random_source.gen_range(0..1000)
    }

    fn get_random_jam_team(&mut self, team: &GameTeam) -> OnTrackTeam {
        let mut on_track_skaters: Vec<JamSkater> =
            self.penalty_box.clone().into_iter()
                .filter(|s| team.roster.iter().any(|r| r.details.id == s.details.id))
                .map(|s| 
                    if s.position == Position::Jammer {
                        JamSkater {
                            can_receive_lead: true,
                            is_lead: false,
                            activity: s.activity,
                            details: s.details,
                            position: s.position,
                        }
                    } else {
                        s
                    })
                .collect();

        let mut available_skaters: Vec<&GameSkater> =
            team.roster.iter()
                .filter(|s| !on_track_skaters.iter().any(|r| r.details.id == s.details.id))
                .collect();

        available_skaters.sort_by(|a, b| a.last_jam_tick.cmp(&b.last_jam_tick));

        if !on_track_skaters.iter().any(|s| s.position == Position::Jammer) {
            let mut available_jammers = available_skaters.clone();
            available_jammers.sort_by(Self::compare_preferences(&Self::get_position_jammer_value));

            let index = self.random_source.gen_range(0..3);
            let jammer = available_jammers[index];
            on_track_skaters.push(JamSkater {
                details: jammer.details.clone(),
                position: Position::Jammer,
                activity: SkaterActivity::SkatingOnTrack(SkatingOnTrack { location: 95.0 }),
                can_receive_lead: true,
                is_lead: false,
            });
            available_skaters.retain(|s| s.details.id != jammer.details.id);
        }

        if !on_track_skaters.iter().any(|s| s.position == Position::Pivot) {
            let mut available_pivots = available_skaters.clone();
            available_pivots.sort_by(Self::compare_preferences(&Self::get_position_pivot_value));

            let index = self.random_source.gen_range(0..3);
            let pivot = available_pivots[index];
            on_track_skaters.push(JamSkater {
                details: pivot.details.clone(),
                position: Position::Pivot,
                activity: SkaterActivity::SkatingOnTrack(SkatingOnTrack { location: 0.0 }),
                can_receive_lead: false,
                is_lead: false,
            });
            available_skaters.retain(|s| s.details.id != pivot.details.id);
        }

        let mut available_blockers = available_skaters.clone();
        available_blockers.sort_by(Self::compare_preferences(&Self::get_position_blocker_value));

        while on_track_skaters.len() < 5 && available_blockers.len() > 0 {
            let index = self.random_source.gen_range(0..3);
            let blocker = available_blockers[index];
            on_track_skaters.push(JamSkater {
                details: blocker.details.clone(),
                position: Position::Blocker,
                activity: SkaterActivity::SkatingOnTrack(SkatingOnTrack { location: 0.0 }),
                can_receive_lead: false,
                is_lead: false,
            });
            available_blockers.remove(index);
        }

        return on_track_skaters;
    }

    fn compare_preferences(comparison: &'static dyn Fn(Position) -> u8) -> impl Fn(&&GameSkater, &&GameSkater) -> Ordering {
        |a, b| {
            comparison(b.details.favored_position).cmp(&comparison(a.details.favored_position))
        }
    }

    fn get_position_jammer_value(preference: Position) -> u8 {
        match preference {
            Position::Jammer => 2,
            Position:: Blocker => 1,
            Position:: Pivot => 0,
        }
    }

    fn get_position_pivot_value(preference: Position) -> u8 {
        match preference {
            Position::Pivot => 2,
            Position::Blocker => 1,
            Position::Jammer => 0,
        }
    }

    fn get_position_blocker_value(preference: Position) -> u8 {
        match preference {
            Position::Blocker => 2,
            Position::Pivot => 1,
            Position::Jammer => 0,
        }
    }
}