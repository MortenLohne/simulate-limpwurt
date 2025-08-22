use std::{fmt, ops::RangeInclusive};

use rand::Rng;
use strum::Display;

fn main() {
    for assignment in VANNAKA_ASSIGNMENTS.iter() {
        println!("{}, weight {}", assignment.monster, assignment.weight);
    }

    let turael_tasks_weight_sum: u32 = TURAEL_ASSIGNMENTS
        .iter()
        .map(|assignment| assignment.weight)
        .sum();

    let vannaka_tasks_weight_sum: u32 = VANNAKA_ASSIGNMENTS
        .iter()
        .map(|assignment| assignment.weight)
        .sum();
    println!("Turael tasks weight sum: {}", turael_tasks_weight_sum);
    println!("Vannaka tasks number: {}", VANNAKA_ASSIGNMENTS.len());
    println!("Vannaka tasks weight sum: {}", vannaka_tasks_weight_sum);

    let limpwurt = PlayerState {
        slayer_level: 75,
        quests_done: vec![Quest::LostCity],
    };

    let limpwurt_turael_tasks_weight_sum: u32 = TURAEL_ASSIGNMENTS
        .iter()
        .filter(|assignment| limpwurt.can_receive_assignment(assignment))
        .map(|assignment| assignment.weight)
        .sum();
    let limpwurt_vannaka_tasks_weight_sum: u32 = VANNAKA_ASSIGNMENTS
        .iter()
        .filter(|assignment| limpwurt.can_receive_assignment(assignment))
        .map(|assignment| assignment.weight)
        .sum();
    println!(
        "Limpwurt Turael tasks weight sum: {}",
        limpwurt_turael_tasks_weight_sum
    );
    println!(
        "Limpwurt Vannaka tasks weight sum: {}",
        limpwurt_vannaka_tasks_weight_sum
    );

    let mut slayer_state = SlayerState {
        points: 0,
        task_state: TaskState::Completed(Monster::Monkeys),
    };

    let mut rng = rand::rng();

    slayer_state.new_assignment(&mut rng, SlayerMaster::Turael, &limpwurt);

    println!("New assignment: {}", slayer_state.task_state,);
}

#[derive(Display, Clone, PartialEq, Eq)]
enum SlayerMaster {
    Turael,
    Vannaka,
    Chaeldar,
}

impl SlayerMaster {
    pub fn assignments(&self) -> &[Assignment] {
        match self {
            SlayerMaster::Turael => TURAEL_ASSIGNMENTS,
            SlayerMaster::Vannaka => VANNAKA_ASSIGNMENTS,
            SlayerMaster::Chaeldar => unimplemented!(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum TaskState {
    Active(Assignment),
    Completed(Monster),
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskState::Active(assignment) => write!(f, "Active task: {}", assignment.monster),
            TaskState::Completed(monster) => write!(f, "Completed task: {}", monster),
        }
    }
}

struct SlayerState {
    points: u32,
    task_state: TaskState,
}

impl SlayerState {
    pub fn new_assignment<R: Rng>(
        &mut self,
        rng: &mut R,
        master: SlayerMaster,
        player_state: &PlayerState,
    ) {
        let possible_tasks: Vec<(u32, Assignment)> = master
            .assignments()
            .iter()
            .filter(|assignment| player_state.can_receive_assignment(*assignment))
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

        self.task_state = TaskState::Active(task);
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

#[derive(Clone, PartialEq, Eq)]
struct Assignment {
    monster: Monster,
    amount: RangeInclusive<u32>,
    quest_requirement: Option<Quest>,
    weight: u32,
}

#[derive(Display, Clone, Copy, PartialEq, Eq)]
enum Monster {
    AberrantSpectres,
    AbyssalDemons,
    Ankous,
    Banshees,
    Basilisks,
    Bats,
    Bears,
    Birds,
    Bloodveld,
    BlueDragons,
    BrineRats,
    CaveBugs,
    CaveCrawlers,
    CaveSlimes,
    Cockatrice,
    Cows,
    Crabs,
    CrawlingHands,
    Crocodiles,
    Dagannoth,
    DustDevils,
    Dogs,
    Dwarves,
    Elves,
    FeverSpiders,
    FireGiants,
    Gargoyles,
    Ghosts,
    Ghouls,
    Goblins,
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
    Lizards,
    Minotaurs,
    Mogres,
    Molanisks,
    Monkeys,
    MossGiants,
    Nechryael,
    Ogres,
    OtherwordlyBeings,
    Pyrefiends,
    Rats,
    Scorpions,
    SeaSnakes,
    Shades,
    ShadowWarriors,
    Skeletons,
    Spiders,
    SpiritualCreatures,
    TerrorDogs,
    Trolls,
    Turoth,
    Vampyres,
    Werewolves,
    Wolves,
    Zombies,
}

impl Monster {
    fn slayer_req(&self) -> u8 {
        match self {
            Monster::AberrantSpectres => 60,
            Monster::AbyssalDemons => 85,
            Monster::Ankous => 0,
            Monster::Banshees => 15,
            Monster::Basilisks => 40,
            Monster::Bats => 0,
            Monster::Bears => 0,
            Monster::Birds => 0,
            Monster::Bloodveld => 50,
            Monster::BlueDragons => 0,
            Monster::BrineRats => 47,
            Monster::CaveBugs => 7,
            Monster::CaveCrawlers => 10,
            Monster::CaveSlimes => 17,
            Monster::Cockatrice => 25,
            Monster::Cows => 0,
            Monster::Crabs => 0,
            Monster::CrawlingHands => 5,
            Monster::Crocodiles => 0,
            Monster::Dagannoth => 0,
            Monster::DustDevils => 65,
            Monster::Dogs => 0,
            Monster::Dwarves => 0,
            Monster::Elves => 0,
            Monster::FeverSpiders => 42,
            Monster::FireGiants => 0,
            Monster::Gargoyles => 75,
            Monster::Ghosts => 0,
            Monster::Ghouls => 0,
            Monster::Goblins => 0,
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
            Monster::Lizards => 22,
            Monster::Minotaurs => 0,
            Monster::Mogres => 32,
            Monster::Molanisks => 39,
            Monster::Monkeys => 0,
            Monster::MossGiants => 0,
            Monster::Nechryael => 80,
            Monster::Ogres => 0,
            Monster::OtherwordlyBeings => 0,
            Monster::Pyrefiends => 30,
            Monster::Rats => 0,
            Monster::Scorpions => 0,
            Monster::SeaSnakes => 40,
            Monster::Shades => 0,
            Monster::ShadowWarriors => 0,
            Monster::Skeletons => 0,
            Monster::Spiders => 0,
            Monster::SpiritualCreatures => 63,
            Monster::TerrorDogs => 40,
            Monster::Trolls => 0,
            Monster::Turoth => 55,
            Monster::Vampyres => 0,
            Monster::Werewolves => 0,
            Monster::Wolves => 0,
            Monster::Zombies => 0,
        }
    }
}

#[derive(Display, Clone, Copy, PartialEq, Eq)]
enum Quest {
    CabinFever,
    DeathToTheDorgeshuun,
    DesertTreasure,
    DragonSlayer,
    HorrorFromTheDeep,
    LegendsQuest,
    OlafsQuest,
    PriestInPeril,
    Regicide,
    RumDeal,
    SkippyAndTheMogres,
    RoyalTrouble,
    HauntedMine,
    DeathPlateau,
    LostCity,
}
