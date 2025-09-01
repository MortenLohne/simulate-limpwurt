use crate::{
    Location, Monster, PlayerState, Quest, SlayerData, SlayerMaster, SlayerState, TaskState, data,
};

#[test]
fn turael_total_weight_test() {
    let total_weight: u32 = data::TURAEL_ASSIGNMENTS.iter().map(|a| a.weight).sum();
    assert_eq!(total_weight, 172);

    let player_state = PlayerState::new(1_308_538, vec![Quest::LostCity], false);
    let player_total_weight = total_weight_prop(&player_state, SlayerMaster::Turael);
    assert_eq!(player_total_weight, 156);
}

#[test]
fn vannaka_total_weight_test() {
    let total_weight: u32 = data::VANNAKA_ASSIGNMENTS.iter().map(|a| a.weight).sum();
    assert_eq!(total_weight, 322);

    let player_state = PlayerState::new(1_308_538, vec![Quest::LostCity], false);
    let player_total_weight = total_weight_prop(&player_state, SlayerMaster::Vannaka);
    assert_eq!(player_total_weight, 169);
}

#[test]
fn chaeldar_total_weight_test() {
    let total_weight: u32 = data::CHAELDAR_ASSIGNMENTS.iter().map(|a| a.weight).sum();
    assert_eq!(total_weight, 350);

    let player_state = PlayerState::new(1_308_538, vec![Quest::LostCity], false);
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

    let mut player = PlayerState::new(
        1_308_538,
        vec![Quest::LostCity, Quest::PorcineOfInterest],
        false,
    );

    let mut slayer_state = SlayerState {
        points: 0,
        task_streak: 0,
        task_state: TaskState::Completed(Monster::Monkeys),
        stored_task: None,
        slayer_data: SlayerData::default(),
        location: Location::SlayerMaster(SlayerMaster::Turael),
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
        slayer_state.complete_assignment(&mut rng, &mut player);
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
