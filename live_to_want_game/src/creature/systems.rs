use crate::{map_state::MapState, tasks::EventChain, map_state::location_to_map_location, tasks::Event, tasks::EventType};

use super::{CreatureState, STARVING_SLOW_METABOLISM_FACTOR};

pub fn budding_system(m: &MapState, c: &CreatureState) -> Option<EventChain> {
    // first check if its the frame to reproduce.
    // If so, find a random nearby open spot. that is not blocked
    // make an event chain of first placing the creature on that space. (create unit)
    // then next event resets the counters for the creatures budding component

    // Then later in the main loop, run a function that looks at all spots and checks for
    // more than 1 blocker. if that's the case, remove all the "excess" units and put them in a new list, (their location should mark where they previously were).
    // one list for blockers one for not.
    // then linearly go through and for each one find a nearby open space and add them there?
    // actually can make it parallelized by doing it based on region?
    // do the blockers first?
    // for each blocker it might also remove additional non-blocker creatures
    // blocking units have to be done first actually
    // should be fast since shouldnt happen often?...
    None
}

pub fn starvation_system(c: &mut CreatureState) {
    if let Some(s) = c.components.starvation_component.as_mut() {
        if let Some(h) = c.components.health_component.as_mut() {
            let starving = s.calories <= 0;
            if starving {
                h.health -= 1;
            }
            let multiplier = if starving {STARVING_SLOW_METABOLISM_FACTOR} else {1.0};
            s.calories -= (s.metabolism as f32 * multiplier) as i32;
        } else {
            panic!("All starvation components require health component for: {}", c)
        }
    }
}


pub fn navigation_system(c: &mut CreatureState) {
    // TODO if the target for the creature is currently blocked, fuck
}

pub fn movement_system(m: &MapState, c: &CreatureState) -> Option<EventChain> {
    if let Some(movement) = c.components.movement_component.as_ref() {
        if movement.frame_ready_to_move <= m.frame_count {
            let dest = location_to_map_location(m, &movement.destination).id_component_creatures.id();
            let rm_event = Event {
                event_type: EventType::RemoveCreature(c.components.id_component.id(), 
                    Some(dest)),
                get_requirements: Box::new(|_,_| true),
                on_fail: None,
                target: dest,
            };
        }
    }
    None
}

