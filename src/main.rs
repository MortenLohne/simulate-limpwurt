use std::{
    collections::HashMap,
    fmt,
    ops::RangeInclusive,
    time::{self, Duration},
};

mod costs;
mod data;
#[cfg(test)]
mod tests;

use rand::Rng;
use rayon::prelude::*;
use strum::Display;

#[derive(Display, PartialEq, Eq)]
#[allow(dead_code)]
enum WorldState {
    Limp2024,
    Limp2025,
    Limp2026,
}

const WORLD_STATE: WorldState = WorldState::Limp2026;

fn main() {
    let start = match WORLD_STATE {
        WorldState::Limp2024 => SimulationStartPoint {
            slayer_level: 55,
            quests_done: vec![Quest::PorcineOfInterest],
            task_streak: 0,
            points: 0,
            task_state: TaskState::Active((Monster::Hellhounds, SlayerMaster::Vannaka, 40)),
        },
        WorldState::Limp2025 => SimulationStartPoint {
            slayer_level: 75,
            quests_done: vec![Quest::LostCity, Quest::PorcineOfInterest],
            task_streak: 1,
            points: 120,
            task_state: TaskState::Active((Monster::Monkeys, SlayerMaster::Turael, 20)),
        },
        WorldState::Limp2026 => SimulationStartPoint {
            slayer_level: 75,
            quests_done: vec![Quest::LostCity, Quest::PorcineOfInterest],
            task_streak: 1,
            points: 120,
            task_state: TaskState::Active((Monster::Monkeys, SlayerMaster::Turael, 20)),
        },
    };
    let start_time = time::Instant::now();
    let n = 1_000_000;

    let results: Vec<_> = (0..n)
        .into_par_iter()
        .map(|_| simulate_limpwurt(start.clone()))
        .collect();

    let mut num_successes = 0;
    let mut num_tasks_received: u64 = 0;
    let mut num_tasks_per_failed_run = vec![];
    let mut num_tasks_per_successful_run = vec![];
    let mut min_points_per_successful_run = vec![];
    let mut total_time_successful_runs = vec![];

    let mut max_points_locked = 0;

    for (slayer_data, success) in results {
        let num_tasks = slayer_data.total_tasks_started.values().sum::<u64>();
        num_tasks_received += num_tasks;
        if success {
            num_successes += 1;
            num_tasks_per_successful_run.push(num_tasks);
            min_points_per_successful_run.push(slayer_data.min_points);
            total_time_successful_runs.push(slayer_data.total_time);
        } else {
            max_points_locked = max_points_locked.max(slayer_data.max_points);
            num_tasks_per_failed_run.push(num_tasks);
        }
    }
    num_tasks_per_failed_run.sort();
    num_tasks_per_successful_run.sort();
    min_points_per_successful_run.sort();
    total_time_successful_runs.sort();

    let median_successful_tasks = num_tasks_per_successful_run
        .get(num_tasks_per_successful_run.len() / 2)
        .unwrap_or(&0);
    let median_failed_tasks = num_tasks_per_failed_run
        .get(num_tasks_per_failed_run.len() / 2)
        .unwrap_or(&0);
    let median_min_points = min_points_per_successful_run
        .get(min_points_per_successful_run.len() / 2)
        .unwrap_or(&0);
    let median_total_time = total_time_successful_runs
        .get(total_time_successful_runs.len() / 2)
        .unwrap_or(&Duration::ZERO);

    println!("Finished in {:.1}s", start_time.elapsed().as_secs_f32());
    println!(
        "Number of successes: {}, {:.3}%, {:.1} tasks received on average, {} tasks median on success, {} tasks median on failure",
        num_successes,
        100.0 * num_successes as f32 / n as f32,
        num_tasks_received as f32 / n as f32,
        median_successful_tasks,
        median_failed_tasks
    );
    println!(
        "Max points while eventually getting slayer-locked: {}, median min points on success: {}, min total time on succes: {:.1} hours, median total time on success: {:.1} hours, maximum total time on success: {:.1} hours",
        max_points_locked,
        median_min_points,
        total_time_successful_runs
            .first()
            .unwrap_or(&Duration::ZERO)
            .as_secs_f32()
            / 3600.0,
        median_total_time.as_secs_f32() / 3600.0,
        total_time_successful_runs
            .last()
            .unwrap_or(&Duration::ZERO)
            .as_secs_f32()
            / 3600.0,
    );
}

