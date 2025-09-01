use std::{
    collections::BTreeMap,
    fmt,
    ops::{self, RangeInclusive},
    time::{self, Duration},
};

use SlayerMaster::*;

mod costs;
mod data;
#[cfg(test)]
mod tests;

use rand::{Rng, SeedableRng, rngs::SmallRng};
use rayon::prelude::*;
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::costs::{MonsterData, STORE_TASK_TIME, UNSTORE_TASK_TIME};

#[derive(Display, PartialEq, Eq)]
#[allow(dead_code)]
enum WorldState {
    Limp2024,
    Limp2025,
    Limp2026,
}

const WORLD_STATE: WorldState = WorldState::Limp2026;

fn main() {
    let _start = match WORLD_STATE {
        WorldState::Limp2024 => SimulationStartPoint {
            slayer_exp: 168_538,
            quests_done: vec![Quest::PorcineOfInterest],
            task_streak: 0,
            points: 0,
            task_state: TaskState::Active((Monster::Hellhounds, Vannaka, 40)),
            storage_unlocked: false,
        },
        WorldState::Limp2025 => SimulationStartPoint {
            slayer_exp: 1_308_538,
            quests_done: vec![Quest::LostCity, Quest::PorcineOfInterest],
            task_streak: 1,
            points: 120,
            task_state: TaskState::Active((Monster::Monkeys, Turael, 20)),
            storage_unlocked: false,
        },
        WorldState::Limp2026 => SimulationStartPoint {
            slayer_exp: 1_308_538,
            quests_done: vec![
                Quest::LostCity,
                Quest::PorcineOfInterest,
                Quest::DragonSlayer,
            ],
            task_streak: 1,
            points: 120,
            task_state: TaskState::Active((Monster::Monkeys, Turael, 20)),
            storage_unlocked: false,
        },
    };
    // run_slayer_start_simulation();
    run_superiors_simulation();
}

