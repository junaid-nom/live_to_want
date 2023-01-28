

use std::cmp::Ordering;
use std::thread;
use std::time;
use std::{cell::{Ref, RefCell}, rc::Rc};
use std::collections::HashMap;
use std::ops::Deref;
use std::{fmt::{Debug, Formatter}, borrow::Borrow};
use std::sync::{Arc, atomic::AtomicU64};
use core::fmt;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

extern crate rayon;
use rayon::prelude::*;

mod map_state;
pub use map_state::*;

mod networking;
pub use networking::*;

mod utils;
pub use utils::*;

mod creature;
pub use creature::*;

mod tasks;
pub use tasks::*;

mod ai;
pub use ai::*;

// NOTE: All event chains with items need to end in a final failure case of putting item on ground.
// this is because you can try to give an item away as someone else fills your inventory and 
// if both giving fails and putting back into your inventory fails, need to put item somewhere, so put on ground.

// DECISION:
// All VecVecs have 0,0 as bottom left corner.
// Exit points will be blocked off if there is no neighboring region.
// Exits must never be blocked from each other or path finding gets fucked, so only allow all exits of a side to be blocked if there is no region on that side at all!

// TODO Big: 
// Pretty sure items in a MapLocation and inventory in a creature state don't have to be rc<refcell<>>
// Can convert all the Vec<Vec<>>s to a single vector with a wrapper struct that deals with like indexing with x,y using mod %. Will be faster.

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameState {
    pub map_state: MapState,
}

pub fn unwrap_option_list(opList: Vec<Option<EventChain>>) -> Vec<EventChain> {
    let ret :Vec<EventChain> = opList.into_par_iter().flat_map(
        |opt| {
            if let Some(ec) = opt {
                vec![ec]
            } else {
                Vec::new()
            }
        }).collect();
    return ret;
}

pub async fn game() {
    // TODONEXT (UNTESTED)
    // Make initial map state
    let map_state = create_basic_map_state();
    let mut game_state = GameState {
        map_state,
    };
    // generate initial goal root
    let goal_node = GoalNode::generate_single_node_graph();
    // start server
    let mut server = ConnectionManager::new().await;
    
    
    // loop
    loop {
        // get input from connections
        // run frame
        // TODO: if in super-fast mode, just loop
        // TODO: if in user controlled just check for input until receive something
        // TODO: also can do "slow" mode with a wait

        let msgs = server.get_messages();
        game_state = run_frame_with_input(game_state, &goal_node, msgs);
        thread::sleep(time::Duration::from_millis(1000));
        //server.send_message_all();
    }
}

pub fn run_frame(mut game_state: GameState, root: &GoalNode)  -> GameState {
    run_frame_with_input(game_state, root, vec![])
}