#[derive(Clone)]
struct SimulationStartPoint {
    slayer_level: u8,
    quests_done: Vec<Quest>,
    task_streak: u32,
    points: u32,
    task_state: TaskState,
}

/// Returns the number of tasks received, the minimum/maximum points reached, and whether he escaped (i.e. got lots of points)
fn simulate_limpwurt(start: SimulationStartPoint) -> (SlayerData, bool) {
    let limpwurt = PlayerState {
        slayer_level: start.slayer_level,
        quests_done: start.quests_done,
    };

    let mut slayer_state = SlayerState {
        task_streak: start.task_streak,
        points: start.points,
        task_state: start.task_state,
        slayer_data: SlayerData::default(),
    };

    let mut rng = rand::rng();

    loop {
        if slayer_state.points >= 1000 {
            return (slayer_state.slayer_data, true);
        }
        let TaskState::Active((monster, _, _)) = slayer_state.task_state else {
            panic!("Expected an active task");
        };

        // Simply complete the task if possible
        if monster.can_limpwurt_kill() {
            slayer_state.complete_assignment();
            // Do the 10th, 11th, 12th, 13th and 14th task at Vannaka
            let next_slayer_master =
                if slayer_state.task_streak >= 4 && (slayer_state.task_streak + 1) % 10 <= 4 {
                    SlayerMaster::Vannaka
                } else {
                    SlayerMaster::Spria
                };
            slayer_state.new_assignment(&mut rng, next_slayer_master, &limpwurt);
            continue;
        }

        let can_be_turael_skipped = data::TURAEL_ASSIGNMENTS
            .iter()
            .all(|assignment| assignment.monster != monster);
        // If Turael assigns the monster, we must point skip
        if !can_be_turael_skipped {
            if slayer_state.points >= 30 {
                slayer_state.point_skip();
                slayer_state.new_assignment(&mut rng, SlayerMaster::Spria, &limpwurt);
                continue;
            } else {
                return (slayer_state.slayer_data, false);
            }
        }
        // Otherwise, we Turael skip
        // TODO: This will count the time cost of getting to Turael, even if we're already there
        slayer_state.new_assignment(&mut rng, SlayerMaster::Turael, &limpwurt);
    }
}

#[derive(Display, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
enum SlayerMaster {
    Turael,
    Spria,
    Vannaka,
    Chaeldar,
}

impl SlayerMaster {
    pub fn assignments(&self) -> &[Assignment] {
        match self {
            SlayerMaster::Turael => data::TURAEL_ASSIGNMENTS,
            SlayerMaster::Spria => data::SPRIA_ASSIGNMENTS,
            SlayerMaster::Vannaka => data::VANNAKA_ASSIGNMENTS,
            SlayerMaster::Chaeldar => data::CHAELDAR_ASSIGNMENTS,
        }
    }