fn run_simulation<S: Strategy + Clone + Send>(start: SimulationStartPoint) {
    let start_time = time::Instant::now();
    let n = 10_000;

    let results: Vec<_> = (0..n)
        .into_par_iter()
        .map(|_| simulate_limpwurt(start.clone(), S::default()))
        .collect();

    let mut num_successes = 0;
    let mut num_tasks_received: u64 = 0;
    let mut num_tasks_per_failed_run = vec![];
    let mut num_tasks_per_successful_run = vec![];
    let mut min_points_per_successful_run = vec![];
    let mut total_points_per_successful_run = vec![];
    let mut end_points_per_successful_run = vec![];

    let mut all_successful_runs = vec![];

    let mut max_points_locked = 0;
    let mut all_drops = SlayerDrops::default();
    let mut all_supplies = Supplies::default();
    let mut cave_crawlers_killed = 0;

    for (slayer_state, player_state, success) in results {
        let slayer_data = slayer_state.slayer_data.clone();
        let num_tasks = slayer_data.total_tasks_started.values().sum::<u64>();
        num_tasks_received += num_tasks;
        if success {
            num_successes += 1;
            num_tasks_per_successful_run.push(num_tasks);
            min_points_per_successful_run.push(slayer_data.min_points);
            total_points_per_successful_run.push(slayer_data.total_points);
            end_points_per_successful_run.push(slayer_state.points as u64);
            all_successful_runs.push((slayer_state.clone(), player_state.clone()));
        } else {
            max_points_locked = max_points_locked.max(slayer_data.max_points);
            num_tasks_per_failed_run.push(num_tasks);
        }
        all_drops = all_drops + slayer_data.drops;
        all_supplies = all_supplies + slayer_data.supplies_used;
        cave_crawlers_killed += slayer_data
            .total_kills
            .get(&Monster::CaveCrawlers)
            .unwrap_or(&0);
    }
    num_tasks_per_failed_run.sort();
    num_tasks_per_successful_run.sort();
    min_points_per_successful_run.sort();
    total_points_per_successful_run.sort();
    end_points_per_successful_run.sort();
    all_successful_runs.sort_by_cached_key(|(data, _)| data.slayer_data.time_spent());

    let median_successful_tasks = num_tasks_per_successful_run
        .get(num_tasks_per_successful_run.len() / 2)
        .unwrap_or(&0);
    let median_failed_tasks = num_tasks_per_failed_run
        .get(num_tasks_per_failed_run.len() / 2)
        .unwrap_or(&0);
    let median_min_points = min_points_per_successful_run
        .get(min_points_per_successful_run.len() / 2)
        .unwrap_or(&0);
    let (median_run, median_player_data) = all_successful_runs
        .get(all_successful_runs.len() / 2)
        .unwrap_or(&Default::default())
        .clone();
    let median_total_points = total_points_per_successful_run
        .get(total_points_per_successful_run.len() / 2)
        .unwrap_or(&0);
    let median_end_points = end_points_per_successful_run
        .get(end_points_per_successful_run.len() / 2)
        .unwrap_or(&0);
    let total_hours = all_successful_runs
        .iter()
        .map(|(run, _)| run.slayer_data.time_spent().as_secs_f32() / 3600.0)
        .sum::<f32>();

    println!(
        "All drops {:?}, {} cave crawlers killed",
        all_drops, cave_crawlers_killed
    );

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
        all_successful_runs
            .first()
            .unwrap_or(&Default::default())
            .0
            .slayer_data
            .time_spent()
            .as_secs_f32()
            / 3600.0,
        median_run.slayer_data.time_spent().as_secs_f32() / 3600.0,
        all_successful_runs
            .last()
            .unwrap_or(&Default::default())
            .0
            .slayer_data
            .time_spent()
            .as_secs_f32()
            / 3600.0,
    );
    println!(
        "Average time: {:.1} hours",
        total_hours / all_successful_runs.len() as f32
    );
    println!(
        "Median total points: {}, median end points: {}",
        median_total_points, median_end_points
    );

    println!("Median simulation:");
    println!(
        "{} total points, {} total exp, {} end level, {:.1} total hours, {} total tasks",
        median_run.slayer_data.total_points,
        median_player_data.slayer_exp,
        median_player_data.slayer_level,
        median_run.slayer_data.time_spent().as_secs_f32() / 3600.0,
        median_run
            .slayer_data
            .total_tasks_done
            .values()
            .sum::<u64>()
    );
    println!(
        "New time spent: {:.1} hours",
        median_run.slayer_data.time_spent().as_secs_f32() / 3600.0
    );
    println!("Supplies used: {:?}", median_run.slayer_data.supplies_used);
    println!(
        "{:.1} hours spent gathering supplies, {:.1} hours total",
        median_run
            .slayer_data
            .supplies_used
            .time_to_gather()
            .as_secs_f32()
            / 3600.0,
        median_run.slayer_data.time_spent().as_secs_f32() / 3600.0
    );
    println!();
    println!("Total tasks done per slayer master:");
    for ((master, monster), kills) in median_run.slayer_data.total_tasks_done {
        println!("{:10} {:16} {}", master, monster, kills);
    }
    println!("Average tasks done per slayer master:");
    for master in SlayerMaster::iter() {
        for monster in Monster::iter() {
            let kills = all_successful_runs
                .iter()
                .map(|run| {
                    run.0
                        .slayer_data
                        .total_tasks_done
                        .get(&(master, monster))
                        .unwrap_or(&0)
                })
                .sum::<u64>();
            if kills > 0 {
                println!(
                    "{:10} {:16} {}",
                    master,
                    monster,
                    kills / all_successful_runs.len() as u64
                );
            }
        }
    }
    println!();
    println!("Total kills:");
    for (monster, kills) in median_run.slayer_data.total_kills {
        println!("{:16} {}", monster, kills);
    }

    println!("Finished in {:.1}s", start_time.elapsed().as_secs_f32());

    // let mut buckets: BTreeMap<u32, u32> = (0..=600).map(|x| (x, 0)).collect::<BTreeMap<_, _>>();
    // for (run, _) in all_successful_runs.iter() {
    //     // Bucket is the total time, measured is hundreds of hours;
    //     let bucket = (run.slayer_data.time_spent().as_secs_f32() / (3600.0 * 100.0)) as u32;
    //     *buckets.get_mut(&bucket.min(600)).unwrap() += 1;
    // }

    // for (points, count) in buckets {
    //     println!("[{}, {}],", points * 100, count);
    // }
}

pub fn run_superiors_simulation() {
    // Simulation is only valid after the slayer update
    assert!(WORLD_STATE == WorldState::Limp2026);

    let start = SimulationStartPoint {
        slayer_exp: 1_308_538,
        quests_done: vec![Quest::LostCity, Quest::PorcineOfInterest],
        task_streak: 1,
        points: 120,
        task_state: TaskState::Active((Monster::Monkeys, Turael, 20)),
        storage_unlocked: false,
    };

    run_simulation::<SuperiorsStrategy>(start);
}