pub fn run_frame_with_input(mut game_state: GameState, root: &GoalNode, msgs: Vec<GameMessageWrapUsername>) -> GameState {
    let mut m = game_state.map_state;
    {
        m.frame_count += 1;
    }
    let current_frame = m.frame_count;
    let battle_list_id = m.battle_list.id;

    // Run spawn systems first, and spawn new creatures
    let spawn_events: Vec<EventChain> = m.regions.par_iter().flat_map(|x| {
        x.par_iter().flat_map(|y| {
            y.grid.par_iter().flat_map(|xl| {
                xl.par_iter().flat_map(|yl| {
                    if let Some(par_iter) = yl.creatures.get_par_iter() {
                        let ret: Vec<EventChain> = par_iter.flat_map(
                            |c| {
                                let mut all_chains = budding_system(&m, c);
                                all_chains.extend(soil_spread_system(&m, c));
                                all_chains.extend(sex_reproduction_system(&m, c));
                                all_chains
                            }
                        ).collect();
                        return ret;
                    } else {
                        Vec::new()
                    }
                })
            })
        })
    }).collect();

    process_events_from_mapstate(&mut m, spawn_events);

    // TODO: Deal with blockers that spawned
    // first vec is BLOCKERS second vec is NON BLOCKERS
    let blocked_creatures: Vec<(Vec<CreatureState>, Vec<CreatureState>)> = m.regions.par_iter_mut().flat_map(|x| {
        x.par_iter_mut().flat_map(|y| {
            y.grid.par_iter_mut().flat_map(|xl| {
                xl.par_iter_mut().map(|yl| {
                    yl.creatures.drain_all_but_first_blocker(current_frame)
                })
            })
        })
    }).collect();
    let (blocked_blockers, mut blocked_nonblockers): (Vec<CreatureState>, Vec<CreatureState>) = blocked_creatures.into_par_iter().
    reduce(|| (Vec::new(), Vec::new()),|(mut tl1, mut tl2), (l1, l2)| {
        tl1.extend(l1.into_iter());
        tl2.extend(l2.into_iter());
        (tl1, tl2)
    });
    // TODO: go through blocking creatures list first and have them find nearest
    // non blocking location. then have them each return a new list of Creatures that are newly blocked
    // gonna be weird doing it in parallel based on region. will have to make a hashmap of
    // tuples of &mut Region : Vec<creatures>. then run on each (&mut Region, Vec)
    // probably better off for now just going through linearly
    let mut dead_list = Vec::new();

    // foreach blocked USE map_state.find_closest_non_blocked to find closest non blocked loc
    // if there are creatures in that unblocked location add them to blocked_nonblockers
    // then add the blocked creature to that loc
    blocked_blockers.into_iter().for_each(|c| {
        let loc = m.find_closest_non_blocked(Location::new(c.components.region_component.region, c.components.location_component.location), true);
        if let Some(open_loc) = loc {
            let map_loc: &mut MapLocation = &mut m.regions[open_loc.region.x as usize][open_loc.region.y as usize]
                .grid[open_loc.position.x as usize][open_loc.position.y as usize];
            map_loc.creatures.drain_creatures(m.frame_count).into_iter().for_each(|c_to_move| {
                println!("Moving bumped creature to non blocking blocker {}", c_to_move.get_id());
                blocked_nonblockers.push(c_to_move);
            });
            
            map_loc.creatures.add_creature(c, m.frame_count);
        }
        else {
            dead_list.push(c);
        }
    });

    // then go through blocked_nonblockers and find_closest_non_blocked
    blocked_nonblockers.into_iter().for_each(|c| { 
        let loc = m.find_closest_non_blocked(Location::new(c.components.region_component.region, c.components.location_component.location), false);
        if let Some(open_loc) = loc {
            let map_loc: &mut MapLocation = &mut m.regions[open_loc.region.x as usize][open_loc.region.y as usize]
                .grid[open_loc.position.x as usize][open_loc.position.y as usize];
            println!("Moving bumped creature to loc {} -> {:?}", c.get_id(), open_loc);
            map_loc.creatures.add_creature(c, m.frame_count);
        }
        else {
            println!("Moving bumped creature to dead {}", c.get_id());
            dead_list.push(c);
        }
    });
    // if no open spots to push creatures to for either of the above just kill them(add to death list?)
    // DECISION: Make it so just cannot have creatures that block exits ever. so region map will never change. FUCK BUT THIS DONT WORK!
    // Can have Below:
    // XOX
    // OXO  Here the sides are all open but you can't traverse. Many examples of this.
    // XOX
    // Need to make it so any location that would lead to blocking exits is blocked not just exits.
    
    // update nav map based on spawns for regions that spawn ones that block
    // basically just need to update the last frame changed for each region
    // then run some "update nav system" that checks every region and sees which ones have a last_frame_updated < last_frame(in MapRegion)

    // How the fuck do I know which regions need to be updated? Maybe make creatures private, and add function like "add_creature"?
    let changed_regions: Vec<Option<Vector2>> = m.regions.par_iter_mut().enumerate().flat_map(|(xidx, x)| {
        let row: Vec<Option<Vector2>> = x.par_iter_mut().enumerate().map(|(yidx, y)| {
            let last_changed_region = y.last_frame_changed;
            let changes: Vec<bool> = y.grid.par_iter_mut().flat_map(|xl| {
                xl.par_iter_mut().map(|yl| {
                    if yl.creatures.get_last_updated() > last_changed_region {
                        true
                    } else {
                        false
                    }
                })
            }).collect();

            if changes.contains(&true) {
                // for each changed region, update it's region's inner nav (implement update_region_nav)
                y.update_region_nav(current_frame);
                Some(Vector2::new(xidx as i32, yidx as i32))
            } else {
                None
            }
        }).collect();
        row
    }).collect();
    let changed_regions: Vec<Vector2> = changed_regions.into_par_iter().filter_map(|opt| opt).collect();

    // each region should have already been updated above.
    // now update the entire map's between region nav if we had any updated regions (TODO: Optimize this? Maybe don't need to update every single region but ones that update paths significantly?).
    // TODO: The region nav update isn't parallelized at all could be a bottleneck? If so this could also only be run every X frame. because its just inter-region nav which can be inaccurate a little since no region path can actually be blocked fully.
    if changed_regions.len() > 0 {
        m.update_nav();
    }

    // Can run immutable systems that rely on reading entire mapstate and need entire creature-list targets here
    let mov_op_ecs: Vec<Option<EventChain>> = m.regions.par_iter().flat_map(|x| {
        x.par_iter().flat_map(|y| {
            y.grid.par_iter().flat_map(|xl| {
                xl.par_iter().flat_map(|yl| {
                    if let Some(cit) = yl.creatures.get_par_iter() {
                        let ret: Vec<Option<EventChain>> = cit.flat_map(
                            |c| {
                                let mut ret: Vec<Option<EventChain>> = vec![];
                                ret.push(movement_system_move(&m, c));
                                ret.push(vision_system_add(&m, c));
                                ret
                            }
                        ).collect();
                        return ret;
                    } else {
                        Vec::new()
                    }
                })
            })
        })
    }).collect();
    let event_chains: Vec<EventChain> = unwrap_option_list(mov_op_ecs);
    process_events_from_mapstate(&mut m, event_chains);

    // Can run MUTABLE multiple systems here so far:
    // These only need the creature state that is being modified, no map state or state of other creatures
    // ex: Starvation, the part of movement that just increases counters, child growth, 
    m.regions.par_iter_mut().for_each(|x| {
        x.par_iter_mut().for_each(|y| {
            let region_loc = y.location;
            y.grid.par_iter_mut().for_each(|xl| {
                xl.par_iter_mut().for_each(|yl| {
                    let position = yl.location;
                    let location = Location::new(region_loc, position);
                    if let Some(cit) = yl.creatures.get_par_iter_mut() {
                        cit.for_each(
                            |c| {
                                child_growth_system(c, current_frame);
                                starvation_system(c, current_frame);
                                movement_system_iterate(current_frame, c, location);
                            }
                        );
                    }
                })
            })
        })
    });


    // TODO: Send out current map state to users via websocket
    // TODO: Then wait for them to respond if doing by-frame, or a timer
    // TODO: Actually probably move the websocket stuff and the ai stuff to the beginning of this function?

    // want to move THEN AFTER do ai stuff so ai can react to the movement
    let mut op_ecs: Vec<Option<EventChain>> = m.regions.par_iter().flat_map(|x| {
        x.par_iter().flat_map(|y| {
            y.grid.par_iter().flat_map(|xl| {
                xl.par_iter().flat_map(|yl| {
                    if let Some(cit) = yl.creatures.get_par_iter() {
                        
                        let ret: Vec<Option<EventChain>> = cit.map(
                            |c| {
                                // TODOREVAMP: Comment out below and instead use evolutionary growing ai approach? Actually just combine this with evo approach.
                                // So that we can just do both. Because old approach is great for running tests so good to keep. maybe put in a bool
                                match GoalCacheNode::get_final_command(&root, &m, &c) {
                                    Some(cc) => {cc.to_event_chain()}
                                    None => {None}
                                }
                            }
                        ).collect();
                        return ret;
                    } else {
                        Vec::new()
                    }
                })
            })
        })
    }).collect();


    // Attack creature command will produce EventChain that will set creatures to battle and have a new target BattleList.
    // The last BattleList event will add the Battle to the list of Battles.
    // Battle is new datatype that will have 2 BattlerInfo for the creatures battling.
    // Each battle is updated once per frame. Does whatever.
    // Battle.update returns Option<BattleResults>.
    // BattleResults List is then created (and maybe dict that points to it for quick ref?). Read only.
    // Then in the mutable system iteration do a:
    // If creature_id in BattleResults list -> call leave_battle_system that takes in an immutable ref to the battle. and updates the creature based on it.
    // may update HP, items, etc.

    // BATTLE SYSTEM RUNS HERE so that can process their events at same time as other stuff so faster
    let mut battle_effects: Vec<Option<EventChain>> = m.battle_list.battles.par_iter_mut().map( |battle| {
        battle.update()
    }).collect();
    op_ecs.append(&mut battle_effects);

    let event_chains = unwrap_option_list(op_ecs);
    process_events_from_mapstate(&mut m, event_chains);

    // Death system
    // TODO: Update nav system if blockers died
    // TODO: also have a death list or something for creatures that dont have health but still died prob just go through them linearly?
    let no_hp_list:Vec<CreatureState> = m.regions.par_iter_mut().flat_map(|x| {
        x.par_iter_mut().flat_map(|y| {
            y.grid.par_iter_mut().flat_map(|xl| {
                xl.par_iter_mut().flat_map(|yl| {
                    if yl.creatures.holds_creatures() {
                        yl.creatures.drain_no_health(current_frame)
                    } else {
                        vec![]
                    }
                })
            })
        })
    }).collect();
    dead_list.extend(no_hp_list);

    // Do stuff with the drained creatures.
    // put items the creature has to the locations they died.
    // put items if they have death_items component type thing
    // death rattles done are here?
    let dead_events: Vec<Option<EventChain>> = dead_list.into_par_iter().flat_map(|dead| {
        //get items to drop from dead_items
        let mut items = match dead.components.death_items_component {
            Some(dead_items) => {dead_items.items_to_drop}
            None => {vec![]}
        };
        // get items to drop that are in inventory
        items.extend(dead.inventory);

        let target = m.regions[dead.components.region_component.region].grid[dead.components.location_component.location].id_component_items.id();

        // create event to add items
        let events: Vec<Event> = items.into_par_iter().map(|item| {
            Event {
                event_type: EventType::AddItem(item.quantity, item.item_type),
                get_requirements: Box::new(|_,_| true),
                on_fail: None,
                target,
            }
        }).collect();
        // Can do other death stuff here like explosion when die etc
        if events.len() > 0 {
            vec![Some(EventChain {
                events,
                debug_string: format!("Dead {}", target),
                creature_list_targets: true,

            })]
        } else {
            vec![]
        }
        
    }).collect();
    process_events_from_mapstate(&mut m, unwrap_option_list(dead_events));

    GameState {
        map_state: m,
    }
}

