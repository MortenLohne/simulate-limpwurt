use std::time::Duration;

use crate::{Monster, SlayerMaster, Supplies};

const GAME_TICK: Duration = Duration::from_millis(600);
pub const STORE_TASK_TIME: Duration = Duration::from_secs(3);
pub const UNSTORE_TASK_TIME: Duration = Duration::from_secs(3);

impl Supplies {
    pub fn time_to_gather(&self) -> Duration {
        Duration::from_millis(3033) * self.expeditious_bracelet_charges as u32 // 91 seconds per bracelet
        + Duration::from_secs(46) * self.bracelet_of_slaughter_charges as u32 // 23 minutes per bracelet
        + Duration::from_secs(8) * self.games_necklace_charges as u32 // 66 seconds per necklace, TODO: Made up
        + Duration::from_secs(8) * self.dueling_ring_charges as u32 // 66 seconds per necklace, TODO: Made up
        + Duration::from_secs(3) * self.necklace_of_passage_charges as u32 // 24 seconds per necklace, assuming spare jades from opal grind
        + Duration::from_millis(500) * self.chronicle_charges as u32
        + Duration::from_secs(2) * self.skull_sceptre_charges as u32 // 15 seconds per sceptre, TODO: Made up
        + Duration::from_secs(8) * self.giantsoul_amulet_charges as u32 // Get big bones from hill giants, and other giants on-task
        + Duration::from_millis(500) * self.law_runes as u32
    }

    pub fn print_time_breakdown(&self) {
        println!("Supplies gathering time:");
        println!(
            "Opals for Expeditious Bracelets: {:.1} hours",
            (Duration::from_millis(3033) * self.expeditious_bracelet_charges as u32).as_secs_f64()
                / 3600.0
        );
        println!(
            "Red topaz for Bracelets of Slaughter: {:.1} hours",
            (Duration::from_secs(46) * self.bracelet_of_slaughter_charges as u32).as_secs_f64()
                / 3600.0
        );
        println!(
            "Games Necklaces: {:.1} hours",
            (Duration::from_secs(8) * self.games_necklace_charges as u32).as_secs_f64() / 3600.0
        );
        println!(
            "Dueling Rings: {:.1} hours",
            (Duration::from_secs(8) * self.dueling_ring_charges as u32).as_secs_f64() / 3600.0
        );
        println!(
            "Necklaces of Passage: {:.1} hours",
            (Duration::from_secs(3) * self.necklace_of_passage_charges as u32).as_secs_f64()
                / 3600.0
        );
        println!(
            "Chronicle charges: {:.1} hours",
            (Duration::from_millis(500) * self.chronicle_charges as u32).as_secs_f64() / 3600.0
        );
        println!(
            "Skull Sceptre charges: {:.1} hours",
            (Duration::from_secs(2) * self.skull_sceptre_charges as u32).as_secs_f64() / 3600.0
        );
        println!(
            "Big Bones for Giantsoul Amulet charges: {:.1} hours",
            (Duration::from_secs(8) * self.giantsoul_amulet_charges as u32).as_secs_f64() / 3600.0
        );
        println!(
            "Law Runes: {:.1} hours",
            (Duration::from_millis(500) * self.law_runes as u32).as_secs_f64() / 3600.0
        );
    }
}

#[derive(Default)]
pub struct MonsterData {
    pub travel_steps: u32,
    pub time_per_kill: Duration,
    pub travel_supplies: Supplies,
    pub superior_unique_drop_rate: Option<f32>,
    pub use_expeditious_bracelet: bool,
    pub use_bracelet_of_slaughter: bool,
}

impl MonsterData {
    pub fn travel_time(&self) -> Duration {
        (GAME_TICK * self.travel_steps).div_f32(1.5) // Assume that we run 50% of the time
    }
}

