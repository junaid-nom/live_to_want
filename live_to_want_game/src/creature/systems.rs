use rand::Rng;

use crate::{EventTarget, Location, Vu2, map_state::MapState, tasks::Event, tasks::EventChain, tasks::EventType, MOVING_INCREASED_METABOLISM_FACTOR, EvolvingTraitsComponent, EvolvingTraits};

use super::{CreatureState, STARVING_SLOW_METABOLISM_FACTOR};

// This will now work even though some of the events will target creature_list and some won't! Need to split them up.
// Works by process_events_from_mapstate now having splitting events into two types
pub fn budding_system(m: &MapState, c: &CreatureState) -> Vec<EventChain> {
    // first check if its the frame to reproduce.
    // If so, find a random nearby open spot. that is not blocked
    // make an event chain of first placing the creature on that space. (create unit)
    // then next event resets the counters for the creatures budding component
    if !c.get_if_in_combat() {
        if let Some(bud) = c.components.budding_component.as_ref() {
            if bud.frame_ready_to_reproduce <= m.frame_count {
                // find open spot first
                let mut open_spots = Vec::new();
                let location = c.components.location_component.location;
                let region = c.components.region_component.region;
                let map_region = &m.regions[region.x as usize][region.y as usize];
                let blocker = c.components.block_space_component.is_some();
                let soil = c.components.soil_component.map_or(None, |s| Some(s.soil_layer));
                // Also this will spawn creatures over and over on the same spots if they are not blocking
                for n in location.get_valid_neighbors(map_region.grid.len(), map_region.grid[0].len()) {
                    if map_region.grid[n.get()].get_if_creature_open_and_soil_open(blocker, soil) {
                        open_spots.push(n.get());
                    }
                }

                let spots = open_spots.len();
                // Reset budding so it doesnt try again every frame, but next time it would reproduce
                let bud_iterate = Event {
                    event_type: EventType::IterateBudding(),
                    target: c.components.id_component.id(),
                    on_fail: None,
                    get_requirements: Box::new(|_,_| true),
                };
                let mut event_chains = vec![EventChain{
                    events: vec![bud_iterate],
                    debug_string: format!("Reset Budding {}", c.components.id_component.id()),
                    creature_list_targets: false,
                }];
                
                if spots > 0 {
                    let mut rng = rand::thread_rng();
                    let chosen = rng.gen_range(0, spots);
                    let loc = open_spots[chosen];
                    let mut new_creature = CreatureState::clone_to_new_location(c, loc);
                    new_creature.components = new_creature.components.copy_from_other(&bud.seed_creature_differences);
                    let target_location = map_region.grid[loc.x as usize][loc.y as usize].id_component_creatures.id();
                    let create_event = Event {
                        event_type: EventType::AddCreature(new_creature, m.frame_count),
                        target: target_location,
                        on_fail: None,
                        get_requirements: Box::new(|e_target, ev_type| {
                            if let EventType::AddCreature(c, clist) = ev_type {
                                match e_target {
                                    EventTarget::LocationCreaturesTarget(cl, uid) => {
                                        return cl.get_if_open_and_open_soil(c.components.soil_component.map_or(None, |sc| Some(sc.soil_layer)))
                                    }
                                    _ => {
                                        panic!("Got eventtarget that isnt for add creature")
                                    }
                                }
                            }
                            false
                        }),
                    };
                    event_chains.push(EventChain{
                        events: vec![create_event],
                        debug_string: format!("Bud {} to creature list {} loc {:?}", c.components.id_component.id(), target_location, loc),
                        creature_list_targets: true,
                    });
                }
                event_chains
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    } else {
        vec![]
    }
    // Then later in the main loop, run a function that looks at all spots and checks for
    // more than 1 blocker. if that's the case, remove all the "excess" units and put them in a new list, (their location should mark where they previously were).
    // one list for blockers one for not.
    // then linearly go through and for each one find a nearby open space and add them there?
    // actually can make it parallelized by doing it based on region?
    // do the blockers first?
    // for each blocker it might also remove additional non-blocker creatures
    // blocking units have to be done first actually
    // should be fast since shouldnt happen often?...
    
}

pub fn starvation_system(c: &mut CreatureState) {
    if let Some(s) = c.components.starvation_component.as_mut() {
        if let Some(h) = c.components.health_component.as_mut() {
            let starving = s.calories <= 0;
            if starving {
                h.health -= 1;
            }
            let mut multiplier = if starving {STARVING_SLOW_METABOLISM_FACTOR} else {1.0};
            let mut is_moving = false;
            if let Some(movement) = c.components.movement_component.as_ref() {
                is_moving = movement.moving;
            }
            // If you have evo traits, use that to determine penalty for moving, otherwise use the flat constant
            if let Some(traits) = c.components.evolving_traits.as_ref() {
                multiplier *= traits.get_total_metabolism_multiplier(is_moving);
            } else if is_moving {
                multiplier *= MOVING_INCREASED_METABOLISM_FACTOR;
            }
            s.calories -= (s.metabolism as f32 * multiplier) as i32;
        } else {
            panic!("All starvation components require health component for: {}", c)
        }
    }
}

pub fn reproduction_system(m: &MapState, c: &mut CreatureState) -> Option<EventChain> {
    // check if pregnant, if so, check if ready to pop, if so, pop kids out as event chain after figuring out their stats
    if let Some(mut s) = c.components.sexual_reproduction {
        if s.is_pregnant {
            if s.pregnancy_completion_frame == m.frame_count {
                s.is_pregnant = false;
                // NOTE assumes this is not a blocker!
                assert!(c.components.block_space_component.is_none());

                let location = c.components.location_component.location;
                let region = c.components.region_component.region;
                let map_region = &m.regions[region.x as usize][region.y as usize];
                let target_location = map_region.grid[location.x as usize][location.y as usize].id_component_creatures.id();

                let mut make_newborns: Vec<Event> = vec![];

                (0..s.litter_size).for_each(|_| {
                    let mut new_creature = c.clone();
                    new_creature.components.evolving_traits = Some(EvolvingTraitsComponent {
                        adult_traits: c.components.evolving_traits.unwrap().traits.clone_with_mate(&s.partner_genes),
                        traits: EvolvingTraits::default(), // need to base this off of the childness and stuff
                        child_until_frame: m.frame_count, // TODONEXT: Based on current frame and pregnancy time of mother, as well as growth rate of child
                        born_on_frame: m.frame_count,
                    });
                    // TODONEXT: Setup childness traits of the new_creature
                    // also setup creatures initial stuff based on traits, health for example.
                    // prob need to also zero out things like creature memory for the child?

                    let create_event = Event {
                        event_type: EventType::AddCreature(new_creature, m.frame_count),
                        target: target_location,
                        on_fail: None,
                        // TODO: Not sure if its okay to spawn creature on a blocked tile? Will it move the blocked creature auto? I think so right?
                        get_requirements: Box::new(|_,_|  true),
                    };
                    make_newborns.push(create_event);
                });

                // TODONEXT: return event chain
            }
        }
    }
    None
}


pub fn movement_system_move(m: &MapState, c: &CreatureState) -> Option<EventChain> {
    if !c.get_if_in_combat() {
        if let Some(movement) = c.components.movement_component.as_ref() {
            if movement.moving && movement.frame_ready_to_move <= m.frame_count {
                let next_move = m.navigate_to(&c.get_location(), &movement.destination);
                println!("moving from: {:?} to {:?}", c.get_location(), next_move);
                let src = m.location_to_map_location(&c.get_location()).id_component_creatures.id();
                let dest = m.location_to_map_location(&next_move).id_component_creatures.id();
                println!("creating movement src id: {} , creature loc: {:?} dst loc: {:?} dst id: {}", src, c.get_location(), next_move, dest);
                let rm_event = Event {
                    event_type: EventType::RemoveCreature(c.components.id_component.id(), 
                        Some(dest), m.frame_count),
                    get_requirements: Box::new(|_,_| true),
                    on_fail: None,
                    target: src,
                };
                // let iter_move = Event {
                //     event_type: EventType::IterateMovement(m.frame_count),
                //     get_requirements: Box::new(|_,_| true),
                //     on_fail: None,
                //     target: c.components.id_component.id(),
                // };
                return Some(EventChain {
                    events: vec![rm_event],
                    debug_string: format!("Moving {}", c.components.id_component.id()),
                    creature_list_targets: true
                });
            }
        }
    }
    
    None
}

pub fn movement_system_iterate(current_frame: u128, c: &mut CreatureState, current_location: Location) {
    // actually update there position!
    // TODO maybe only needs to change after movement system changed position, so can put inside the if above for speed?
    c.components.region_component.region = current_location.region;
    c.components.location_component.location = current_location.position;
    let in_combat = c.get_if_in_combat();
    if let Some(movement) = c.components.movement_component.as_mut() {
        // reset moving if in combat
        if in_combat {
            movement.moving = false;
        }
        if movement.moving && movement.frame_ready_to_move <= current_frame {
            let dst_reached =  c.components.location_component.location == movement.destination.position &&
            c.components.region_component.region == movement.destination.region;
            movement.check_ready_and_reset_move(current_frame, dst_reached);
        }
    }
}
