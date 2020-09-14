

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
use map_state::*;

mod utils;
use utils::*;

mod creature;
use creature::*;

mod tasks;
use tasks::*;

mod ai;
use ai::*;

// NOTE: All event chains with items need to end in a final failure case of putting item on ground.
// this is because you can try to give an item away as someone else fills your inventory and 
// if both giving fails and putting back into your inventory fails, need to put item somewhere, so put on ground.

// TODO: 
// Pretty sure items in a MapLocation and inventory in a creature state don't have to be rc<refcell<>>


pub struct GameState {
    pub navigation_map: NavigationMap,
    pub map_state: MapState,
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
    let f_c = m.frame_count;
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

    // TODO: Send out current map state to users via websocket
    // TODO: Then wait for them to respond if doing by-frame, or a timer

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
    let mut event_chains = Vec::new();
    for o in op_ecs {
        if let Some(ec) = o {
            event_chains.push(ec);
        }
    }

    process_events_from_mapstate(&mut m, event_chains);

    // Death system
    m.regions.par_iter_mut().for_each(|x| {
        x.par_iter_mut().for_each(|y| {
            y.grid.par_iter_mut().for_each(|xl| {
                xl.par_iter_mut().for_each(|yl| {
                    if let Some(creatures) = yl.creatures.as_mut() {
                        // IF CREATURES WERE BLOCKERS, need to set that nav region
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
