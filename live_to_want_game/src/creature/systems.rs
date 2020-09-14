
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
            let dest = location_to_map_location(m, &movement.destination).id_component_creatures.id;
            let rm_event = Event {
                event_type: EventType::RemoveCreature(c.components.id_component.id, 
                    Some(dest)),
                get_requirements: Box::new(|_,_| true),
                on_fail: None,
                target: dest,
            };
        }
    }
    None
}