pub fn create_basic_map_state() -> MapState {
        // make a mapstate with some budders
        let openr = RegionCreationStruct::new(10,10, 0, vec![]);
        let rgrid = vec![
            vec![openr.clone()],
        ];
        //create map
        let mut map = MapState::new(rgrid, 0);
        let  region: &mut MapRegion = &mut map.regions[0][0];
    
        let mut grass = CreatureState{
            components: ComponentMap::default(),
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        };
        grass.components.region_component = RegionComponent {
            region: Vu2{x: 0, y: 0},
        };
        grass.components.location_component = LocationComponent {
            location: Vu2{x: 1, y: 1}
        };
        grass.components.health_component = Some(HealthComponent {
            health:  1,
            max_health: 1,
        });
        grass.components.budding_component = Some(BuddingComponent {
            reproduction_rate: 3,
            frame_ready_to_reproduce: 3,
            seed_creature_differences: Box::new(ComponentMap::fake_default()),
        });
        grass.components.soil_component = Some(SoilComponent{
            soil_height: SoilHeight::Grass,
            ..Default::default()
        });
        // Just to make sure the grass doesn't replicate with the inventory
        grass.inventory.push(Item{
            item_type: ItemType::Berry,
            quantity: 1,
        });
    
        let mut flower = CreatureState{
            components: ComponentMap::default(),
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        };
        flower.components.region_component = RegionComponent {
            region: Vu2{x: 0, y: 0},
        };
        flower.components.location_component = LocationComponent {
            location: Vu2{x: 8, y: 1}
        };
        flower.components.health_component = Some(HealthComponent {
            health:  1,
            max_health: 1,
        });
        flower.components.budding_component = Some(BuddingComponent {
            reproduction_rate: 3,
            frame_ready_to_reproduce: 3,
            seed_creature_differences: Box::new(ComponentMap::fake_default()),
        });
        flower.components.soil_component = Some(SoilComponent{
            soil_height: SoilHeight::Flower,
            ..Default::default()
        });
    
        let mut bush = CreatureState{
            components: ComponentMap::default(),
            inventory: Vec::new(),
            memory: CreatureMemory::default(),
        };
        bush.components.region_component = RegionComponent {
            region: Vu2{x: 0, y: 0},
        };
        bush.components.location_component = LocationComponent {
            location: Vu2{x: 5, y: 8}
        };
        bush.components.health_component = Some(HealthComponent {
            health:  1,
            max_health: 1,
        });
        bush.components.budding_component = Some(BuddingComponent {
            reproduction_rate: 3,
            frame_ready_to_reproduce: 3,
            seed_creature_differences: Box::new(ComponentMap::fake_default()),
        });
        bush.components.soil_component = Some(SoilComponent{
            soil_height: SoilHeight::Bush,
            ..Default::default()
        });
        
        region.grid[grass.components.location_component.location].creatures.add_creature(
            grass, 0
        );
        region.grid[flower.components.location_component.location].creatures.add_creature(
            flower, 0
        );
        region.grid[bush.components.location_component.location].creatures.add_creature(
            bush, 0
        );
    
        return map;
}

#[cfg(test)]
mod tests {
    use crate::*;
    // use std::{cell::{RefCell}, rc::Rc};
    // use std::collections::HashMap;
    
    // extern crate rayon;
    // use rayon::prelude::*;

    use std::{cell::{Ref, RefCell}, rc::Rc};
    use std::collections::HashMap;
    use std::ops::Deref;
    use std::{fmt::{Debug, Formatter}, borrow::Borrow};
    use std::sync::atomic::AtomicU64;
    use core::fmt;

}
