use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Map};
use uuid::Uuid;

use crate::official::{Official, OfficialRole};

const PERIOD_DURATION: u64 = 30 * 60 * 1000;

pub struct TripJson {
    id: Uuid,
    pub after_star_pass: bool,
    pub start_tick: u64,
    pub duration: u64,
    pub score: u8,
}

#[derive(Clone)]
pub struct FieldingSkaterJson {
    pub skater_id: Uuid,
    pub number: String,
}

#[derive(Clone)]
pub struct TeamJamFielding {
    pub jammer: FieldingSkaterJson,
    pub pivot: FieldingSkaterJson,
    pub blocker1: FieldingSkaterJson,
    pub blocker2: FieldingSkaterJson,
    pub blocker3: FieldingSkaterJson,
}

pub struct TeamJamJson {
    fielding: TeamJamFielding,
    pub called_off: bool,
    pub is_lead: bool,
    trips: Vec<TripJson>,
}

impl TeamJamJson {
    pub fn current_trip_mut(&mut self) -> Option<&mut TripJson> {
        self.trips.last_mut()
    }

    pub fn trip_count(&self) -> usize {
        self.trips.len()
    }

    pub fn add_trip(&mut self, start_tick: u64) {
        self.trips.push(TripJson { 
            id: Uuid::new_v4(),
            after_star_pass: false,
            start_tick: start_tick,
            duration: 0,
            score: 0,
        });
    }
}

pub struct JamJson {
    id: Uuid,
    pub start_tick: u64,
    pub end_tick: u64,
    pub home_team_jam: TeamJamJson,
    pub away_team_jam: TeamJamJson,
}

pub struct PeriodJson {
    id: Uuid,
    pub start_tick: u64,
    pub duration: u64,
    jams: Vec<JamJson>,
}

impl PeriodJson {
    pub fn current_jam_mut(&mut self) -> Option<&mut JamJson> {
        self.jams.last_mut()
    }

    pub fn jam_count(&self) -> usize {
        self.jams.len()
    }

    pub fn add_jam(&mut self, start_tick: u64, home_team_fielding: &TeamJamFielding, away_team_fielding: &TeamJamFielding) {
        self.jams.push(JamJson { 
            id: Uuid::new_v4(),
            start_tick: start_tick,
            end_tick: 0,
            home_team_jam: TeamJamJson { fielding: home_team_fielding.clone(), called_off: false, is_lead: false, trips: Vec::new() },
            away_team_jam: TeamJamJson { fielding: away_team_fielding.clone(), called_off: false, is_lead: false, trips: Vec::new() },
        });
    }
}


pub struct GameJson {
    id: Uuid,
    officials_crew: Vec<Official>,
    periods: Vec<PeriodJson>,
}

impl GameJson {
    pub fn new() -> GameJson {
        GameJson {
            id: Uuid::new_v4(),
            officials_crew: Vec::default(),
            periods: Vec::default(),
        }
    }

    pub fn current_period_mut(&mut self) -> Option<&mut PeriodJson> {
        self.periods.last_mut()
    }

    pub fn add_period(&mut self, start_tick: u64) {
        self.periods.push(PeriodJson { 
            id: Uuid::new_v4(),
            start_tick,
            duration: 0,
            jams: Vec::new(),
        });
    }

    pub fn add_official(&mut self, official: &Official) {
        self.officials_crew.push(official.clone());
    }

