extern crate rand;
use self::rand::Rng;

#[derive(Clone, Debug, PartialEq)]
pub struct MathRLGameMove {
    pub operation: usize, // 0 = undo, 1 = add, 2 = subtract, 3 = multiply, 4 = divide
    pub pos1: usize,
    pub pos2: usize,
    pub save_into_pos1: bool,
    pub undone: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct MathRLGame{
    pub starting: Vec<f32>,
    pub goal: Vec<f32>,
    pub moves: Vec<MathRLGameMove>,
    pub current: Vec<f32>,
    pub answer: Vec<MathRLGameMove>,
    pub won: bool,
}
impl MathRLGame{
    pub fn new(number_length: usize, sequence_until_goal: usize, max_num: f32) -> MathRLGame {
        let mut rng = rand::thread_rng();
        let mut starting = vec![];
        for _ in 0..number_length {
            let r:f32 = rng.gen_range(1., max_num);
            starting.push(r);
        };
        let mut goal = starting.clone();
        let mut answer = vec![];
        let length = starting.len();
        for _ in 0..sequence_until_goal {
            let pos1 = rng.gen_range(0, length);
            let mut pos2 = rng.gen_range(0, length);
            if pos2 == pos1 {
                pos2 = if pos2 == (length - 1) { 
                    rng.gen_range(1, pos1) 
                } else { 
                    pos1 + 1
                }
            }
            assert!(pos1 < length);
            assert!(pos2 < length);
            assert_ne!(pos1, pos2);
            let mut op = MathRLGameMove {
                operation: rng.gen_range(1, 5),
                pos1,
                pos2,
                save_into_pos1: rng.gen_bool(0.5),
                undone: false,
            };
            // Don't divide by zero
            if op.operation == 4 {
                let divisor = if op.save_into_pos1 { goal[op.pos2] } else { goal[op.pos1] };
                if divisor == 0. {
                    op.operation = 1;
                }
            }
            // dont mult by 0, makes it boring
            if op.operation == 3 && (goal[op.pos1] == 0. || goal[op.pos2] == 0.) {
                op.operation = 1;
            }

            MathRLGame::operate_inplace(&op, &mut goal, None);
            answer.push(op);
        }

        MathRLGame {
            goal,
            moves: vec![],
            current: starting.clone(),
            starting,
            answer,
            won: false,
        }
    }
    pub fn operate_self(&mut self, op: MathRLGameMove) {
        let mut prev = None;
        if op.operation == 0 {
            for i in (0..self.moves.len()).rev() {
                if !self.moves[i].undone && self.moves[i].operation != 0 {
                    prev = Some(&mut self.moves[i]);
                    break;
                }
            }
        }
        if op.operation == 0 && prev.is_none() {
            self.moves.push(op);
            return;
        } else {
            MathRLGame::operate_inplace(&op, &mut self.current, prev);
            self.moves.push(op);
            if self.check_if_won() {
                self.won = true;
            }
        }
    }

    pub fn check_if_won(&self) -> bool {
        return self.current == self.goal;

        let mut diff = 0.;
        for i in 0..self.goal.len() {
            diff += (self.current[i] - self.goal[i]).abs();
        }
        return diff < 0.0001;
    }

