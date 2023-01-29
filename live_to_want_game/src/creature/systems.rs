use rand::Rng;

use crate::{EventTarget, Location, Vu2, map_state::MapState, tasks::Event, tasks::EventChain, tasks::EventType, MOVING_INCREASED_METABOLISM_FACTOR, EvolvingTraitsComponent, EvolvingTraits, REPRODUCE_STARTING_CALORIES_MULTIPLIER, STANDARD_METABOLISM, STANDARD_PREGNANCY_LIVE_WEIGHT, THICK_HIDE_METABOLISM_MULTIPLIER, FAST_GROWER_CALORIE_MULTIPLIER, MOVE_SPEED_METABOLISM_MULTIPLIER, STANDARD_PREGNANCY_METABOLISM_MULTIPLIER, LITTER_SIZE_METABOLISM_MULTIPLIER, UID, SoilHeight};

use super::{CreatureState, STARVING_SLOW_METABOLISM_FACTOR};

pub fn soil_spread_system(m: &MapState, c: &CreatureState) -> Vec<EventChain> {
    if !c.get_if_in_combat() {
        if let Some(soil_c) = c.components.soil_component.as_ref() {
            if soil_c.spread_rate.is_some() && soil_c.frame_ready_to_spread <= m.frame_count {
                // find open spot first
                let mut open_spots = Vec::new();
                let location = c.components.location_component.location;
                let region = c.components.region_component.region;
                let map_region = &m.regions[region.x as usize][region.y as usize];

                let soil_type = soil_c.soil_type_spread;
                //println!("{:#?}, {:#?}", location, location.get_valid_neighbors(map_region.grid.len(), map_region.grid[0].len()));
                for n in location.get_valid_neighbors(map_region.grid.len(), map_region.grid[0].len()) {
                    // SoilHeight All should make it mean if ANY creature with soil
                    // is there, then it won't spread
                    if map_region.grid[n.get()].get_soil_type() != soil_type && 
                        map_region.grid[n.get()].get_if_creature_open_and_soil_open(false, Some(SoilHeight::All), None) {
                        open_spots.push(n.get());
                    }
                }

                let spots = open_spots.len();
                // Reset spread so it doesnt try again every frame, but next time it would spread
                let spread_iterate = Event {
                    event_type: EventType::IterateSoilSpread(),
                    target: c.components.id_component.id(),
                    on_fail: None,
                    get_requirements: Box::new(|_,_| true),
                };
                let mut event_chains = vec![EventChain{
                    events: vec![spread_iterate],
                    debug_string: format!("Reset Budding {}", c.components.id_component.id()),
                    creature_list_targets: false,
                }];
                
                if spots > 0 {
                    let mut rng = rand::thread_rng();
                    let chosen = rng.gen_range(0, spots);
                    let loc = open_spots[chosen];
                    let target_location = map_region.grid[loc.x as usize][loc.y as usize].id_component_creatures.id();
                    let create_event = Event {
                        event_type: EventType::ChangeSoil(soil_type),
                        target: target_location,
                        on_fail: None,
                        get_requirements: Box::new(|e_target, ev_type| {
                            match e_target {
                                EventTarget::LocationCreaturesTarget(cl, uid) => {
                                    return cl.get_if_open_and_open_soil(Some(SoilHeight::All));
                                }
                                _ => {
                                    panic!("Got eventtarget that isnt for add creature")
                                }
                            }
                        }),
                    };
                    event_chains.push(EventChain{
                        events: vec![create_event],
                        debug_string: format!("Spread soil {} to creature list {} loc {:?}", c.components.id_component.id(), target_location, loc),
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
}

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

                // new soil budding algo.
                // Check for SoilHeight if soil Height is open (only 2 soil heights
                // can exist on a square, or 1 All). Then check if the soilType matches.
                let soil_height = c.components.soil_component.map_or(None, |s| Some(s.soil_height));
                let soil_type_invalid = c.components.soil_component.map_or(None, |s| Some(s.soil_type_cannot_grow));
                // Also this will spawn creatures over and over on the same spots if they are not blocking
                for n in location.get_valid_neighbors(map_region.grid.len(), map_region.grid[0].len()) {
                    if map_region.grid[n.get()].get_if_creature_open_and_soil_open(blocker, soil_height, soil_type_invalid) {
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
                                        return cl.get_if_open_and_open_soil(c.components.soil_component.map_or(None, |sc| Some(sc.soil_height)))
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


pub fn starvation_system(c: &mut CreatureState, frame: u128) {
    let is_child = c.get_if_child(frame);
    let adult_percent = c.get_adult_percent(frame);
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
                // TODONEXT: integrate move_speed, fast_grower, thick_hide, is_pregnant and litter_size
                let is_pregnant = if let Some(sex) = c.components.sexual_reproduction.as_ref() {
                    sex.is_pregnant
                } else {
                    false
                };
                
                if is_pregnant {
                    multiplier *= STANDARD_PREGNANCY_METABOLISM_MULTIPLIER;
                    multiplier *= 1. + (LITTER_SIZE_METABOLISM_MULTIPLIER * traits.traits.litter_size as f32);
                }

                if is_child {
                    multiplier *= 1. + (traits.traits.fast_grower as f32 * FAST_GROWER_CALORIE_MULTIPLIER);
                    // Children need less calories than adults by the percent they are adults.
                    multiplier *= adult_percent;
                }

                multiplier *= 1. + (traits.traits.thick_hide as f32 * THICK_HIDE_METABOLISM_MULTIPLIER);

                if is_moving {
                    multiplier *= MOVING_INCREASED_METABOLISM_FACTOR;
                    multiplier *= traits.traits.move_speed as f32 * MOVE_SPEED_METABOLISM_MULTIPLIER;
                }
            } else if is_moving {
                multiplier *= MOVING_INCREASED_METABOLISM_FACTOR;
                // Pregnant?
            }
            s.calories -= (s.metabolism as f32 * multiplier) as i32;

        } else {
            panic!("All starvation components require health component for: {}", c)
        }
    }
}

pub fn child_growth_system(c: &mut CreatureState, frame: u128) {
    if let Some(_) = c.components.evolving_traits.as_mut() {
        c.setup_creature(frame, false);
    }
}

pub fn sex_reproduction_system(m: &MapState, c: &CreatureState) -> Vec<EventChain> {
    // check if pregnant, if so, check if ready to pop, if so, pop kids out as event chain after figuring out their stats
    if let Some(s) = c.components.sexual_reproduction.as_ref() {
        if s.is_pregnant {
            //println!("Checking if ready to pop out {} frame: {} goal: {}", c.get_id(), m.frame_count, s.pregnancy_completion_frame);
            if s.pregnancy_completion_frame == m.frame_count {
                // NOTE assumes this is not a blocker!
                assert!(c.components.block_space_component.is_none());

                let location = c.components.location_component.location;
                let region = c.components.region_component.region;
                let map_region = &m.regions[region.x as usize][region.y as usize];
                let target_location = map_region.grid[location.x as usize][location.y as usize].id_component_creatures.id();
                let mother_pregnany_time = c.components.evolving_traits.as_ref().unwrap().get_pregnancy_length();
                let mut make_newborns: Vec<Event> = vec![];

                (0..s.litter_size).for_each(|_| {
                    let mut new_creature = CreatureState::clone_to_new_location(c, location);
                    new_creature.components.evolving_traits = Some(EvolvingTraitsComponent {
                        adult_traits: c.components.evolving_traits.as_ref().unwrap().traits.clone_with_mate(&s.partner_genes),
                        traits: EvolvingTraits::default(), // need to base this off of the childness and stuff
                        child_until_frame: 0, // Based on current frame and pregnancy time of mother, as well as growth rate of child
                        born_on_frame: m.frame_count,
                    }.get_mutated(1));
                    new_creature.components.evolving_traits.as_mut().unwrap().child_until_frame = m.frame_count + new_creature.get_child_length(mother_pregnany_time);
                    
                    // Setup childness traits of the new_creature
                    // also setup creatures initial stuff based on traits, health for example.
                    // prob need to also zero out things like creature memory for the child?
                    new_creature.setup_creature(m.frame_count, true);

                    if let Some(starvation) = new_creature.components.starvation_component.as_mut() {
                        starvation.calories = (STANDARD_METABOLISM as f32 * REPRODUCE_STARTING_CALORIES_MULTIPLIER as f32 * c.components.evolving_traits.as_ref().unwrap().get_newborn_starting_calories_multiplier()) as i32;
                    }

                    let create_event = Event {
                        event_type: EventType::AddCreature(new_creature, m.frame_count),
                        target: target_location,
                        on_fail: None,
                        // TODO: Not sure if its okay to spawn creature on a blocked tile? Will it move the blocked creature auto? I think so right?
                        get_requirements: Box::new(|_,_|  true),
                    };
                    make_newborns.push(create_event);
                });

                let mut iterate_events = vec![Event { 
                    event_type: EventType::ResetSexReproduction(), 
                    on_fail: None,
                    get_requirements: Box::new(|_,_|  true),
                    target: c.get_id(),
                }];

                let death_weight = c.components.evolving_traits.as_ref().unwrap().get_weight_of_childbirth_death();
                let live_weight = STANDARD_PREGNANCY_LIVE_WEIGHT;
                let mut rng = rand::thread_rng();
                let chosen = rng.gen_range(0, live_weight);
                if chosen < death_weight {
                    iterate_events.push(Event { 
                        event_type: EventType::SetHealth(0),
                        on_fail: None,
                        get_requirements: Box::new(|_,_|  true),
                        target: c.get_id(),
                    });
                }

                return vec![
                    EventChain {
                        events: make_newborns,
                        debug_string: format!("Birthing {}", c.components.id_component.id()),
                        creature_list_targets: true
                    },
                    EventChain {
                        events: iterate_events,
                        debug_string: format!("Birthing Iterate {}", c.components.id_component.id()),
                        creature_list_targets: false
                    },
                ];
            }
        }
    }
    vec![]
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

pub fn vision_system_add(m: &MapState, c: &CreatureState) -> Option<EventChain> {
    if let Some(_) = c.components.vision_component.as_ref() {
        let ids: Vec<UID> = m.find_creatures_in_range_to_creature(c, c.get_vision_range()).iter().map(|c| c.get_id()).collect();

        let mut events: Vec<Event> = ids.iter().map(|id| Event{
                event_type: EventType::AddVisible(*id),
                get_requirements: Box::new(|_,_| true),
                on_fail: None,
                target: c.get_id(),
        }).collect();
        events.insert(0, Event{
            event_type: EventType::ClearVisible(),
            get_requirements: Box::new(|_,_| true),
            on_fail: None,
            target: c.get_id(),
        });
        return Some(EventChain {
            events: events,
            debug_string: format!("vision update for {}", c.components.id_component.id()),
            creature_list_targets: false
        });
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
