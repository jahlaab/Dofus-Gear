pub mod character;
pub mod dofapi;
pub mod rls;

extern crate rand;
extern crate rayon;
extern crate serde_json;

use rand::prelude::*;
use rayon::prelude::*;

use crate::character::Character;
use crate::dofapi::carac::CaracKind;
use crate::dofapi::effect::Element;
use crate::dofapi::equipement::{Equipement, ItemType};
use crate::rls::Blackbox;

use std::fs::File;
use std::io;

fn main() -> io::Result<()> {
    // ---
    eprintln!("-- Loading data...");
    let equipements: Vec<Equipement> = {
        let file = File::open("./data/allequipments.json")?;
        serde_json::from_reader(io::BufReader::new(file))?
    };
    let equipements: Vec<Equipement> = equipements
        .into_iter()
        .filter(|item| item.level >= 150 || item.item_type == ItemType::Dofus)
        .collect();

    // --- Index per slot
    eprintln!("-- Build index of available items for each slot...");
    let slot_pool: Vec<_> = Character::new()
        .item_slots
        .iter()
        .map(|slot| {
            equipements
                .iter()
                .filter(|item| slot.get_allowed().contains(&item.item_type))
                .collect::<Vec<_>>()
        })
        .collect();

    // ---
    eprintln!("-- Build random stuffs...");

    let best = (0..16)
        .into_par_iter()
        .map_init(
            || rand::thread_rng(),
            |mut rng, _| {
                Character::bb_find_max(
                    Character::new(),
                    1_000_000,
                    &mut rng,
                    |mut new, rng| -> Character {
                        let slot_i = rng.gen_range(0, slot_pool.len());
                        let item = slot_pool[slot_i]
                            .choose(rng)
                            .expect("No available item for slot");
                        new.item_slots[slot_i].equip(item);
                        new
                    },
                )
            },
        )
        .max_by(|c1, c2| c1.eval().partial_cmp(&c2.eval()).unwrap());

    // ---
    eprintln!("-- Result...");
    match best {
        None => println!("No feasible stuff found :("),
        Some(character) => {
            println!("------------------------------------------------");
            character
                .item_slots
                .iter()
                .for_each(|i| println!(" {:^46}", i.get_item().unwrap().name));
            println!("------------------------------------------------");

            let stats = &[
                CaracKind::AP,
                CaracKind::MP,
                CaracKind::Range,
                CaracKind::Vitality,
                CaracKind::Initiative,
                CaracKind::Stats(Element::Air),
                CaracKind::Stats(Element::Earth),
                CaracKind::Stats(Element::Fire),
                CaracKind::Stats(Element::Water),
                CaracKind::Power,
                CaracKind::Critical,
                CaracKind::CriticalDamage,
                CaracKind::Wisdom,
                CaracKind::APResistance,
                CaracKind::MPResistance,
                CaracKind::PerResistance(Element::Air),
                CaracKind::PerResistance(Element::Earth),
                CaracKind::PerResistance(Element::Fire),
                CaracKind::PerResistance(Element::Water),
                CaracKind::PerResistance(Element::Neutral),
            ];

            let caracs = character.get_caracs();
            for stat in stats {
                println!(" {:35} {:>10}", stat, caracs.get_carac(stat));
            }
            println!("conflicts: {}", character.count_item_conflicts());
        }
    }

    Ok(())
}