    pub fn to_string(&self) -> String {
        let mut output = OutputJson { state: Map::new() };

        let key_prefix = format!("ScoreBoard.Game({})", Uuid::new_v4().as_hyphenated().to_string());
        let key = |k: &str| format!("{}.{}", key_prefix, k);

        output.state.insert(key("AbortReason"), json!(""));
        self.output_clocks(&key_prefix, &mut output);
        output.state.insert(key("ClockDuringFinalScore"), json!(false));
        output.state.insert(key("CurrentPeriod"), json!(self.periods.iter().last().unwrap().id.as_hyphenated().to_string()));
        output.state.insert(key("CurrentPeriodNumber"), json!(self.periods.len()));
        output.state.insert(key("CurrentTimeout"), json!("noTimeout"));
        output.state.insert(key("EventInfo(City)"), json!("Testville"));
        let datetime: DateTime<Utc> = std::time::SystemTime::now().into();
        output.state.insert(key("EventInfo(Date)"), json!(datetime.format("%Y-%m-%d").to_string()));
        output.state.insert(key("EventInfo(GameNo)"), json!("1"));
        output.state.insert(key("EventInfo(HostLeague)"), json!("Test Roller Derby"));
        output.state.insert(key("EventInfo(StartTime)"), json!("12pm"));
        output.state.insert(key("EventInfo(State)"), json!("Testshire"));
        output.state.insert(key("EventInfo(Tournament)"), json!(""));
        output.state.insert(key("EventInfo(Venue)"), json!("Example Sports Center"));
        output.state.insert(key("ExportBlockedBy"), json!(""));
        output.state.insert(key("Filename"), json!("STATS-Test"));
        output.state.insert(key("HNSO"), json!(self.officials_crew.iter().filter(|o| o.is_head && o.role != OfficialRole::InsidePackReferee).next().unwrap().name));
        output.state.insert(key("HR"), json!(self.officials_crew.iter().filter(|o| o.is_head && o.role == OfficialRole::InsidePackReferee).next().unwrap().name));
        output.state.insert(key("Id"), json!(self.id.as_hyphenated().to_string()));
        output.state.insert(key("InJam"), json!(false));
        output.state.insert(key("InOvertime"), json!(false));
        output.state.insert(key("InPeriod"), json!(false));
        output.state.insert(key("InSuddenScoring"), json!(false));
        output.state.insert(key("InjuryContinuationUpcoming"), json!(false));
        output.state.insert(key("JsonExists"), json!(true));
        output.state.insert(key("Label(Replaced)"), json!("---"));
        output.state.insert(key("Label(Start)"), json!("Start Jam"));
        output.state.insert(key("Label(Stop)"), json!("Lineup"));
        output.state.insert(key("Label(Timeout)"), json!("Timeout"));
        output.state.insert(key("Label(Undo)"), json!("---"));
        output.state.insert(key("LastFileUpdate"), json!("Never"));
        output.state.insert(key("Name"), json!("Test"));
        output.state.insert(key("NameFormat"), json!("Test"));
        output.state.insert(key("NoMoreJam"), json!(false));
        output.state.insert(key("OfficialReview"), json!(false));
        output.state.insert(key("OfficialScore"), json!(true));
        output.state.insert(key("PenaltyCode(?)"), json!("Unknown"));
        output.state.insert(key("PenaltyCode(A)"), json!("High Block"));
        output.state.insert(key("PenaltyCode(B)"), json!("Back Block"));
        output.state.insert(key("PenaltyCode(C)"), json!("Illegal Contact,Illegal Assist,OOP Block,Early/Late Hit"));
        output.state.insert(key("PenaltyCode(D)"), json!("Direction,Stop Block"));
        output.state.insert(key("PenaltyCode(E)"), json!("Leg Block"));
        output.state.insert(key("PenaltyCode(F)"), json!("Forearm"));
        output.state.insert(key("PenaltyCode(G)"), json!("Misconduct,Insubordination"));
        output.state.insert(key("PenaltyCode(H)"), json!("Head Block"));
        output.state.insert(key("PenaltyCode(I)"), json!("Illegal Procedure,Star Pass Violation,Pass Interference"));
        output.state.insert(key("PenaltyCode(L)"), json!("Low Block"));
        output.state.insert(key("PenaltyCode(M)"), json!("Multiplayer"));
        output.state.insert(key("PenaltyCode(N)"), json!("Interference,Delay Of Game"));
        output.state.insert(key("PenaltyCode(P)"), json!("Illegal Position,Destruction,Skating OOB,Failure to..."));
        output.state.insert(key("PenaltyCode(X)"), json!("Cut,Illegal Re-Entry"));
        output.state.insert(key("Readonly"), json!(false));
        output.state.insert(key("State"), json!("Finished"));
        output.state.insert(key("StatsbookExists"), json!(false));
        output.state.insert(key("SuspensionsServed"), json!(""));

        output.state.insert("ScoreBoard.Version(release)".to_string(), json!("v2023.3"));

        let first_jam_id = Uuid::new_v4();
        let upcoming_jam_id = Uuid::new_v4();

        let mut period_number = 0;
        for period in self.periods.iter() {
            period_number += 1;

            let period_key_prefix = key(format!("Period({})", period_number).as_str());
            let key = |k: &str| format!("{}.{}", period_key_prefix, k);

            output.state.insert(key("CurrentJam"), json!(period.jams.last().unwrap().id.as_hyphenated().to_string()));
            output.state.insert(key("CurrentJamNumber"), json!(period.jams.len()));
            output.state.insert(key("Duration"), json!(period.duration));
            output.state.insert(key("FirstJam"), json!(period.jams[0].id.as_hyphenated().to_string()));
            output.state.insert(key("FirstJamNumber"), json!(1));
            output.state.insert(key("id"), json!(period.id.as_hyphenated().to_string()));

            let mut jam_number = 0;
            for jam in period.jams.iter() {
                jam_number += 1;

                let next_jam_id = if jam_number < period.jams.len() {
                    period.jams[jam_number].id
                } else {
                    if period_number < self.periods.len() {
                        self.periods[period_number].jams[0].id
                    } else {
                        upcoming_jam_id
                    }
                };

                let previous_jam_id = if jam_number > 1 {
                    period.jams[jam_number - 2].id
                } else {
                    if period_number > 1 {
                        self.periods[period_number - 2].jams.last().unwrap().id
                    } else {
                        first_jam_id
                    }
                };

                let jam_key_prefix = key(format!("Jam({})", jam_number).as_str());
                let key = |k: &str| format!("{}.{}", jam_key_prefix, k);

                output.state.insert(key("Duration"), json!(jam.end_tick - jam.start_tick));
                output.state.insert(key("Id"), json!(jam.id.as_hyphenated().to_string()));
                output.state.insert(key("InjuryContinuation"), json!(false));
                output.state.insert(key("Next"), json!(next_jam_id.as_hyphenated().to_string()));
                output.state.insert(key("Number"), json!(jam_number));
                output.state.insert(key("Overtime"), json!(false));
                output.state.insert(key("PeriodClockDisplayEnd"), json!(PERIOD_DURATION - (jam.end_tick - period.start_tick)));
                output.state.insert(key("PeriodClockElapsedEnd"), json!(jam.end_tick - period.start_tick));
                output.state.insert(key("PeriodClockElapsedStart"), json!(jam.start_tick - period.start_tick));
                output.state.insert(key("PeriodNumber"), json!(period_number));
                output.state.insert(key("Previous"), json!(previous_jam_id.as_hyphenated().to_string()));
                output.state.insert(key("Readonly"), json!(false));
                output.state.insert(key("StarPass"), json!(false));

                self.output_team(&jam, jam_number, &jam.home_team_jam, &format!("{}.TeamJam(1)", jam_key_prefix), next_jam_id, previous_jam_id, &mut output);
                self.output_team(&jam, jam_number, &jam.away_team_jam, &format!("{}.TeamJam(2)", jam_key_prefix), next_jam_id, previous_jam_id, &mut output);
            }
        }
        serde_json::to_string_pretty(&output).unwrap()
    }

