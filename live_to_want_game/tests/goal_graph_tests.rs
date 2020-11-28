use std::sync::Arc;

use live_to_want_game::*;
// PRETTY SURE GoalNode is fucked and needs Rc in connections to work
// because if u return a GoalNode the connected other GoalNodes go out of scope
fn generate_basic_graph() -> GoalNode<'static> {
    let mut root = GoalNode {
        get_want_local: Box::new(|_, _| 0),
        get_effort_local: Box::new(|_, _| 1),
        children: Vec::new(),
        name: "root",
        get_command: None,
        get_requirements_met: Box::new(|_, _| false),
    };
    let mut gather = GoalNode {
        get_want_local: Box::new(|_, _| 0),
        get_effort_local: Box::new(|_, _| 1),
        children: Vec::new(),
        name: "gather",
        get_command: None,
        get_requirements_met: Box::new(|_, _| false),
    };
    let mut hunt = GoalNode {
        get_want_local: Box::new(|_, _| 0),
        get_effort_local: Box::new(|_, _| 1),
        children: Vec::new(),
        name: "hunt",
        get_command: None,
        get_requirements_met: Box::new(|_, _| false),
    };

    // gather, normally these would lead to eat/sells but lazy for this test
    let berry = GoalNode {
        get_want_local: Box::new(|_, _| {
            100
        }),
        get_effort_local: Box::new(|_, c| {
            if c.components.location_component.location.x == 1 {
                30
            } else {
                50
            }
        }),
        children: Vec::new(),
        name: "berry",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("berry", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, _| true),
    };
    let fruit = GoalNode {
        get_want_local: Box::new(|_, c| {
            if c.components.location_component.location.y == 1 {
                101
            } else {
                99
            }
        }),
        get_effort_local: Box::new(|_, c| {
            if c.components.location_component.location.x == 1 {
                30
            } else {
                50
            }
        }),
        children: Vec::new(),
        name: "fruit",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("fruit", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, _| true),
    };
    gather.children.push(GoalConnection{
        child: Arc::new(berry),
        is_additive: false,
        amplifier: 1.0,
    });
    gather.children.push(GoalConnection{
        child: Arc::new(fruit),
        is_additive: false,
        amplifier: 1.0,
    });


    //hunt stuff
    let mut find_deer = GoalNode {
        get_want_local: Box::new(|_, _| {
            0
        }),
        get_effort_local: Box::new(|_, _| {
            50
        }),
        children: Vec::new(),
        name: "find_deer",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("find_deer", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, _| true),
    };
    let mut attack_deer = GoalNode {
        get_want_local: Box::new(|_, _| {
            0
        }),
        get_effort_local: Box::new(|_, _| {
            1
        }),
        children: Vec::new(),
        name: "attack_deer",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("attack_deer", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, c| c.components.location_component.location.x==5),
    };
    let mut loot_deer = GoalNode {
        get_want_local: Box::new(|_, _| {
            0
        }),
        get_effort_local: Box::new(|_, _| {
            1
        }),
        children: Vec::new(),
        name: "loot_deer",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("loot_deer", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, c| c.components.location_component.location.x==6),
    };
    
    let eat = GoalNode {
        get_want_local: Box::new(|_, _| {
            10
        }),
        get_effort_local: Box::new(|_, _| {
            1
        }),
        children: Vec::new(),
        name: "eat",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("eat", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, c| c.components.location_component.location.y==0 && c.components.location_component.location.x==7),
    };
    let eat = Arc::new(eat);
    let sell = GoalNode {
        get_want_local: Box::new(|_, _| {
            10
        }),
        get_effort_local: Box::new(|_, _| {
            1
        }),
        children: Vec::new(),
        name: "sell",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("sell", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, c| c.components.location_component.location.y==1 && 
            (c.components.location_component.location.x==7 || c.components.location_component.location.x==11)),
    };
    let sell = Arc::new(sell);

    loot_deer.children.push(GoalConnection{
        child: sell.clone(),
        is_additive: true,
        amplifier: 4.0,
    });
    loot_deer.children.push(GoalConnection{
        child: eat.clone(),
        is_additive: true,
        amplifier: 7.0,
    });
    attack_deer.children.push(GoalConnection{
        child: Arc::new(loot_deer),
        is_additive: false,
        amplifier: 1.0,
    });
    find_deer.children.push(GoalConnection{
        child: Arc::new(attack_deer),
        is_additive: false,
        amplifier: 1.0,
    });
    hunt.children.push(GoalConnection{
        child: Arc::new(find_deer),
        is_additive: false,
        amplifier: 1.0,
    });


    let mut find_wolf = GoalNode {
        get_want_local: Box::new(|_, _| {
            0
        }),
        get_effort_local: Box::new(|_, _| {
            60
        }),
        children: Vec::new(),
        name: "find_wolf",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("find_wolf", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, _| true),
    };
    let mut attack_wolf = GoalNode {
        get_want_local: Box::new(|_, _| {
            0
        }),
        get_effort_local: Box::new(|_, _| {
            1
        }),
        children: Vec::new(),
        name: "attack_wolf",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("attack_wolf", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, c| c.components.location_component.location.x==9),
    };
    let mut loot_wolf = GoalNode {
        get_want_local: Box::new(|_, _| {
            0
        }),
        get_effort_local: Box::new(|_, _| {
            1
        }),
        children: Vec::new(),
        name: "loot_wolf",
        get_command: Some(Box::new(|_, c| CreatureCommand::MoveTo("loot_wolf", c, Vu2{x: 0, y:0}))),
        get_requirements_met: Box::new(|_, c| c.components.location_component.location.x==10),
    };
    loot_wolf.children.push(GoalConnection{
        child: sell.clone(),
        is_additive: true,
        amplifier: 12.0,
    });
    attack_wolf.children.push(GoalConnection{
        child: Arc::new(loot_wolf),
        is_additive: false,
        amplifier: 1.0,
    });
    find_wolf.children.push(GoalConnection{
        child: Arc::new(attack_wolf),
        is_additive: false,
        amplifier: 1.0,
    });
    hunt.children.push(GoalConnection{
        child: Arc::new(find_wolf),
        is_additive: false,
        amplifier: 1.0,
    });

    root.children.push(GoalConnection{
        child: Arc::new(gather),
        is_additive: false,
        amplifier: 1.0,
    });
    root.children.push(GoalConnection{
        child: Arc::new(hunt),
        is_additive: false,
        amplifier: 1.0,
    });

    root
}