    pub fn operate(op: &MathRLGameMove, src: &Vec<f32>, previous: Option<&mut MathRLGameMove>) -> Vec<f32> {
        let mut r = src.clone();
        MathRLGame::operate_inplace(op, &mut r, previous);
        r
    }
    pub fn operate_inplace(op: &MathRLGameMove, src: &mut Vec<f32>, previous: Option<&mut MathRLGameMove>) {
        let starting = src;
        let result_pos = if op.save_into_pos1 {
            op.pos1
        } else {
            op.pos2
        };
        let other_pos = if op.save_into_pos1 {
            op.pos2
        } else {
            op.pos1
        };
        match op.operation {
            0 => { // undo
                let mut prev = previous.unwrap();
                let result_pos = if prev.save_into_pos1 {
                    prev.pos1
                } else {
                    prev.pos2
                };
                let other_pos = if prev.save_into_pos1 {
                    prev.pos2
                } else {
                    prev.pos1
                };
                match prev.operation {
                    1 => { // add
                        starting[result_pos] = starting[result_pos] - starting[other_pos];
                    }
                    2 => { // sub
                        starting[result_pos] = starting[result_pos] + starting[other_pos];
                    }
                    3 => { // mult
                        starting[result_pos] = starting[result_pos] / starting[other_pos];
                    }
                    4 => { // divide
                        starting[result_pos] = starting[result_pos] * starting[other_pos];
                    }
                    _ => panic!("unexpected operation previous {}", prev.operation),
                }
                assert_eq!(prev.undone, false);
                prev.undone = true;
            },

            1 => { // add
                starting[result_pos] = starting[op.pos1] + starting[op.pos2]
            },
            2 => { // sub
                starting[result_pos] = starting[result_pos] - starting[other_pos];
            }
            3 => {
                starting[result_pos] = starting[result_pos] * starting[other_pos];
            }
            4 => {
                starting[result_pos] = starting[result_pos] / starting[other_pos];
            }
            _ => {
                panic!("unexpected operation {}", op.operation);
            }
        }        
    }
}


// TESTS:
#[test]
pub fn test_math_rl_game() {
    // perform a bunch of moves and some undos and make sure it works
    let mut many_undos = MathRLGame {
        starting: vec![2., 2., 3.],
        goal: vec![1., 6., 12.],
        moves: vec![],
        current: vec![2., 2., 3.],
        answer: vec![],
        won: false,
    };
    let moves_to_do = vec![
        MathRLGameMove { // 3*2 = 6
            operation: 3,
            pos1: 2, 
            pos2: 1, 
            save_into_pos1: true, 
            undone: false 
        },
        MathRLGameMove { 
            operation: 3, // 6*2 = 12
            pos1: 2, 
            pos2: 1, 
            save_into_pos1: true, 
            undone: false 
        },

        // 2 wrong actions, undo one, then do another wrong action, then undo twice
        MathRLGameMove { 
            operation: 2, // 2 - 12 = -10
            pos1: 1,
            pos2: 2, 
            save_into_pos1: true, 
            undone: false
        },
        MathRLGameMove { 
            operation: 4, // 2 / -10 = -.2
            pos1: 0, 
            pos2: 1,
            save_into_pos1: true, 
            undone: false 
        },
        MathRLGameMove { 
            operation: 0, // undo
            pos1: 0, 
            pos2: 0,
            save_into_pos1: true, 
            undone: false 
        }, // 2, -10, 12
        MathRLGameMove {
            operation: 4, // -10/2 = -5
            pos1: 1, 
            pos2: 0,
            save_into_pos1: true, 
            undone: false 
        },
        MathRLGameMove { 
            operation: 0, // undo
            pos1: 0, 
            pos2: 0,
            save_into_pos1: true, 
            undone: false 
        }, 
        MathRLGameMove { 
            operation: 0, // undo
            pos1: 0, 
            pos2: 0,
            save_into_pos1: true, 
            undone: false 
        }, // 2, 6, 12
    ];
    for m in moves_to_do {
        many_undos.operate_self(m);
        //println!("game state now: {:#?}", many_undos);
    }
    assert_eq!(many_undos.current.into_iter().map(|x| (x * 100.).round()).collect::<Vec<f32>>(), vec![2.0f32, 2., 12.].into_iter().map(|x| (x * 100.).round()).collect::<Vec<f32>>());

    // make a test where we do a game. try every operation. and make sure u try subtract and divide with diff bools for which one u save to.
    let mut all_ops = MathRLGame {
        starting: vec![6., 4., 3.],
        goal: vec![9., 1., 4.],
        moves: vec![],
        current: vec![6., 4., 3.],
        answer: vec![],
        won: false,
    };
    let moves_for_win = vec![
        MathRLGameMove { // 6 * 4 = 24, 24, 4, 3
            operation: 3,
            pos1: 0, 
            pos2: 1, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // 8, 4, 3
            operation: 4,
            pos1: 2, 
            pos2: 0, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // undo
            operation: 0,
            pos1: 2, 
            pos2: 0, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // 8, 4, 3
            operation: 4,
            pos1: 2, 
            pos2: 0, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // 8, 4, 12
            operation: 3, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // 8, 4, 8
            operation: 2, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // 8, 4, 4
            operation: 2,
            pos1: 2, 
            pos2: 1, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // 8, 1, 4
            operation: 4, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // 8, 1, 4
            operation: 4, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        
        MathRLGameMove { // 8, 9, 4 (bad shud be undo)
            operation: 1, 
            pos1: 0, 
            pos2: 1, 
            save_into_pos1: false,
            undone: false 
        },

        MathRLGameMove { // idk
            operation: 4, 
            pos1: 0, 
            pos2: 1, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 4, 
            pos1: 0, 
            pos2: 2, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 3, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 3, 
            pos1: 2, 
            pos2: 1, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 3, 
            pos1: 0, 
            pos2: 1, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 3, 
            pos1: 0, 
            pos2: 2, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 1, 
            pos1: 0, 
            pos2: 1, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 4, 
            pos1: 1, 
            pos2: 0, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 1, 
            pos1: 0, 
            pos2: 1, 
            save_into_pos1: false,
            undone: false 
        },
        MathRLGameMove { // idk
            operation: 3, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: false,
            undone: false 
        },

        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },
        MathRLGameMove { // UNDO
            operation: 0, 
            pos1: 1, 
            pos2: 2, 
            save_into_pos1: true,
            undone: false 
        },


        MathRLGameMove { // 9, 1, 4
            operation: 1, 
            pos1: 1, 
            pos2: 0, 
            save_into_pos1: false,
            undone: false 
        },
    ];
    for op in moves_for_win {
        all_ops.operate_self(op);
        println!("game state now: {:#?}", all_ops.current);
    }
    assert_eq!(all_ops.current, vec![9., 1., 4.]);
    assert_eq!(all_ops.won, true);
    // Make a game with a 100 length sequence and check you win if u do all the moves in answer
}

#[test]
pub fn run_long_sequence() {
    let mut long_one = MathRLGame::new(3, 200, 99.);
    for mov in long_one.answer.clone() {
        long_one.operate_self(mov);
        println!("game state now: {:#?}", long_one.current);
    }
    println!("game state now: {:#?} goal {:#?}", long_one.current, long_one.goal);
    assert_eq!(long_one.won, true);
}