    fn output_clocks(&self, key_prefix: &String, output: &mut OutputJson) {
        let mut output_clock = |name: &str, direction: bool, max_time: u64| {
            let key = |k: &str| format!("{}.Clock({}).{}", key_prefix, name, k);
            output.state.insert(key("Direction"), json!(direction));
            output.state.insert(key("Id"), json!(Uuid::new_v4().as_hyphenated().to_string()));
            output.state.insert(key("InvertedTime"), json!(max_time));
            output.state.insert(key("MaximumTime"), json!(max_time));
            output.state.insert(key("Name"), json!(name));
            output.state.insert(key("Number"), json!(0));
            output.state.insert(key("Readonly"), json!(true));
            output.state.insert(key("Running"), json!(false));
            output.state.insert(key("Time"), json!(0));
        };

        output_clock("Intermission", true, 3600000);
        output_clock("Jam", true, 120000);
        output_clock("Lineup", false, 86400000);
        output_clock("Period", true, 1200000);
        output_clock("Timeout", false, 86400000);
    }

    fn output_team(&self, jam: &JamJson, jam_number: usize, team_jam: &TeamJamJson, jam_key_prefix: &String, next_jam_id: Uuid, previous_jam_id: Uuid, output: &mut OutputJson) {
        let key = |k: &str| format!("{}.{}", jam_key_prefix, k);

        output.state.insert(key("AfterSPScore"), json!(0));
        output.state.insert(key("Calloff"), json!(team_jam.called_off));
        output.state.insert(key("CurrentTrip"), json!(team_jam.trips.last().unwrap().id.as_hyphenated().to_string()));
        output.state.insert(key("CurrentTripNumber"), json!(team_jam.trips.len()));
        output.state.insert(key("DisplayLead"), json!(team_jam.is_lead));

        self.output_team_jam_roster(jam, jam_number, team_jam, jam_key_prefix, next_jam_id, previous_jam_id, output);
        self.output_team_trips(jam, team_jam, jam_key_prefix, output);
    }