// should be
// loc x=1, y=0 -> berry wins
#[test]
fn berry_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 1, y:0});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "berry"),
        _ => panic!("should return moveto!"),
    };
}
// loc x=1 y=1 -> fruit wins
#[test]
fn fruit_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 1, y:1});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "fruit"),
        _ => panic!("should return moveto!"),
    };
}
// x=0 y=0 -> hunt deer wins
#[test]
fn find_deer_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 0, y:0});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "find_deer"),
        _ => panic!("should return moveto!"),
    };
}
// x=5 -> attack deer
#[test]
fn attack_deer_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 5, y:0});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "attack_deer"),
        _ => panic!("should return moveto!"),
    };
}
// x=6 -> loot deer
#[test]
fn loot_deer_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 6, y:0});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "loot_deer"),
        _ => panic!("should return moveto!"),
    };
}
// x=7 y=0 -> eat deer (req met for eat if x==7 and y==0)
#[test]
fn eat_deer_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 7, y:0});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "eat"),
        _ => panic!("should return moveto!"),
    };
}
// x=7 y=1 -> sell deer (req met for sell if x==7 and y==1) OR x==11 (sell wolf)
#[test]
fn sell_deer_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 7, y:1});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "sell"),
        _ => panic!("should return moveto!"),
    };
}
// x=9 -> attack wolf
#[test]
fn attack_wolf_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 9, y:0});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "attack_wolf"),
        _ => panic!("should return moveto!"),
    };
}

// x=10 -> loot wolf
#[test]
fn loot_wolf_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 10, y:0});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "loot_wolf"),
        _ => panic!("should return moveto!"),
    };
}

// x=11 -> sell wolf
#[test]
fn sell_wolf_wins() {
    let root = generate_basic_graph();
    let m_s = MapState::default();
    let c_s = CreatureState::new(Vu2{x: 11, y:1});
    let res = GoalCacheNode::get_final_command(&root, &m_s, &c_s);
    let res = res.unwrap();
    println!("Got: {:#?}", &res);

    match res {
        CreatureCommand::MoveTo(n, _, _) => assert_eq!(n, "sell"),
        _ => panic!("should return moveto!"),
    };
}

#[test]
fn how_does_lifetime_loops() {
    let mut v = vec![1,2,3];
    fn fun (vv: &mut Vec<i32>) {
        vv[0] += 1;
    }
    for _ in 0..3 {
        let f = &mut v;
        fun(f);
    }
}