pub fn run_slayer_start_simulation() {
    // Simulation is only valid after the slayer update
    assert!(WORLD_STATE == WorldState::Limp2026);

    let start = SimulationStartPoint {
        slayer_exp: 1_308_538,
        quests_done: vec![Quest::LostCity, Quest::PorcineOfInterest],
        task_streak: 1,
        points: 120,
        task_state: TaskState::Active((Monster::Monkeys, Turael, 20)),
        storage_unlocked: false,
    };

    run_simulation::<MinimizeSlayerLockStrategy>(start);
}

#[derive(Clone)]
struct SimulationStartPoint {
    slayer_exp: u32,
    quests_done: Vec<Quest>,
    task_streak: u32,
    points: u32,
    task_state: TaskState,
    storage_unlocked: bool,
}

enum SimulationAction {
    CompleteTask,
    PointSkip,
    NewAssignment(SlayerMaster),
    UnlockTaskStorage,
    StoreTask,
    UnstoreTask,
}

trait Strategy: Default {
    fn should_terminate(
        &mut self,
        slayer_state: &SlayerState,
        player_state: &PlayerState,
    ) -> Option<bool>;
    fn select_action(
        &mut self,
        slayer_state: &SlayerState,
        player_state: &PlayerState,
    ) -> SimulationAction;
}

#[derive(Default, Clone)]
struct MinimizeSlayerLockStrategy {}

impl Strategy for MinimizeSlayerLockStrategy {
    fn should_terminate(
        &mut self,
        slayer_state: &SlayerState,
        player_state: &PlayerState,
    ) -> Option<bool> {
        match slayer_state.task_state {
            TaskState::Active((monster, _, _)) => {
                if !monster.can_limpwurt_kill()
                    && slayer_state.points < 30
                    && Turael.can_assign(monster)
                    && (!player_state.storage_unlocked || slayer_state.stored_task.is_some())
                {
                    Some(false)
                } else {
                    None
                }
            }

            TaskState::Completed(_) if slayer_state.points >= 1000 => Some(true),
            TaskState::Completed(_) => None,
        }
    }