    fn output_team_jam_roster(&self, jam: &JamJson, jam_number: usize, team_jam: &TeamJamJson, key_prefix: &String, next_jam_id: Uuid, previous_jam_id: Uuid, output: &mut OutputJson) {
        self.output_skater(jam, jam_number, &team_jam.fielding.blocker1, &format!("{}.Fielding(Blocker1)", key_prefix), "blocker1", next_jam_id, previous_jam_id, output);
        self.output_skater(jam, jam_number, &team_jam.fielding.blocker2, &format!("{}.Fielding(Blocker2)", key_prefix), "blocker2", next_jam_id, previous_jam_id, output);
        self.output_skater(jam, jam_number, &team_jam.fielding.blocker3, &format!("{}.Fielding(Blocker3)", key_prefix), "blocker3", next_jam_id, previous_jam_id, output);
        self.output_skater(jam, jam_number, &team_jam.fielding.jammer, &format!("{}.Fielding(Jammer)", key_prefix), "jammer", next_jam_id, previous_jam_id, output);
        self.output_skater(jam, jam_number, &team_jam.fielding.pivot, &format!("{}.Fielding(Pivot)", key_prefix), "pivot", next_jam_id, previous_jam_id, output);
    }

    fn output_skater(&self, jam: &JamJson, jam_number: usize, skater: &FieldingSkaterJson, key_prefix: &String, position_name: &str, next_jam_id: Uuid, previous_jam_id: Uuid, output: &mut OutputJson) {
        let key = |k: &str| format!("{}.{}", key_prefix, k);

        output.state.insert(key("Annotation"), json!(""));
        output.state.insert(key("BoxTripSymbols"), json!(""));
        output.state.insert(key("BoxTripSymbolsAfterSP"), json!(""));
        output.state.insert(key("BoxTripSymbolsBeforeSP"), json!(""));
        output.state.insert(key("CurrentBoxTrip"), json!(""));
        output.state.insert(key("Id"), json!(format!("{}_1_{}", jam.id.as_hyphenated().to_string(), position_name)));
        output.state.insert(key("Next"), json!(format!("{}_1_{}", next_jam_id.as_hyphenated().to_string(), position_name)));
        output.state.insert(key("NotFielded"), json!(false));
        output.state.insert(key("Number"), json!(jam_number));
        output.state.insert(key("PenaltyBox"), json!(false));
        output.state.insert(key("Position"), json!(format!("00000000-0000-0000-0000-000000000000_1_{}", position_name)));
        output.state.insert(key("Next"), json!(format!("{}_1_{}", previous_jam_id.as_hyphenated().to_string(), position_name)));
        output.state.insert(key("Readonly"), json!(false));
        output.state.insert(key("SitFor3"), json!(false));
        output.state.insert(key("Skater"), json!(skater.skater_id.as_hyphenated().to_string()));
        output.state.insert(key("SkaterNumber"), json!(skater.number));
    }

    fn output_team_trips(&self, jam: &JamJson, team_jam: &TeamJamJson, jam_key_prefix: &String, output: &mut OutputJson) {
        let mut trip_number = 0;
        for trip in team_jam.trips.iter() {
            trip_number += 1;

            let trip_key_prefix = format!("{}.ScoringTrip({})", jam_key_prefix, trip_number);

            let key = |k: &str| format!("{}.{}", trip_key_prefix, k);

            output.state.insert(key("Current"), json!(false));
            output.state.insert(key("Duration"), json!(trip.duration));
            output.state.insert(key("Id"), json!(trip.id.as_hyphenated().to_string()));
            output.state.insert(key("JamClockStart"), json!(trip.start_tick - jam.start_tick));
            output.state.insert(key("JamClockEnd"), json!(trip.start_tick + trip.duration - jam.start_tick));
            output.state.insert(key("Number"), json!(trip_number));
            output.state.insert(key("Readonly"), json!(false));
            output.state.insert(key("Score"), json!(trip.score));
        }
    }
}

#[derive(Serialize)]
struct OutputJson {
    state: Map<String, serde_json::Value>,
}