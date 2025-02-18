// For Tic Tac Toe

use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
};

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize, Deserialize)]
enum Player {
    You,
    Opponent,
}

impl Player {
    fn opposite(&self) -> Player {
        match *self {
            Player::You => Player::Opponent,
            Player::Opponent => Player::You,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize, Deserialize)]
struct GameState {
    board: [[Option<Player>; 3]; 3],
    turn: Player,
}

impl GameState {
    fn opposite(&self) -> GameState {
        let mut board = self.board;
        for i in 0..3 {
            for j in 0..3 {
                board[i][j] = board[i][j].map(|player| player.opposite());
            }
        }
        let turn = self.turn.opposite();
        GameState { board, turn }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Serialize, Deserialize)]
struct Action {
    row: usize,
    col: usize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
enum GameStateInfo {
    W,   // Forced Win
    WD,  // Never Lose
    WDL, // Not Sure
    WL,  // Never Draw
    D,   // Forced Draw
    DL,  // Never Win
    L,   // Forced Lose
}

enum Progress {
    Draw,
    Win(Player),
    Ongoing,
}

impl GameState {
    fn progress(&self) -> Progress {
        for i in 0..3 {
            if self.board[i][0].is_some()
                && self.board[i][0] == self.board[i][1]
                && self.board[i][0] == self.board[i][2]
            {
                return Progress::Win(self.board[i][0].unwrap());
            }
            if self.board[0][i].is_some()
                && self.board[0][i] == self.board[1][i]
                && self.board[0][i] == self.board[2][i]
            {
                return Progress::Win(self.board[0][i].unwrap());
            }
        }
        if self.board[0][0].is_some()
            && self.board[0][0] == self.board[1][1]
            && self.board[0][0] == self.board[2][2]
        {
            return Progress::Win(self.board[0][0].unwrap());
        }
        if self.board[0][2].is_some()
            && self.board[0][2] == self.board[1][1]
            && self.board[0][2] == self.board[2][0]
        {
            return Progress::Win(self.board[0][2].unwrap());
        }
        let mut result = Progress::Draw;
        for i in 0..3 {
            for j in 0..3 {
                if self.board[i][j].is_none() {
                    result = Progress::Ongoing;
                }
            }
        }
        result
    }

    fn possible_actions(&self) -> Vec<Action> {
        if matches!(self.progress(), Progress::Win(_)) {
            return Vec::new();
        }
        let mut result = Vec::new();
        for row in 0..3 {
            for col in 0..3 {
                if self.board[row][col].is_none() {
                    result.push(Action { row, col });
                }
            }
        }
        result
    }

