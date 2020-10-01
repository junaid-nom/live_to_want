

use std::cmp::Ordering;
use std::{cell::{Ref, RefCell}, rc::Rc};
use std::collections::HashMap;
use std::ops::Deref;
use std::{fmt::{Debug, Formatter}, borrow::Borrow};
use std::sync::{Arc, atomic::AtomicU64};
use core::fmt;
use rand::prelude::*;

extern crate rayon;
use rayon::prelude::*;

mod map_state;
pub use map_state::*;

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

// TODO: 
// Pretty sure items in a MapLocation and inventory in a creature state don't have to be rc<refcell<>>


pub struct GameState {
    pub navigation_map: NavigationMap,
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

pub fn game() {
    // Make initial map state
    
    // generate initial goal root

    // start server

    // loop
    // get input from connections
    // run frame
    // if in super-fast mode, just loop
    // if in user controlled just check for input until receive something
    // also can do "slow" mode with a wait
}

fn run_frame(mut game_state: GameState, root: &GoalNode) -> GameState {
    let mut m = game_state.map_state;
    let mut nav_map = game_state.navigation_map;
    {
        m.frame_count += 1;
    }

    // TODO: Run spawn systems first, and spawn new creatures
    let spawn_events: Vec<Option<EventChain>> = m.regions.par_iter().flat_map(|x| {
        x.par_iter().flat_map(|y| {
            y.grid.par_iter().flat_map(|xl| {
                xl.par_iter().flat_map(|yl| {
                    if let Some(creatures) = yl.creatures.as_ref() {
                        let ret: Vec<Option<EventChain>> = creatures.par_iter().map(
                            |c| {
                                budding_system(&m, c)
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
    let mut event_chains: Vec<EventChain> = unwrap_option_list(spawn_events);

    process_events_from_mapstate(&mut m, event_chains);

    // TODO: Deal with blockers that spawned
    // first vec is BLOCKERS second vec is NON BLOCKERS
    let blocked_creatures: Vec<(Vec<CreatureState>, Vec<CreatureState>)> = m.regions.par_iter_mut().flat_map(|x| {
        x.par_iter_mut().flat_map(|y| {
            y.grid.par_iter_mut().flat_map(|xl| {
                xl.par_iter_mut().map(|yl| {
                    if let Some(creatures) = yl.creatures.as_mut() {
                        let mut ret: (Vec<CreatureState>, Vec<CreatureState>) = (Vec::new(), Vec::new());
                        // if there is a blocking creature and any other creature here
                        // then we have to remove them
                        let mut first_blocker: Option<UID> = None;
                        for i in 0..creatures.len() {
                            let c = &creatures[i];
                            if let Some(_) = c.components.block_space_component {
                                first_blocker = Some(c.components.id_component.id());
                                break;
                            }
                        };
                        if let Some(first) = first_blocker {
                            for i in 0..creatures.len() {
                                if i < creatures.len() {
                                    let c = &creatures[i];
                                    if c.components.id_component.id() != first {
                                        if let Some(_) = c.components.block_space_component {
                                            ret.0.push(creatures.remove(i));
                                        } else {
                                            ret.1.push(creatures.remove(i));
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        return ret;
                    } else {
                        (Vec::new(), Vec::new())
                    }
                })
            })
        })
    }).collect();
    let (blocked_blockers, blocked_nonblockers): (Vec<CreatureState>, Vec<CreatureState>) = blocked_creatures.into_par_iter().
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
    
    //TODONEXT: foreach blocked USE map_state.find_closest_non_blocked to find closest non blocked loc
    // if there are creatures in that unblocked location add them to blocked_nonblockers
    // then add the blocked creature to that loc

    // then go through blocked_nonblockers and find_closest_non_blocked
    
    // if none for either of the above just kill them(add to death list?)



    // TODO: update nav map based on spawns for regions that spawn ones that block

    // Can run MUTABLE multiple systems here so far:
    // Starvation system
    // nav system
    m.regions.par_iter_mut().for_each(|x| {
        x.par_iter_mut().for_each(|y| {
            y.grid.par_iter_mut().for_each(|xl| {
                xl.par_iter_mut().for_each(|yl| {
                    if let Some(creatures) = yl.creatures.as_mut() {
                        creatures.par_iter_mut().for_each(
                            |c| {
                                starvation_system(c);
                                navigation_system(c);
                            }
                        );
                    }
                })
            })
        })
    });

    // Can run immutable systems that rely on reading entire mapstate here
    let mov_op_ecs: Vec<Option<EventChain>> = m.regions.par_iter().flat_map(|x| {
        x.par_iter().flat_map(|y| {
            y.grid.par_iter().flat_map(|xl| {
                xl.par_iter().flat_map(|yl| {
                    if let Some(creatures) = yl.creatures.as_ref() {
                        let ret: Vec<Option<EventChain>> = creatures.par_iter().map(
                            |c| {
                                movement_system(&m, c)
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
    let mut event_chains: Vec<EventChain> = unwrap_option_list(mov_op_ecs);
    process_events_from_mapstate(&mut m, event_chains);

    // TODO: Send out current map state to users via websocket
    // TODO: Then wait for them to respond if doing by-frame, or a timer
    // TODO: Actually probably move the websocket stuff and the ai stuff to the beginning of this function?

    // want to move THEN AFTER do ai stuff so ai can react to the movement
    let op_ecs: Vec<Option<EventChain>> = m.regions.par_iter().flat_map(|x| {
        x.par_iter().flat_map(|y| {
            y.grid.par_iter().flat_map(|xl| {
                xl.par_iter().flat_map(|yl| {
                    if let Some(creatures) = yl.creatures.as_ref() {
                        let ret: Vec<Option<EventChain>> = creatures.par_iter().map(
                            |c| {
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
    let mut event_chains = unwrap_option_list(op_ecs);
    process_events_from_mapstate(&mut m, event_chains);

    // Death system
    // TODO: Update nav system if blockers died
    // TODO: also have a death list or something for creatures that dont have health but still died prob just go through the linearly?
    // TODO: Add some kind of death_rattle event chain system, will need to do things like add items to ground etc
    m.regions.par_iter_mut().for_each(|x| {
        x.par_iter_mut().for_each(|y| {
            y.grid.par_iter_mut().for_each(|xl| {
                xl.par_iter_mut().for_each(|yl| {
                    if let Some(creatures) = yl.creatures.as_mut() {
                        // TODO: IF CREATURES WERE BLOCKERS, need to set that nav region dirty
                        creatures.retain(
                            |c| {
                                if let Some(h) = c.components.health_component.as_ref() {
                                    if h.health <= 0 {
                                        false
                                    } else {
                                        true
                                    }
                                } else {
                                    true
                                }
                            }
                        );
                    }
                })
            })
        })
    });

    GameState {
        map_state: m,
        navigation_map: nav_map,
    }
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
