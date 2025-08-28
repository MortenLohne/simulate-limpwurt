use std::time::Duration;

use crate::{Assignment, Monster, SlayerData, SlayerMaster, Supplies, TaskState};

const GAME_TICK: Duration = Duration::from_millis(600);

#[derive(Default)]
struct MonsterData {
    travel_steps: u32,
    time_per_kill: Duration,
    travel_supplies: Supplies,
    superior_unique_drop_rate: Option<f32>,
    use_expeditious_bracelet: bool,
    use_bracelet_of_slaughter: bool,
}

impl Monster {
    pub fn task_time(&self, amount: u32) -> Duration {
        let monster_data = self.task_data().unwrap_or(MonsterData {
            travel_steps: 100, // TODO: This is a completely made up and wrong average for Vannaka tasks
            time_per_kill: Duration::from_millis(30_000),
            ..Default::default()
        });
        let travel_time = (GAME_TICK * monster_data.travel_steps).div_f32(1.5); // Assume that we run 50% of the time
        let kill_time = monster_data.time_per_kill * amount as u32;
        travel_time + kill_time
    }

    pub fn task_data(&self) -> Option<MonsterData> {
        match self {
            Monster::AberrantSpectres => None,
            Monster::AbyssalDemons => None,
            Monster::Ankous => None,
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
            Monster::Crocodiles => None,
            Monster::CustodianStalker => None,
            Monster::Dagannoth => None,
            Monster::DustDevils => None,
            Monster::Dogs => Some(MonsterData {
                travel_steps: 120,
                time_per_kill: Duration::from_millis(8900),
                ..Default::default()
            }),
            Monster::Dwarves => Some(MonsterData {
                travel_steps: 100,
                time_per_kill: Duration::from_millis(7600),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::Elves => None,
            Monster::FeverSpiders => None,
            Monster::FireGiants => None,
            Monster::FossilIslandWyverns => None,
            Monster::Gargoyles => None,
            Monster::Ghosts => Some(MonsterData {
                travel_steps: 200,
                time_per_kill: Duration::from_millis(7300),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
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
            Monster::HillGiants => None,
            Monster::Hobgoblins => None,
            Monster::Icefiends => Some(MonsterData {
                travel_steps: 140,
                time_per_kill: Duration::from_millis(5500),
                travel_supplies: Supplies {
                    law_runes: 1,
                    ..Default::default()
                },
                ..Default::default()
            }),
            Monster::IceGiants => None,
            Monster::IceWarriors => None,
            Monster::InfernalMages => None,
            Monster::Jellies => None,
            Monster::JungleHorrors => None,
            Monster::Kalphite => Some(MonsterData {
                travel_steps: 60,
                time_per_kill: Duration::from_millis(10500),
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
            Monster::Monkeys => None,
            Monster::MossGiants => None,
            Monster::MutatedZygomites => None,
            Monster::Nechryael => None,
            Monster::Ogres => None,
            Monster::OtherwordlyBeings => None,
            Monster::Pyrefiends => None,
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
            Monster::Shades => None,
            Monster::ShadowWarriors => None,
            Monster::SkeletalWyverns => None,
            Monster::Skeletons => Some(MonsterData {
                travel_steps: 100,
                time_per_kill: Duration::from_millis(8100),
                travel_supplies: Supplies {
                    skull_sceptre_charges: 1,
                    ..Default::default()
                },
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
            Monster::Trolls => None,
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
                ..Default::default()
            }),
        }
    }
}

impl SlayerData {
    pub fn assignment_cost(&mut self, master: SlayerMaster) {
        match master {
            SlayerMaster::Turael => {
                self.supplies_used.games_necklace_charges += 1;
                self.total_time += Duration::from_secs(16);
            }
            SlayerMaster::Spria => {
                self.supplies_used.necklace_of_passage_charges += 1;
                self.total_time += Duration::from_secs(34);
            }
            SlayerMaster::Vannaka => {
                self.supplies_used.skull_sceptre_charges += 1;
                self.total_time += Duration::from_secs(60);
            }
            SlayerMaster::Chaeldar => {
                self.supplies_used.law_runes += 1;
                self.total_time += Duration::from_secs(49);
            }
        }
    }
}