    fn next(&self, action: Action) -> GameState {
        let mut board = self.board;
        board[action.row][action.col] = Some(self.turn);
        let turn = self.turn.opposite();
        GameState { board, turn }
    }
}

struct GameStateGraphNode {
    state: GameState,
    actions_from_here: BTreeMap<Action, Rc<RefCell<GameStateGraphNode>>>,
    actions_to_here: Vec<Weak<RefCell<GameStateGraphNode>>>,
    info: Option<GameStateInfo>,
}

fn build_info(summary: (Player, bool, bool, bool, bool, bool, bool, bool)) -> GameStateInfo {
    match summary {
        // Forced Win
        (Player::You, true, _, _, _, _, _, _) => GameStateInfo::W,
        (Player::Opponent, true, false, false, false, false, false, false) => GameStateInfo::W,

        // Forced Lose
        (Player::You, false, false, false, false, false, false, true) => GameStateInfo::L,
        (Player::Opponent, _, _, _, _, _, _, true) => GameStateInfo::L,

        // Forced Draw
        (_, false, false, false, false, true, false, false) => GameStateInfo::D,

        // Never Win
        (Player::Opponent, _, _, _, _, true, _, _) => GameStateInfo::DL,
        (Player::Opponent, _, _, _, _, _, true, _) => GameStateInfo::DL,
        (_, false, false, false, false, _, _, _) => GameStateInfo::DL,

        // Never Lose
        (_, _, _, false, false, _, false, false) => GameStateInfo::WD,

        // Never Draw
        (_, _, false, false, _, false, false, _) => GameStateInfo::WL,

        _ => GameStateInfo::WDL,
    }
}

fn build_info_recursively(node: &Rc<RefCell<GameStateGraphNode>>) {
    if node.borrow().info.is_some() {
        // already built
        return;
    }
    if node.borrow().state.possible_actions().len() != node.borrow().actions_from_here.len() {
        // incomplete
        return;
    }
    let mut complete = true;
    let mut has_w = false;
    let mut has_wd = false;
    let mut has_wdl = false;
    let mut has_wl = false;
    let mut has_d = false;
    let mut has_dl = false;
    let mut has_l = false;
    for (_, next) in node.borrow().actions_from_here.iter() {
        match next.borrow().info {
            None => complete = false,
            Some(GameStateInfo::W) => has_w = true,
            Some(GameStateInfo::WD) => has_wd = true,
            Some(GameStateInfo::WDL) => has_wdl = true,
            Some(GameStateInfo::WL) => has_wl = true,
            Some(GameStateInfo::D) => has_d = true,
            Some(GameStateInfo::DL) => has_dl = true,
            Some(GameStateInfo::L) => has_l = true,
        }
    }
    if !complete {
        return;
    }

    let progress = node.borrow().state.progress();
    let info = match progress {
        Progress::Ongoing => build_info((
            node.borrow().state.turn,
            has_w,
            has_wd,
            has_wdl,
            has_wl,
            has_d,
            has_dl,
            has_l,
        )),
        Progress::Draw => GameStateInfo::D,
        Progress::Win(Player::You) => GameStateInfo::W,
        Progress::Win(Player::Opponent) => GameStateInfo::L,
    };
    node.borrow_mut().info.replace(info);

    for previous in node.borrow().actions_to_here.iter() {
        build_info_recursively(&previous.upgrade().unwrap());
    }
}

fn build_next_states_recursively(
    node: &Rc<RefCell<GameStateGraphNode>>,
    map: &mut BTreeMap<GameState, Rc<RefCell<GameStateGraphNode>>>,
) {
    if map.get(&node.borrow().state).is_some() {
        return;
    } else {
        map.insert(node.borrow().state, node.clone());
    }
    let actions = node.borrow().state.possible_actions();
    for action in actions.iter() {
        let next_state = node.borrow().state.next(*action);
        if let Some(next) = map.get(&next_state) {
            node.borrow_mut()
                .actions_from_here
                .insert(*action, next.clone());
            next.borrow_mut().actions_to_here.push(Rc::downgrade(node));
            continue;
        }
        let next_node = Rc::new(RefCell::new(GameStateGraphNode {
            state: next_state,
            actions_from_here: BTreeMap::new(),
            actions_to_here: Vec::new(),
            info: None,
        }));
        node.borrow_mut()
            .actions_from_here
            .insert(*action, next_node.clone());
        next_node
            .borrow_mut()
            .actions_to_here
            .push(Rc::downgrade(node));
        build_next_states_recursively(&next_node, map);
    }
    build_info_recursively(node);
}

fn main() {
    let initial_node = Rc::new(RefCell::new(GameStateGraphNode {
        state: GameState {
            board: [[None; 3]; 3],
            turn: Player::You,
        },
        actions_from_here: BTreeMap::new(),
        actions_to_here: Vec::new(),
        info: None,
    }));
    let mut map = BTreeMap::new();
    build_next_states_recursively(&initial_node, &mut map);
    for (_, node) in map.iter() {
        println!("{:?} - {:?}", node.borrow().state, node.borrow().info);
        for (_, node) in node.borrow().actions_from_here.iter() {
            println!("    {:?} - {:?}", node.borrow().state, node.borrow().info);
        }
    }
}
