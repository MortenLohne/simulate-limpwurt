use std::{
    collections::HashMap,
    fmt,
    ops::RangeInclusive,
    time::{self, Duration},
};

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

    let mut max_points_locked = 0;

    for (slayer_data, success) in results {
        let num_tasks = slayer_data.total_tasks_started.values().sum::<u64>();
        num_tasks_received += num_tasks;
        if success {
            num_successes += 1;
            num_tasks_per_successful_run.push(num_tasks);
            min_points_per_successful_run.push(slayer_data.min_points);
        } else {
            max_points_locked = max_points_locked.max(slayer_data.max_points);
            num_tasks_per_failed_run.push(num_tasks);
        }
    }
    num_tasks_per_failed_run.sort();
    num_tasks_per_successful_run.sort();
    min_points_per_successful_run.sort();

    let median_successful_tasks = num_tasks_per_successful_run
        .get(num_tasks_per_successful_run.len() / 2)
        .unwrap_or(&0);
    let median_failed_tasks = num_tasks_per_failed_run
        .get(num_tasks_per_failed_run.len() / 2)
        .unwrap_or(&0);
    let median_min_points = min_points_per_successful_run
        .get(min_points_per_successful_run.len() / 2)
        .unwrap_or(&0);

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
        "Max points while eventually getting slayer-locked: {}, median min points on success: {}",
        max_points_locked, median_min_points
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

        // If we're about to get a big streak bonus, point skip
        if (slayer_state.task_streak + 1).is_multiple_of(50) && slayer_state.points >= 30 {
            slayer_state.point_skip();
            slayer_state.new_assignment(&mut rng, SlayerMaster::Vannaka, &limpwurt);
            continue;
        }

        let can_be_turael_skipped = TURAEL_ASSIGNMENTS
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
            SlayerMaster::Turael => TURAEL_ASSIGNMENTS,
            SlayerMaster::Spria => SPRIA_ASSIGNMENTS,
            SlayerMaster::Vannaka => VANNAKA_ASSIGNMENTS,
            SlayerMaster::Chaeldar => CHAELDAR_ASSIGNMENTS,
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
                if TURAEL_ASSIGNMENTS.iter().any(|assignment| {
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

const TURAEL_ASSIGNMENTS: &[Assignment] = &[
    Assignment {
        monster: Monster::Banshees,
        amount: 15..=30,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::Bats,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Bears,
        amount: 10..=20,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Birds,
        amount: 15..=30,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::CaveBugs,
        amount: 10..=30,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::CaveCrawlers,
        amount: 15..=30,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::CaveSlimes,
        amount: 10..=20,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Cows,
        amount: 15..=30,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::CrawlingHands,
        amount: 15..=30,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::Dogs,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Dwarves,
        amount: 10..=25,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Ghosts,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Goblins,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Icefiends,
        amount: 15..=20,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Kalphite,
        amount: 15..=30,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::Lizards,
        amount: 15..=30,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Minotaurs,
        amount: 10..=20,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Monkeys,
        amount: 15..=30,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::Rats,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Scorpions,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Skeletons,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Spiders,
        amount: 15..=30,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::Wolves,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Zombies,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
];

const SPRIA_ASSIGNMENTS: &[Assignment] = &[
    Assignment {
        monster: Monster::Banshees,
        amount: 15..=30,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::Bats,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Bears,
        amount: 10..=20,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Birds,
        amount: 15..=30,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::CaveBugs,
        amount: 10..=30,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::CaveCrawlers,
        amount: 15..=30,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::CaveSlimes,
        amount: 10..=20,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Cows,
        amount: 15..=30,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::CrawlingHands,
        amount: 15..=30,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::Dogs,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Dwarves,
        amount: 10..=25,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Ghosts,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Goblins,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Icefiends,
        amount: 15..=20,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Kalphite,
        amount: 15..=30,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::Lizards,
        amount: 15..=30,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Minotaurs,
        amount: 10..=20,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Monkeys,
        amount: 15..=30,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::Rats,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Scorpions,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Skeletons,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Sourhogs,
        amount: 15..=25,
        quest_requirement: Some(Quest::PorcineOfInterest),
        weight: 6,
    },
    Assignment {
        monster: Monster::Spiders,
        amount: 15..=30,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::Wolves,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Zombies,
        amount: 15..=30,
        quest_requirement: None,
        weight: 7,
    },
];

const VANNAKA_ASSIGNMENTS: &[Assignment] = &[
    Assignment {
        monster: Monster::AberrantSpectres,
        amount: 40..=90,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::AbyssalDemons,
        amount: 40..=90,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 5,
    },
    Assignment {
        monster: Monster::Ankous,
        amount: 25..=35,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Basilisks,
        amount: 40..=90,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Bloodveld,
        amount: 40..=90,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::BlueDragons,
        amount: 40..=90,
        quest_requirement: Some(Quest::DragonSlayer),
        weight: 7,
    },
    Assignment {
        monster: Monster::BrineRats,
        amount: 40..=90,
        quest_requirement: Some(Quest::OlafsQuest),
        weight: 7,
    },
    Assignment {
        monster: Monster::Cockatrice,
        amount: 40..=90,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Crabs,
        amount: 40..=90,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Crocodiles,
        amount: 40..=90,
        quest_requirement: None,
        weight: 6,
    },
    Assignment {
        monster: Monster::Dagannoth,
        amount: 40..=90,
        quest_requirement: Some(Quest::HorrorFromTheDeep),
        weight: 7,
    },
    Assignment {
        monster: Monster::DustDevils,
        amount: 40..=90,
        quest_requirement: Some(Quest::DesertTreasure),
        weight: 8,
    },
    Assignment {
        monster: Monster::Elves,
        amount: 30..=70,
        quest_requirement: Some(Quest::Regicide),
        weight: 7,
    },
    Assignment {
        monster: Monster::FeverSpiders,
        amount: 30..=90,
        quest_requirement: Some(Quest::RumDeal),
        weight: 7,
    },
    Assignment {
        monster: Monster::FireGiants,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Gargoyles,
        amount: 40..=90,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 5,
    },
    Assignment {
        monster: Monster::Ghouls,
        amount: 10..=40,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 7,
    },
    Assignment {
        monster: Monster::HarpieBugSwarms,
        amount: 40..=90,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Hellhounds,
        amount: 30..=60,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::HillGiants,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Hobgoblins,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::IceGiants,
        amount: 30..=80,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::IceWarriors,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::InfernalMages,
        amount: 40..=90,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::Jellies,
        amount: 40..=90,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::JungleHorrors,
        amount: 40..=90,
        quest_requirement: Some(Quest::CabinFever),
        weight: 8,
    },
    Assignment {
        monster: Monster::Kalphite,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Kurask,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::LesserDemons,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Mogres,
        amount: 40..=90,
        quest_requirement: Some(Quest::SkippyAndTheMogres),
        weight: 7,
    },
    Assignment {
        monster: Monster::Molanisks,
        amount: 40..=50,
        quest_requirement: Some(Quest::DeathToTheDorgeshuun),
        weight: 7,
    },
    Assignment {
        monster: Monster::MossGiants,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Nechryael,
        amount: 40..=90,
        quest_requirement: None,
        weight: 5,
    },
    Assignment {
        monster: Monster::Ogres,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::OtherwordlyBeings,
        amount: 40..=90,
        quest_requirement: Some(Quest::LostCity),
        weight: 8,
    },
    Assignment {
        monster: Monster::Pyrefiends,
        amount: 40..=90,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::SeaSnakes,
        amount: 40..=90,
        quest_requirement: Some(Quest::RoyalTrouble),
        weight: 6,
    },
    Assignment {
        monster: Monster::Shades,
        amount: 40..=90,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::ShadowWarriors,
        amount: 40..=90,
        quest_requirement: Some(Quest::LegendsQuest),
        weight: 8,
    },
    Assignment {
        monster: Monster::SpiritualCreatures,
        amount: 40..=90,
        quest_requirement: Some(Quest::DeathPlateau),
        weight: 8,
    },
    Assignment {
        monster: Monster::TerrorDogs,
        amount: 40..=90,
        quest_requirement: Some(Quest::HauntedMine),
        weight: 6,
    },
    Assignment {
        monster: Monster::Trolls,
        amount: 40..=90,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::Turoth,
        amount: 30..=90,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::Vampyres,
        amount: 10..=20,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 7,
    },
    Assignment {
        monster: Monster::Werewolves,
        amount: 30..=60,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 7,
    },
];

const CHAELDAR_ASSIGNMENTS: &[Assignment] = &[
    Assignment {
        monster: Monster::AberrantSpectres,
        amount: 70..=130,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::AbyssalDemons,
        amount: 70..=130,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 12,
    },
    Assignment {
        monster: Monster::Aviansie,
        amount: 70..=130,
        quest_requirement: Some(Quest::WatchTheBirdie),
        weight: 9,
    },
    Assignment {
        monster: Monster::Basilisks,
        amount: 70..=130,
        quest_requirement: None,
        weight: 7,
    },
    Assignment {
        monster: Monster::BlackDemons,
        amount: 70..=130,
        quest_requirement: None,
        weight: 10,
    },
    Assignment {
        monster: Monster::Bloodveld,
        amount: 70..=130,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 8,
    },
    Assignment {
        monster: Monster::BlueDragons,
        amount: 70..=130,
        quest_requirement: Some(Quest::DragonSlayer),
        weight: 8,
    },
    Assignment {
        monster: Monster::BrineRats,
        amount: 70..=130,
        quest_requirement: Some(Quest::OlafsQuest),
        weight: 7,
    },
    Assignment {
        monster: Monster::CaveHorrors,
        amount: 70..=130,
        quest_requirement: Some(Quest::CabinFever),
        weight: 10,
    },
    Assignment {
        monster: Monster::CaveKraken,
        amount: 30..=50,
        quest_requirement: None,
        weight: 12,
    },
    Assignment {
        monster: Monster::Crabs,
        amount: 70..=130,
        quest_requirement: None,
        weight: 8,
    },
    Assignment {
        monster: Monster::CustodianStalker,
        amount: 70..=130,
        quest_requirement: Some(Quest::ShadowsOfCustodia),
        weight: 11,
    },
    Assignment {
        monster: Monster::Dagannoth,
        amount: 70..=130,
        quest_requirement: Some(Quest::HorrorFromTheDeep),
        weight: 11,
    },
    Assignment {
        monster: Monster::DustDevils,
        amount: 70..=130,
        quest_requirement: Some(Quest::DesertTreasure),
        weight: 9,
    },
    Assignment {
        monster: Monster::Elves,
        amount: 70..=130,
        quest_requirement: Some(Quest::Regicide),
        weight: 8,
    },
    Assignment {
        monster: Monster::FeverSpiders,
        amount: 70..=130,
        quest_requirement: Some(Quest::RumDeal),
        weight: 7,
    },
    Assignment {
        monster: Monster::FireGiants,
        amount: 70..=130,
        quest_requirement: None,
        weight: 12,
    },
    Assignment {
        monster: Monster::FossilIslandWyverns,
        amount: 10..=20,
        quest_requirement: Some(Quest::ElementalWorkshop),
        weight: 7,
    },
    Assignment {
        monster: Monster::Gargoyles,
        amount: 70..=130,
        quest_requirement: Some(Quest::PriestInPeril),
        weight: 11,
    },
    Assignment {
        monster: Monster::GreaterDemons,
        amount: 70..=130,
        quest_requirement: None,
        weight: 9,
    },
    Assignment {
        monster: Monster::Hellhounds,
        amount: 70..=130,
        quest_requirement: None,
        weight: 9,
    },
    Assignment {
        monster: Monster::Jellies,
        amount: 70..=130,
        quest_requirement: None,
        weight: 10,
    },
    Assignment {
        monster: Monster::JungleHorrors,
        amount: 70..=130,
        quest_requirement: Some(Quest::CabinFever),
        weight: 10,
    },
    Assignment {
        monster: Monster::Kalphite,
        amount: 70..=130,
        quest_requirement: None,
        weight: 11,
    },
    Assignment {
        monster: Monster::Kurask,
        amount: 70..=130,
        quest_requirement: None,
        weight: 12,
    },
    Assignment {
        monster: Monster::LesserDemons,
        amount: 70..=130,
        quest_requirement: None,
        weight: 9,
    },
    Assignment {
        monster: Monster::LesserNagua,
        amount: 50..=100,
        quest_requirement: Some(Quest::PerilousMoons),
        weight: 4,
    },
    Assignment {
        monster: Monster::Lizardmen,
        amount: 50..=90,
        quest_requirement: Some(Quest::ReptileGotRipped),
        weight: 8,
    },
    Assignment {
        monster: Monster::MutatedZygomites,
        amount: 8..=15,
        quest_requirement: Some(Quest::LostCity),
        weight: 7,
    },
    Assignment {
        monster: Monster::Nechryael,
        amount: 70..=130,
        quest_requirement: None,
        weight: 12,
    },
    Assignment {
        monster: Monster::ShadowWarriors,
        amount: 70..=130,
        quest_requirement: Some(Quest::LegendsQuest),
        weight: 8,
    },
    Assignment {
        monster: Monster::SkeletalWyverns,
        amount: 10..=20,
        quest_requirement: Some(Quest::ElementalWorkshop),
        weight: 7,
    },
    Assignment {
        monster: Monster::SpiritualCreatures,
        amount: 70..=130,
        quest_requirement: Some(Quest::DeathPlateau),
        weight: 12,
    },
    Assignment {
        monster: Monster::Trolls,
        amount: 70..=130,
        quest_requirement: None,
        weight: 11,
    },
    Assignment {
        monster: Monster::Turoth,
        amount: 70..=130,
        quest_requirement: None,
        weight: 10,
    },
    Assignment {
        monster: Monster::TzHaar,
        amount: 90..=150,
        quest_requirement: Some(Quest::HotStuff),
        weight: 8,
    },
    Assignment {
        monster: Monster::Vampyres,
        amount: 80..=100,
        quest_requirement: Some(Quest::ActualVampyreSlayer),
        weight: 6,
    },
    Assignment {
        monster: Monster::WarpedCreatures,
        amount: 70..=130,
        quest_requirement: Some(Quest::WarpedReality),
        weight: 6,
    },
    Assignment {
        monster: Monster::Wyrms,
        amount: 60..=100,
        quest_requirement: None,
        weight: 6,
    },
];

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

impl Monster {
    fn can_limpwurt_kill(self) -> bool {
        use Monster::*;
        match self {
            AberrantSpectres => false,
            AbyssalDemons => false,
            Ankous => true,
            Aviansie => false,
            Banshees => false,
            Basilisks => false,
            Bats => true,
            Bears => true,
            Birds => true,
            BlackDemons => true,
            Bloodveld => WORLD_STATE != WorldState::Limp2024,
            BlueDragons => false,
            BrineRats => false,
            CaveBugs => true,
            CaveCrawlers => true,
            CaveHorrors => false,
            CaveKraken => false,
            CaveSlimes => true,
            Cockatrice => false,
            Cows => true,
            Crabs => false,
            CrawlingHands => false,
            Crocodiles => true,
            CustodianStalker => false,
            Dagannoth => false,
            DustDevils => false,
            Dogs => true,
            Dwarves => true,
            Elves => false,
            FeverSpiders => false,
            FireGiants => true,
            FossilIslandWyverns => false,
            Gargoyles => false,
            Ghosts => true,
            Ghouls => false,
            Goblins => true,
            GreaterDemons => WORLD_STATE != WorldState::Limp2024,
            HarpieBugSwarms => false,
            Hellhounds => false,
            HillGiants => true,
            Hobgoblins => true,
            Icefiends => true,
            IceGiants => true,
            IceWarriors => true,
            InfernalMages => false,
            Jellies => false,
            JungleHorrors => false,
            Kalphite => true,
            Kurask => false,
            LesserDemons => false,
            LesserNagua => false,
            Lizardmen => false,
            Lizards => true,
            Minotaurs => true,
            Mogres => false,
            Molanisks => false,
            Monkeys => false,
            MossGiants => true,
            MutatedZygomites => WORLD_STATE != WorldState::Limp2024,
            Nechryael => false,
            Ogres => false,
            OtherwordlyBeings => WORLD_STATE != WorldState::Limp2024,
            Pyrefiends => WORLD_STATE != WorldState::Limp2024,
            Rats => true,
            Scorpions => true,
            SeaSnakes => false,
            Shades => true,
            ShadowWarriors => false,
            SkeletalWyverns => WORLD_STATE != WorldState::Limp2024,
            Skeletons => true,
            Sourhogs => true,
            Spiders => true,
            SpiritualCreatures => WORLD_STATE != WorldState::Limp2024,
            TerrorDogs => false,
            Trolls => false,
            Turoth => false,
            TzHaar => false,
            Vampyres => false,
            WarpedCreatures => false,
            Werewolves => false,
            Wolves => true,
            Wyrms => false,
            Zombies => true,
        }
    }
    fn slayer_req(&self) -> u8 {
        match self {
            Monster::AberrantSpectres => 60,
            Monster::AbyssalDemons => 85,
            Monster::Ankous => 0,
            Monster::Aviansie => 0,
            Monster::Banshees => 15,
            Monster::Basilisks => 40,
            Monster::Bats => 0,
            Monster::Bears => 0,
            Monster::Birds => 0,
            Monster::BlackDemons => 0,
            Monster::Bloodveld => 50,
            Monster::BlueDragons => 0,
            Monster::BrineRats => 47,
            Monster::CaveBugs => 7,
            Monster::CaveCrawlers => 10,
            Monster::CaveHorrors => 58,
            Monster::CaveKraken => 87,
            Monster::CaveSlimes => 17,
            Monster::Cockatrice => 25,
            Monster::Cows => 0,
            Monster::Crabs => 0,
            Monster::CrawlingHands => 5,
            Monster::Crocodiles => 0,
            Monster::CustodianStalker => 54,
            Monster::Dagannoth => 0,
            Monster::DustDevils => 65,
            Monster::Dogs => 0,
            Monster::Dwarves => 0,
            Monster::Elves => 0,
            Monster::FeverSpiders => 42,
            Monster::FireGiants => 0,
            Monster::FossilIslandWyverns => 66,
            Monster::Gargoyles => 75,
            Monster::Ghosts => 0,
            Monster::Ghouls => 0,
            Monster::Goblins => 0,
            Monster::GreaterDemons => 0,
            Monster::HarpieBugSwarms => 33,
            Monster::Hellhounds => 0,
            Monster::HillGiants => 0,
            Monster::Hobgoblins => 0,
            Monster::Icefiends => 0,
            Monster::IceGiants => 0,
            Monster::IceWarriors => 0,
            Monster::InfernalMages => 45,
            Monster::Jellies => 52,
            Monster::JungleHorrors => 0,
            Monster::Kalphite => 0,
            Monster::Kurask => 70,
            Monster::LesserDemons => 0,
            Monster::LesserNagua => 48,
            Monster::Lizardmen => 0,
            Monster::Lizards => 22,
            Monster::Minotaurs => 0,
            Monster::Mogres => 32,
            Monster::Molanisks => 39,
            Monster::Monkeys => 0,
            Monster::MossGiants => 0,
            Monster::MutatedZygomites => 57,
            Monster::Nechryael => 80,
            Monster::Ogres => 0,
            Monster::OtherwordlyBeings => 0,
            Monster::Pyrefiends => 30,
            Monster::Rats => 0,
            Monster::Scorpions => 0,
            Monster::SeaSnakes => 40,
            Monster::Shades => 0,
            Monster::ShadowWarriors => 0,
            Monster::SkeletalWyverns => 72,
            Monster::Skeletons => 0,
            Monster::Sourhogs => 1,
            Monster::Spiders => 0,
            Monster::SpiritualCreatures => 63,
            Monster::TerrorDogs => 40,
            Monster::Trolls => 0,
            Monster::Turoth => 55,
            Monster::TzHaar => 0,
            Monster::Vampyres => 0,
            Monster::WarpedCreatures => 0,
            Monster::Werewolves => 0,
            Monster::Wolves => 0,
            Monster::Wyrms => 62,
            Monster::Zombies => 0,
        }
    }
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

#[test]
fn turael_total_weight_test() {
    let total_weight: u32 = TURAEL_ASSIGNMENTS.iter().map(|a| a.weight).sum();
    assert_eq!(total_weight, 172);

    let player_state = PlayerState {
        slayer_level: 75,
        quests_done: vec![Quest::LostCity],
    };
    let player_total_weight = total_weight_prop(&player_state, SlayerMaster::Turael);
    assert_eq!(player_total_weight, 156);
}

#[test]
fn vannaka_total_weight_test() {
    let total_weight: u32 = VANNAKA_ASSIGNMENTS.iter().map(|a| a.weight).sum();
    assert_eq!(total_weight, 322);

    let player_state = PlayerState {
        slayer_level: 75,
        quests_done: vec![Quest::LostCity],
    };
    let player_total_weight = total_weight_prop(&player_state, SlayerMaster::Vannaka);
    assert_eq!(player_total_weight, 169);
}

#[test]
fn chaeldar_total_weight_test() {
    let total_weight: u32 = CHAELDAR_ASSIGNMENTS.iter().map(|a| a.weight).sum();
    assert_eq!(total_weight, 350);

    let player_state = PlayerState {
        slayer_level: 75,
        quests_done: vec![Quest::LostCity],
    };
    let player_total_weight = total_weight_prop(&player_state, SlayerMaster::Chaeldar);
    assert_eq!(player_total_weight, 131);
}

#[cfg(test)]
fn total_weight_prop(player_state: &PlayerState, master: SlayerMaster) -> u32 {
    master
        .assignments()
        .iter()
        .filter(|assignment| player_state.can_receive_assignment(assignment))
        .map(|assignment| assignment.weight)
        .sum()
}

#[test]
fn all_monster_are_assigned_test() {
    use std::collections::BTreeMap;
    let mut frequency: BTreeMap<Monster, u32> = BTreeMap::new();
    let player = PlayerState {
        slayer_level: 75,
        quests_done: vec![Quest::LostCity, Quest::PorcineOfInterest],
    };
    let mut slayer_state = SlayerState {
        points: 0,
        task_streak: 0,
        task_state: TaskState::Completed(Monster::Monkeys),
        slayer_data: SlayerData::default(),
    };

    let mut rng = rand::rng();

    const N: u32 = 100_000;
    let slayer_master = SlayerMaster::Turael;

    for _ in 0..N {
        slayer_state.new_assignment(&mut rng, slayer_master, &player);

        let TaskState::Active((monster, _, _)) = slayer_state.task_state else {
            panic!();
        };
        *frequency.entry(monster).or_insert(0) += 1;
        slayer_state.complete_assignment();
    }

    assert_eq!(slayer_state.task_streak, N);

    assert_eq!(
        frequency.len(),
        slayer_master
            .assignments()
            .iter()
            .filter(|a| { player.can_receive_assignment(a) })
            .count(),
        "{:?}",
        frequency
    );

    for (monster, count) in frequency {
        println!("{}: {:.2}%", monster, 100.0 * count as f32 / N as f32);
    }
}