    fn select_action(
        &mut self,
        slayer_state: &SlayerState,
        _player_state: &PlayerState,
    ) -> SimulationAction {
        match slayer_state.task_state {
            TaskState::Active((monster, _, _)) => {
                if monster.can_limpwurt_kill() {
                    SimulationAction::CompleteTask
                } else if Turael.can_assign(monster) {
                    if slayer_state.points >= 30 {
                        SimulationAction::PointSkip
                    } else {
                        panic!("Ran out of slayer points, simulation should have stopped already");
                    }
                } else {
                    SimulationAction::NewAssignment(Turael)
                }
            }
            TaskState::Completed(_) => {
                let streak_after_next_task = slayer_state.task_streak + 1;
                let next_slayer_master =
                    if streak_after_next_task >= 5 && streak_after_next_task % 10 <= 4 {
                        Vannaka
                    } else {
                        Spria
                    };
                SimulationAction::NewAssignment(next_slayer_master)
            }
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq)]
enum SuperiorsStrategy {
    #[default]
    AccumulatePoints,
    GetSuperiors,
}

impl Strategy for SuperiorsStrategy {
    fn should_terminate(
        &mut self,
        slayer_state: &SlayerState,
        player_state: &PlayerState,
    ) -> Option<bool> {
        // Stop once we have all the superior drops
        if slayer_state.slayer_data.drops.dust_battlestaff > 0
            && slayer_state.slayer_data.drops.mist_battlestaff > 0
            && slayer_state.slayer_data.drops.imbued_heart > 0
            && slayer_state.slayer_data.drops.eternal_gem > 0
        {
            return Some(true);
        }
        if let TaskState::Active((monster, _, _)) = slayer_state.task_state {
            if !monster.can_limpwurt_kill()
                && slayer_state.points < 30
                && Turael.can_assign(monster)
                && (!player_state.storage_unlocked || slayer_state.stored_task.is_some())
            {
                Some(false)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn select_action(
        &mut self,
        slayer_state: &SlayerState,
        player_state: &PlayerState,
    ) -> SimulationAction {
        use Monster::*;
        if !player_state.storage_unlocked {
            if slayer_state.points >= 620 {
                return SimulationAction::UnlockTaskStorage;
            }
            return MinimizeSlayerLockStrategy::default().select_action(slayer_state, player_state);
        }
        if *self == SuperiorsStrategy::AccumulatePoints && slayer_state.points > 1000 {
            *self = SuperiorsStrategy::GetSuperiors;
        } else if *self == SuperiorsStrategy::GetSuperiors && slayer_state.points < 500 {
            *self = SuperiorsStrategy::AccumulatePoints;
        }
        match (slayer_state.task_state, self.clone()) {
            (TaskState::Active((monster, master, _)), SuperiorsStrategy::AccumulatePoints) => {
                if monster.can_limpwurt_kill() {
                    // Turael-skip Vannaka tasks that are too slow
                    if master == Vannaka {
                        if [
                            Ankous,
                            Crocodiles,
                            IceGiants,
                            IceWarriors,
                            HillGiants,
                            Hobgoblins,
                            Kalphite,
                            MossGiants,
                            Pyrefiends,
                            Trolls,
                        ]
                        .contains(&monster)
                        {
                            SimulationAction::CompleteTask
                        } else if slayer_state.points >= 120 {
                            SimulationAction::PointSkip
                        } else {
                            SimulationAction::NewAssignment(Turael)
                        }
                    } else {
                        SimulationAction::CompleteTask
                    }
                } else if Turael.can_assign(monster) {
                    if slayer_state.stored_task.is_none() {
                        SimulationAction::StoreTask
                    } else if slayer_state.points >= 30 {
                        SimulationAction::PointSkip
                    } else {
                        panic!("Ran out of slayer points, simulation should have stopped already");
                    }
                } else if slayer_state.points > 120 {
                    SimulationAction::PointSkip
                } else {
                    SimulationAction::NewAssignment(Turael)
                }
            }
            (TaskState::Active((monster, master, _)), SuperiorsStrategy::GetSuperiors) => {
                if monster.can_limpwurt_kill() {
                    // Turael-skip Vannaka tasks that are too slow
                    if master == Vannaka {
                        if [Kalphite, Pyrefiends].contains(&monster) {
                            SimulationAction::CompleteTask
                        } else {
                            SimulationAction::NewAssignment(Turael)
                        }
                    } else {
                        SimulationAction::CompleteTask
                    }
                } else if Turael.can_assign(monster) {
                    if slayer_state.stored_task.is_none() {
                        SimulationAction::StoreTask
                    } else if slayer_state.points >= 30 {
                        SimulationAction::PointSkip
                    } else {
                        panic!("Ran out of slayer points, simulation should have stopped already");
                    }
                } else {
                    SimulationAction::NewAssignment(Turael)
                }
            }
            (TaskState::Completed(last_monster), SuperiorsStrategy::AccumulatePoints) => {
                // Only do Vannaka tasks every 10 task
                let streak_after_next_task = slayer_state.task_streak + 1;
                let next_slayer_master = if streak_after_next_task % 10 == 0 {
                    Vannaka
                } else {
                    Turael
                };

                // We don't want to have a task we can as our "last task", because it cannot be assigned again immediately
                // We'd rather unstore a bad task, so that can re-store it, and not be assigned it this time
                if last_monster.can_limpwurt_kill()
                    && next_slayer_master.can_assign(last_monster)
                    && let Some((stored_monster, _, _)) = slayer_state.stored_task
                    && !stored_monster.can_limpwurt_kill()
                    && next_slayer_master.can_assign(stored_monster)
                {
                    SimulationAction::UnstoreTask
                } else {
                    SimulationAction::NewAssignment(next_slayer_master)
                }
            }
            (TaskState::Completed(last_monster), SuperiorsStrategy::GetSuperiors) => {
                // We don't want to have a task we can as our "last task", because it cannot be assigned again immediately
                // We'd rather unstore a bad task, so that can re-store it, and not be assigned it this time
                if last_monster.can_limpwurt_kill()
                    && Vannaka.can_assign(last_monster)
                    && let Some((stored_monster, _, _)) = slayer_state.stored_task
                    && !stored_monster.can_limpwurt_kill()
                    && Vannaka.can_assign(stored_monster)
                {
                    SimulationAction::UnstoreTask
                } else {
                    SimulationAction::NewAssignment(Vannaka)
                }
            }
        }
    }
}

/// Returns the number of tasks received, the minimum/maximum points reached, and whether he escaped (i.e. got lots of points)
fn simulate_limpwurt<S: Strategy + Clone + Send>(
    start: SimulationStartPoint,
    mut strategy: S,
) -> (SlayerState, PlayerState, bool) {
    let mut limpwurt =
        PlayerState::new(start.slayer_exp, start.quests_done, start.storage_unlocked);

    let mut slayer_state = SlayerState {
        task_streak: start.task_streak,
        points: start.points,
        task_state: start.task_state,
        stored_task: None,
        slayer_data: SlayerData::default(),
    };

    let mut rng = SmallRng::from_os_rng();

    loop {
        if let Some(result) = strategy.should_terminate(&slayer_state, &limpwurt) {
            return (slayer_state, limpwurt, result);
        }

        let action = strategy.select_action(&slayer_state, &limpwurt);

        match action {
            SimulationAction::CompleteTask => {
                slayer_state.complete_assignment(&mut rng, &mut limpwurt)
            }
            SimulationAction::PointSkip => slayer_state.point_skip(),
            SimulationAction::NewAssignment(master) => {
                slayer_state.new_assignment(&mut rng, master, &limpwurt)
            }
            SimulationAction::UnlockTaskStorage => {
                assert!(!limpwurt.storage_unlocked);
                limpwurt.storage_unlocked = true;
                slayer_state.points -= 500;
            }
            SimulationAction::StoreTask => slayer_state.store_task(&limpwurt),
            SimulationAction::UnstoreTask => slayer_state.unstore_task(),
        }
    }
}

#[derive(EnumIter, Display, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[allow(dead_code)]
enum SlayerMaster {
    Turael,
    Spria,
    Vannaka,
    Chaeldar,
}

impl SlayerMaster {
    pub fn can_assign(self, monster: Monster) -> bool {
        self.assignments()
            .iter()
            .any(|assignment| assignment.monster == monster)
    }

    pub fn assignments(&self) -> &[Assignment] {
        match self {
            Turael => data::TURAEL_ASSIGNMENTS,
            Spria => data::SPRIA_ASSIGNMENTS,
            Vannaka => data::VANNAKA_ASSIGNMENTS,
            Chaeldar => data::CHAELDAR_ASSIGNMENTS,
        }
    }

    pub fn slayer_points(&self) -> u32 {
        match self {
            Turael => 0,
            Spria => 0,
            Vannaka => match WORLD_STATE {
                WorldState::Limp2024 => 4,
                WorldState::Limp2025 => 4,
                WorldState::Limp2026 => 8,
            },
            Chaeldar => 10,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TaskState {
    Active((Monster, SlayerMaster, u32)), // (monster, master, amount)
    Completed(Monster),
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Completed(Monster::Monkeys)
    }
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
    total_tasks_started: BTreeMap<(SlayerMaster, Monster), u64>,
    total_tasks_done: BTreeMap<(SlayerMaster, Monster), u64>,
    total_kills: BTreeMap<Monster, u64>, // Tracks the number of actual kills, not the number assigned
    num_stored_tasks: u64,               // Only tracked for timekeeping
    num_unstored_tasks: u64,             // Only tracked for timekeeping
    supplies_used: Supplies,
    drops: SlayerDrops,
}

impl SlayerData {
    pub fn time_spent(&self) -> Duration {
        let mut total_time = Duration::ZERO;

        for ((master, _monster), amount) in self.total_tasks_started.iter() {
            total_time += master.travel_time() * *amount as u32;
        }
        for ((_master, monster), amount) in self.total_tasks_done.iter() {
            let monster_data = monster.task_data().unwrap_or(MonsterData {
                travel_steps: 100, // TODO: This is a completely made up and wrong average for Vannaka tasks
                time_per_kill: Duration::from_millis(30_000),
                ..Default::default()
            });
            total_time += monster_data.travel_time() * *amount as u32;
        }
        for (monster, kills) in self.total_kills.iter() {
            let monster_data = monster.task_data().unwrap_or(MonsterData {
                travel_steps: 100, // TODO: This is a completely made up and wrong average for Vannaka tasks
                time_per_kill: Duration::from_millis(30_000),
                ..Default::default()
            });
            total_time += monster_data.time_per_kill * *kills as u32;
        }
        total_time += STORE_TASK_TIME * self.num_stored_tasks as u32;
        total_time += UNSTORE_TASK_TIME * self.num_unstored_tasks as u32;

        total_time += self.supplies_used.time_to_gather();
        total_time
    }
}

impl Default for SlayerData {
    fn default() -> Self {
        Self {
            total_points: 0,
            min_points: u64::MAX,
            max_points: u64::MIN,
            total_tasks_started: BTreeMap::new(),
            total_tasks_done: BTreeMap::new(),
            total_kills: BTreeMap::new(),
            num_stored_tasks: 0,
            num_unstored_tasks: 0,
            supplies_used: Supplies::default(),
            drops: SlayerDrops::default(),
        }
    }
}

#[derive(Default, Clone, Debug)]
struct Supplies {
    expeditious_bracelet_charges: u64,
    bracelet_of_slaughter_charges: u64,
    games_necklace_charges: u64,
    dueling_ring_charges: u64,
    necklace_of_passage_charges: u64,
    chronicle_charges: u64,
    skull_sceptre_charges: u64,
    giantsoul_amulet_charges: u64,
    law_runes: u64,
}

impl ops::Add for Supplies {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            expeditious_bracelet_charges: self.expeditious_bracelet_charges
                + rhs.expeditious_bracelet_charges,
            bracelet_of_slaughter_charges: self.bracelet_of_slaughter_charges
                + rhs.bracelet_of_slaughter_charges,
            games_necklace_charges: self.games_necklace_charges + rhs.games_necklace_charges,
            dueling_ring_charges: self.dueling_ring_charges + rhs.dueling_ring_charges,
            necklace_of_passage_charges: self.necklace_of_passage_charges
                + rhs.necklace_of_passage_charges,
            chronicle_charges: self.chronicle_charges + rhs.chronicle_charges,
            skull_sceptre_charges: self.skull_sceptre_charges + rhs.skull_sceptre_charges,
            giantsoul_amulet_charges: self.giantsoul_amulet_charges + rhs.giantsoul_amulet_charges,
            law_runes: self.law_runes + rhs.law_runes,
        }
    }
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
struct SlayerDrops {
    dust_battlestaff: u64,
    mist_battlestaff: u64,
    imbued_heart: u64,
    eternal_gem: u64,
}

impl ops::Add for SlayerDrops {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            dust_battlestaff: self.dust_battlestaff + rhs.dust_battlestaff,
            mist_battlestaff: self.mist_battlestaff + rhs.mist_battlestaff,
            imbued_heart: self.imbued_heart + rhs.imbued_heart,
            eternal_gem: self.eternal_gem + rhs.eternal_gem,
        }
    }
}

#[derive(Clone, Default)]
struct SlayerState {
    points: u32,
    task_streak: u32,
    task_state: TaskState,
    stored_task: Option<(Monster, SlayerMaster, u32)>,
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
            Turael => (),
            Spria => {
                assert!(player_state.quests_done.contains(&Quest::PorcineOfInterest))
            }
            Vannaka => (),
            Chaeldar => assert!(player_state.quests_done.contains(&Quest::LostCity)),
        }
        let last_task = match self.task_state {
            TaskState::Active((monster, _, _)) => {
                // If this is a Turael skip, reset the task counter
                self.task_streak = 0;
                if master != Turael {
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
            .fold(
                Vec::with_capacity(master.assignments().len()),
                |mut acc, assignment| {
                    acc.push((
                        acc.last().map(|(weight, _)| *weight).unwrap_or(0) + assignment.weight,
                        assignment.clone(),
                    ));
                    acc
                },
            );

        let turael_tasks_weight_sum: u32 = possible_tasks.last().map_or(0, |(weight, _)| *weight);

        let task_num = rng.random_range(0..turael_tasks_weight_sum);

        let task = possible_tasks
            .into_iter()
            .find(|(weight, _)| *weight > task_num)
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

    pub fn store_task(&mut self, player_state: &PlayerState) {
        if !player_state.storage_unlocked {
            panic!("Cannot store task when storage is not unlocked");
        }
        let TaskState::Active((monster, master, amount)) = self.task_state else {
            panic!("Expected an active task");
        };
        if self.stored_task.is_some() {
            panic!("Cannot store task when one is already stored");
        }
        self.stored_task = Some((monster, master, amount));
        self.task_state = TaskState::Completed(monster);
        self.slayer_data.num_stored_tasks += 1;
    }

    pub fn unstore_task(&mut self) {
        let Some((monster, master, amount)) = self.stored_task.take() else {
            panic!("Cannot unstore task when none is stored");
        };
        let TaskState::Completed(_) = self.task_state else {
            panic!("Cannot unstore task with another already active");
        };
        self.task_state = TaskState::Active((monster, master, amount));
        self.slayer_data.num_unstored_tasks += 1;
    }

    pub fn complete_assignment<R: Rng>(&mut self, rng: &mut R, player_state: &mut PlayerState) {
        let TaskState::Active((monster, master, amount)) = self.task_state else {
            panic!("Cannot complete assignment when no task is active");
        };
        self.task_streak += 1;
        *self
            .slayer_data
            .total_tasks_done
            .entry((master, monster))
            .or_default() += 1;

        let task_data = monster.task_data().unwrap_or(MonsterData {
            travel_steps: 100, // TODO: This is a completely made up and wrong average for Vannaka tasks
            time_per_kill: Duration::from_millis(30_000),
            ..Default::default()
        });

        self.slayer_data.supplies_used =
            self.slayer_data.supplies_used.clone() + task_data.travel_supplies.clone();

        let superior_rare_drop_chance = task_data.superior_unique_drop_rate;

        // If the monster has a superior, or we're using a slayer bracelet, simulate each individual kill
        if superior_rare_drop_chance.is_some()
            || task_data.use_bracelet_of_slaughter
            || task_data.use_expeditious_bracelet
        {
            let mut kills_left: u32 = amount;
            while kills_left > 0 {
                *self.slayer_data.total_kills.entry(monster).or_default() += 1;
                player_state.slayer_exp += monster.slayer_exp();

                if task_data.use_bracelet_of_slaughter && rng.random::<f32>() < 0.25 {
                    self.slayer_data.supplies_used.bracelet_of_slaughter_charges += 1;
                    kills_left += 1; // The kill is subtracted later
                }
                if task_data.use_expeditious_bracelet && rng.random::<f32>() < 0.25 {
                    self.slayer_data.supplies_used.expeditious_bracelet_charges += 1;
                    kills_left -= 1;
                }

                if let Some(superior_rare_drop_chance) = superior_rare_drop_chance
                    && rng.random::<f32>() < (1.0 / 200.0)
                {
                    kills_left = kills_left.saturating_sub(1); // The superior counts as an extra kill
                    let main_roll = rng.random::<f32>();
                    if main_roll < superior_rare_drop_chance {
                        let udt_roll = rng.random::<f32>();
                        if udt_roll < 1.0 / 2.286 {
                            self.slayer_data.drops.dust_battlestaff += 1;
                        } else if udt_roll < 2.0 / 2.286 {
                            self.slayer_data.drops.mist_battlestaff += 1;
                        } else {
                            self.slayer_data.drops.imbued_heart += 1;
                        }
                    } else if main_roll < 2.0 * superior_rare_drop_chance {
                        let udt_roll = rng.random::<f32>();
                        if udt_roll < 1.0 / 8.0 {
                            self.slayer_data.drops.eternal_gem += 1;
                        }
                    }
                }
                kills_left = kills_left.saturating_sub(1);
            }
        } else {
            *self.slayer_data.total_kills.entry(monster).or_default() += amount as u64;
            player_state.slayer_exp += monster.slayer_exp() * amount;
        }
        player_state.slayer_level = data::level_for_exp(player_state.slayer_exp);

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

#[derive(Clone, Default)]
struct PlayerState {
    slayer_exp: u32,
    slayer_level: u8,
    quests_done: Vec<Quest>,
    storage_unlocked: bool,
}

impl PlayerState {
    pub fn new(slayer_exp: u32, quests_done: Vec<Quest>, storage_unlocked: bool) -> Self {
        Self {
            slayer_exp,
            slayer_level: data::level_for_exp(slayer_exp),
            quests_done,
            storage_unlocked,
        }
    }

    pub fn slayer_level(&self) -> u8 {
        self.slayer_level
    }

    pub fn can_receive_assignment(&self, assignment: &Assignment) -> bool {
        self.slayer_level() >= assignment.monster.slayer_req()
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

#[derive(EnumIter, Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