    pub fn slayer_points(&self) -> u32 {
        match self {
            SlayerMaster::Turael => 0,
            SlayerMaster::Spria => 0,
            SlayerMaster::Vannaka => match WORLD_STATE {
                WorldState::Limp2024 => 4,
                WorldState::Limp2025 => 4,
                WorldState::Limp2026 => 8,
            },
            SlayerMaster::Chaeldar => 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TaskState {
    Active((Monster, SlayerMaster, u32)), // (monster, master, amount)
    Completed(Monster),
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskState::Active((monster, master, _)) => {
                write!(f, "Active {} task: {}", master, monster)
            }
            TaskState::Completed(monster) => write!(f, "Completed task: {}", monster),
        }
    }
}

#[derive(Clone)]
struct SlayerData {
    total_points: u64,
    min_points: u64,
    max_points: u64,
    total_tasks_started: HashMap<(SlayerMaster, Monster), u64>,
    total_tasks_done: HashMap<(SlayerMaster, Monster), u64>,
    total_kills: HashMap<Monster, u64>,
    total_time: Duration,
    supplies_used: Supplies,
}

impl Default for SlayerData {
    fn default() -> Self {
        Self {
            min_points: u64::MAX,
            max_points: u64::MIN,
            total_points: 0,
            total_tasks_started: HashMap::new(),
            total_tasks_done: HashMap::new(),
            total_kills: HashMap::new(),
            total_time: Duration::default(),
            supplies_used: Supplies::default(),
        }
    }
}

#[derive(Default, Clone)]
struct Supplies {
    expeditious_bracelet_charges: u64,
    bracelet_of_slaughter_charges: u64,
    games_necklace_charges: u64,
    dueling_ring_charges: u64,
    necklace_of_passage_charges: u64,
    chronicle_charges: u64,
    skull_sceptre_charges: u64,
    law_runes: u64,
    attack_potion_doses: u64,
    strength_potion_doses: u64,
}

struct SlayerState {
    points: u32,
    task_streak: u32,
    task_state: TaskState,
    slayer_data: SlayerData,
}

impl SlayerState {
    pub fn new_assignment<R: Rng>(
        &mut self,
        rng: &mut R,
        master: SlayerMaster,
        player_state: &PlayerState,
    ) {
        match master {
            SlayerMaster::Turael => (),
            SlayerMaster::Spria => {
                assert!(player_state.quests_done.contains(&Quest::PorcineOfInterest))
            }
            SlayerMaster::Vannaka => (),
            SlayerMaster::Chaeldar => assert!(player_state.quests_done.contains(&Quest::LostCity)),
        }
        let last_task = match self.task_state {
            TaskState::Active((monster, _, _)) => {
                // If this is a Turael skip, reset the task counter
                self.task_streak = 0;
                if master != SlayerMaster::Turael {
                    panic!("Can only Turael-skip at Turael")
                }
                if data::TURAEL_ASSIGNMENTS.iter().any(|assignment| {
                    assignment.monster == monster && player_state.can_receive_assignment(assignment)
                }) {
                    panic!("Cannot Turael-skip a {} task", monster);
                }
                monster
            }
            TaskState::Completed(monster) => monster,
        };

        let possible_tasks: Vec<(u32, Assignment)> = master
            .assignments()
            .iter()
            .filter(|assignment| {
                player_state.can_receive_assignment(assignment) && assignment.monster != last_task
            })
            .fold(vec![], |mut acc, assignment| {
                acc.push((
                    acc.last().map(|(weight, _)| *weight).unwrap_or(0) + assignment.weight,
                    assignment.clone(),
                ));
                acc
            });

        let turael_tasks_weight_sum: u32 = possible_tasks.last().map_or(0, |(weight, _)| *weight);

        let task_num = rng.random_range(0..=turael_tasks_weight_sum);

        let task = possible_tasks
            .into_iter()
            .find(|(weight, _)| *weight >= task_num)
            .unwrap()
            .1;

        let amount = rng.random_range(task.amount);

        *self
            .slayer_data
            .total_tasks_started
            .entry((master, task.monster))
            .or_default() += 1;
        self.slayer_data.assignment_cost(master);

        self.task_state = TaskState::Active((task.monster, master, amount));
    }

    pub fn complete_assignment(&mut self) {
        let TaskState::Active((monster, master, amount)) = self.task_state else {
            panic!("Cannot complete assignment when no task is active");
        };
        self.task_streak += 1;
        *self
            .slayer_data
            .total_tasks_done
            .entry((master, monster))
            .or_default() += 1;
        *self.slayer_data.total_kills.entry(monster).or_default() += amount as u64;
        self.slayer_data.total_time += monster.task_time(amount);

        if self.task_streak >= 5 {
            let point_multiplier = if self.task_streak.is_multiple_of(1000) {
                50
            } else if self.task_streak.is_multiple_of(250) {
                35
            } else if self.task_streak.is_multiple_of(100) {
                25
            } else if self.task_streak.is_multiple_of(50) {
                15
            } else if self.task_streak.is_multiple_of(10) {
                5
            } else {
                1
            };
            let point_awarded = master.slayer_points() * point_multiplier;
            self.points += point_awarded;
            self.slayer_data.total_points += point_awarded as u64;
            self.slayer_data.max_points = self.slayer_data.max_points.max(self.points as u64);
        }
        self.task_state = TaskState::Completed(monster);
    }

