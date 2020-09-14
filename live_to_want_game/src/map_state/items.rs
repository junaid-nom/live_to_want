#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, Hash, Eq)]
pub enum ItemType {
    Berry,
    Meat,
    Bones,
    Wood,
}
impl Default for ItemType {
    fn default() -> Self { ItemType::Berry }
}

#[derive(Debug)]
#[derive(Default, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Item {
    pub item_type: ItemType,
    pub quantity: u32,
}


#[derive(Debug)]
pub enum CreatureCommand<'b>{
    // str here is for debugging purposes and is usually just the name of the node
    MoveTo(&'static str, &'b CreatureState, Vector2),
    Chase(&'static str, &'b CreatureState, &'b CreatureState),
    Attack(&'static str, &'b CreatureState, &'b CreatureState),
    TakeItem(&'static str, InventoryHolder<'b>, InventoryHolder<'b>, Item),
}
impl CreatureCommand<'_> {
    pub fn to_event_chain(&self) -> Option<EventChain> {
        match self {
            CreatureCommand::MoveTo(_, _, _) => {}
            CreatureCommand::Chase(_, _, _) => {}
            CreatureCommand::Attack(_, _, _) => {}
            CreatureCommand::TakeItem(_, src, dst, item) => {
                // TODO: check if dst has enough space, though maybe just have "cant move" if your inv full
                // check if src has that item, if it doesnt, take as many as possible
                let found_item = get_item_from_inventory(src, item.item_type);
                if let None = found_item {
                    return None;
                }
                let found_item = found_item.unwrap();
                let final_item = if found_item.quantity < item.quantity {
                    found_item
                } else {
                    *item
                };

                // event chain is:
                // remove item from src. req=item exists in that quantity fail=None
                // add item to dst. req=None(for now) fail=None
                let remove = Event{
                    event_type: EventType::RemoveItem(final_item.quantity, item.item_type),
                    target: get_id_from_inventory(src),
                    on_fail: None,
                    get_requirements: Box::new(|e, et| {
                        if let EventType::RemoveItem(q, it) = et {
                            match e {
                                EventTarget::LocationItemTarget(i, _) => {
                                    for item in i.iter() {
                                        if item.item_type == *it && item.quantity >= *q {
                                            return true
                                        }
                                    }
                                    return false
                                }
                                EventTarget::CreatureTarget(c) => {
                                    for item in c.inventory.iter() {
                                        if item.item_type == *it && item.quantity >= *q {
                                            return true
                                        }
                                    }
                                    return false
                                }
                                _ => {
                                    panic!("Got eventtarget that isnt for items")
                                }
                            }
                        }
                        false
                    })
                };
                let add = Event {
                    event_type: EventType::AddItem(final_item.quantity, item.item_type),
                    on_fail: None,
                    get_requirements: Box::new(|_,_| true),
                    target: get_id_from_inventory(dst),
                };
                return Some(EventChain{
                    index: 0,
                    events: vec![remove, add],
                })
            }
        }
        None
    }
}

fn get_id_from_inventory(inv: &InventoryHolder) -> UID {
    match inv {
        InventoryHolder::CreatureInventory(c) => {c.components.id_component.id}
        InventoryHolder::LocationInventory(l) => {l.id_component_items.id}
    }
}

fn get_item_from_inventory(inv: &InventoryHolder, item_type: ItemType) -> Option<Item> {
    match inv {
        InventoryHolder::CreatureInventory(c) => {
            get_item_from_vec_item(&c.inventory,item_type)
        }
        InventoryHolder::LocationInventory(l) => {
            get_item_from_vec_item(&l.items,item_type)
        }
    }
}
fn get_item_from_vec_item(vec_inv: &Vec<Item>, item_type: ItemType) -> Option<Item> {
    for i in vec_inv {
        if i.item_type == item_type {
            return Some(*i)
        }
    }
    None
}

#[derive(Debug, Clone, Copy)]
pub enum InventoryHolder<'a> {
    CreatureInventory(&'a CreatureState),
    LocationInventory(&'a MapLocation),
}