impl Monster {
    pub fn task_data(&self) -> Option<MonsterData> {
        match self {
            Monster::AberrantSpectres => None,
            Monster::AbyssalDemons => None,
            Monster::Ankous => Some(MonsterData {
                travel_steps: 80,
                time_per_kill: Duration::from_millis(13200),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Aviansie => None,
            Monster::Banshees => None,
            Monster::Basilisks => None,
            Monster::Bats => Some(MonsterData {
                travel_steps: 306,
                time_per_kill: Duration::from_millis(3300),
                travel_supplies: Supplies {
                    chronicle_charges: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::Bears => Some(MonsterData {
                travel_steps: 112,
                time_per_kill: Duration::from_millis(8300),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Birds => Some(MonsterData {
                travel_steps: 14,
                time_per_kill: Duration::from_millis(2200),
                travel_supplies: Supplies {
                    chronicle_charges: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::BlackDemons => None,
            Monster::Bloodveld => None,
            Monster::BlueDragons => None,
            Monster::BrineRats => None,
            Monster::CaveBugs => Some(MonsterData {
                travel_steps: 190,
                time_per_kill: Duration::from_millis(3100),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::CaveCrawlers => Some(MonsterData {
                travel_steps: 190,
                time_per_kill: Duration::from_millis(7600),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                superior_unique_drop_rate: Some(1.0 / 166.2),
                use_bracelet_of_slaughter: true,
                ..Default::default()
            }),
            Monster::CaveHorrors => None,
            Monster::CaveKraken => None,
            Monster::CaveSlimes => Some(MonsterData {
                travel_steps: 190,
                time_per_kill: Duration::from_millis(8700),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Cockatrice => None,
            Monster::Cows => Some(MonsterData {
                travel_steps: 66,
                time_per_kill: Duration::from_millis(3400),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::Crabs => None,
            Monster::CrawlingHands => None,
            Monster::Crocodiles => Some(MonsterData {
                travel_steps: 103,
                time_per_kill: Duration::from_millis(17100),
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::CustodianStalker => None,
            Monster::Dagannoth => None,
            Monster::DustDevils => None,
            Monster::Dogs => Some(MonsterData {
                travel_steps: 120,
                time_per_kill: Duration::from_millis(8900),
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Dwarves => Some(MonsterData {
                travel_steps: 100,
                time_per_kill: Duration::from_millis(7600),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Elves => None,
            Monster::FeverSpiders => None,
            Monster::FireGiants => Some(MonsterData {
                travel_steps: 0,
                time_per_kill: Duration::from_millis(480_000),
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::FossilIslandWyverns => None,
            Monster::Gargoyles => None,
            Monster::Ghosts => Some(MonsterData {
                travel_steps: 200,
                time_per_kill: Duration::from_millis(7300),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Ghouls => None,
            Monster::Goblins => Some(MonsterData {
                travel_steps: 32,
                time_per_kill: Duration::from_millis(2600),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::GreaterDemons => None,
            Monster::HarpieBugSwarms => None,
            Monster::Hellhounds => None,
            Monster::HillGiants => Some(MonsterData {
                travel_steps: 5,
                time_per_kill: Duration::from_millis(7800),
                travel_supplies: Supplies {
                    giantsoul_amulet_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: false, // Big bones are needed for giantsoul amulet charges
                ..Default::default()
            }),
            Monster::Hobgoblins => Some(MonsterData {
                travel_steps: 89,
                time_per_kill: Duration::from_millis(11_000),
                travel_supplies: Supplies {
                    giantsoul_amulet_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Icefiends => Some(MonsterData {
                travel_steps: 140,
                time_per_kill: Duration::from_millis(5500),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::IceGiants => Some(MonsterData {
                travel_steps: 10,
                time_per_kill: Duration::from_millis(11_200),
                travel_supplies: Supplies {
                    giantsoul_amulet_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::IceWarriors => Some(MonsterData {
                travel_steps: 136,
                time_per_kill: Duration::from_millis(10_000),
                travel_supplies: Supplies {
                    giantsoul_amulet_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::InfernalMages => None,
            Monster::Jellies => None,
            Monster::JungleHorrors => None,
            Monster::Kalphite => Some(MonsterData {
                travel_steps: 60,
                time_per_kill: Duration::from_millis(10500),
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Kurask => None,
            Monster::LesserDemons => None,
            Monster::LesserNagua => None,
            Monster::Lizardmen => None,
            Monster::Lizards => Some(MonsterData {
                travel_steps: 108,
                time_per_kill: Duration::from_millis(4700),
                ..Default::default()
            }),
            Monster::Minotaurs => Some(MonsterData {
                travel_steps: 44,
                time_per_kill: Duration::from_millis(3800),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::Mogres => None,
            Monster::Molanisks => None,
            Monster::Monkeys => Some(MonsterData {
                travel_steps: 120,
                time_per_kill: Duration::from_millis(4100),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::MossGiants => Some(MonsterData {
                travel_steps: 22,
                time_per_kill: Duration::from_millis(12_600),
                travel_supplies: Supplies {
                    giantsoul_amulet_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::MutatedZygomites => None,
            Monster::Nechryael => None,
            Monster::Ogres => None,
            Monster::OtherwordlyBeings => Some(MonsterData {
                travel_steps: 240,
                time_per_kill: Duration::from_millis(14_000),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Pyrefiends => Some(MonsterData {
                travel_steps: 535,
                time_per_kill: Duration::from_millis(15000),
                superior_unique_drop_rate: Some(1.0 / 142.2),
                use_bracelet_of_slaughter: true,
                ..Default::default()
            }),
            Monster::Rats => Some(MonsterData {
                travel_steps: 20,
                time_per_kill: Duration::from_millis(2600),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::Scorpions => Some(MonsterData {
                travel_steps: 66,
                time_per_kill: Duration::from_millis(5200),
                travel_supplies: Supplies {
                    dueling_ring_charges: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::SeaSnakes => None,
            Monster::Shades => Some(MonsterData {
                travel_steps: 70,
                time_per_kill: Duration::from_millis(40_200),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::ShadowWarriors => None,
            Monster::SkeletalWyverns => None,
            Monster::Skeletons => Some(MonsterData {
                travel_steps: 100,
                time_per_kill: Duration::from_millis(8100),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Sourhogs => Some(MonsterData {
                travel_steps: 72,
                time_per_kill: Duration::from_millis(8000), // TODO: Guesstimated
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::Spiders => Some(MonsterData {
                travel_steps: 76,
                time_per_kill: Duration::from_millis(3000),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::SpiritualCreatures => None,
            Monster::TerrorDogs => None,
            Monster::Trolls => Some(MonsterData {
                travel_steps: 74,
                time_per_kill: Duration::from_millis(24_000),
                travel_supplies: Supplies {
                    games_necklace_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
            Monster::Turoth => None,
            Monster::TzHaar => None,
            Monster::Vampyres => None,
            Monster::WarpedCreatures => None,
            Monster::Werewolves => None,
            Monster::Wolves => Some(MonsterData {
                travel_steps: 40,
                time_per_kill: Duration::from_millis(3800),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::Wyrms => None,
            Monster::Zombies => Some(MonsterData {
                travel_steps: 104,
                time_per_kill: Duration::from_millis(8300),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                use_expeditious_bracelet: true,
                ..Default::default()
            }),
        }
    }
}

impl SlayerMaster {
    pub fn travel_time(&self) -> Duration {
        match self {
            SlayerMaster::Turael => Duration::from_secs(16),
            SlayerMaster::Spria => Duration::from_secs(34),
            SlayerMaster::Vannaka => Duration::from_secs(32),
            SlayerMaster::Chaeldar => Duration::from_secs(49),
        }
    }

    pub fn travel_cost(&self) -> Supplies {
        match self {
            SlayerMaster::Turael => Supplies {
                games_necklace_charges: 1,
                ..Default::default()
            },
            SlayerMaster::Spria => Supplies {
                necklace_of_passage_charges: 1,
                ..Default::default()
            },
            SlayerMaster::Vannaka => Supplies {
                giantsoul_amulet_charges: 1,
                ..Default::default()
            },
            SlayerMaster::Chaeldar => Supplies {
                law_runes: 1,
                ..Default::default()
            },
        }
    }
}