    pub fn point_skip(&mut self) {
        let TaskState::Active((monster, _, _)) = self.task_state else {
            panic!("Expected an active task");
        };
        self.task_state = TaskState::Completed(monster);
        assert!(self.points >= 30);
        self.points -= 30;
        self.slayer_data.min_points = self.slayer_data.min_points.min(self.points as u64);
    }
}

struct PlayerState {
    slayer_level: u8,
    quests_done: Vec<Quest>,
}

impl PlayerState {
    pub fn can_receive_assignment(&self, assignment: &Assignment) -> bool {
        self.slayer_level >= assignment.monster.slayer_req()
            && assignment
                .quest_requirement
                .is_none_or(|quest| self.quests_done.contains(&quest))
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Assignment {
    monster: Monster,
    amount: RangeInclusive<u32>,
    quest_requirement: Option<Quest>,
    weight: u32,
}

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Monster {
    AberrantSpectres,
    AbyssalDemons,
    Ankous,
    Aviansie,
    Banshees,
    Basilisks,
    Bats,
    Bears,
    Birds,
    BlackDemons,
    Bloodveld,
    BlueDragons,
    BrineRats,
    CaveBugs,
    CaveCrawlers,
    CaveHorrors,
    CaveKraken,
    CaveSlimes,
    Cockatrice,
    Cows,
    Crabs,
    CrawlingHands,
    Crocodiles,
    CustodianStalker,
    Dagannoth,
    DustDevils,
    Dogs,
    Dwarves,
    Elves,
    FeverSpiders,
    FireGiants,
    FossilIslandWyverns,
    Gargoyles,
    Ghosts,
    Ghouls,
    Goblins,
    GreaterDemons,
    HarpieBugSwarms,
    Hellhounds,
    HillGiants,
    Hobgoblins,
    Icefiends,
    IceGiants,
    IceWarriors,
    InfernalMages,
    Jellies,
    JungleHorrors,
    Kalphite,
    Kurask,
    LesserDemons,
    LesserNagua,
    Lizardmen,
    Lizards,
    Minotaurs,
    Mogres,
    Molanisks,
    Monkeys,
    MossGiants,
    MutatedZygomites,
    Nechryael,
    Ogres,
    OtherwordlyBeings,
    Pyrefiends,
    Rats,
    Scorpions,
    SeaSnakes,
    Shades,
    ShadowWarriors,
    SkeletalWyverns,
    Skeletons,
    Sourhogs,
    Spiders,
    SpiritualCreatures,
    TerrorDogs,
    Trolls,
    Turoth,
    TzHaar,
    Vampyres,
    WarpedCreatures,
    Werewolves,
    Wolves,
    Wyrms,
    Zombies,
}

#[derive(Display, Clone, Copy, PartialEq, Eq)]
enum Quest {
    ActualVampyreSlayer,
    CabinFever,
    DeathPlateau,
    DeathToTheDorgeshuun,
    DesertTreasure,
    DragonSlayer,
    ElementalWorkshop,
    HauntedMine,
    HorrorFromTheDeep,
    HotStuff,
    LostCity,
    #[allow(clippy::enum_variant_names)]
    LegendsQuest,
    #[allow(clippy::enum_variant_names)]
    OlafsQuest,
    PerilousMoons,
    PorcineOfInterest,
    PriestInPeril,
    Regicide,
    ReptileGotRipped,
    RumDeal,
    ShadowsOfCustodia,
    SkippyAndTheMogres,
    RoyalTrouble,
    WarpedReality,
    WatchTheBirdie,
}
