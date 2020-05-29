use crate::nfa::FANodeType::{End, Normal};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

use std::rc::Rc;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum FANodeType {
    Start,
    End,
    Normal,
}

#[derive(Debug, Clone)]
pub struct NFATransferInfo {
    from_id: u64,
    to_id: u64,
    condition: Option<char>,
}

#[derive(Debug, Clone)]
pub struct DFATransferInfo {
    from_id: u64,
    to_id: u64,
    condition: char,
}

#[derive(Debug, Clone)]
pub struct NFANode {
    node_type: FANodeType,
    id: u64,
    transfers: Vec<NFATransferInfo>,
}

#[derive(Debug, Clone)]
pub struct DFANode {
    node_type: FANodeType,
    id: u64,
    transfers: Vec<DFATransferInfo>,
}

#[derive(PartialOrd, PartialEq, Debug)]
pub struct NFAIdAllocator {
    index: u64,
}

impl Default for NFAIdAllocator {
    fn default() -> Self {
        Self { index: 0 }
    }
}

impl NFAIdAllocator {
    pub fn next_id(&mut self) -> u64 {
        self.index += 1;
        self.index
    }
}

#[derive(Debug)]
pub struct DFA {
    pub nodes: HashMap<u64, DFANode>,
    pub start_id: u64,
}

#[derive(Debug)]
pub struct NFA {
    pub nodes: HashMap<u64, NFANode>,
    pub start_id: u64,
    pub end_id: u64,
    pub character_set: HashSet<char>,
    id_alloc: Rc<RefCell<NFAIdAllocator>>,
}

impl NFA {
    pub fn new_nfa_single_character(id_alloc: &Rc<RefCell<NFAIdAllocator>>, c: char) -> Self {
        let id1 = (*id_alloc).borrow_mut().next_id();
        let id2 = (*id_alloc).borrow_mut().next_id();
        let start_node = NFANode {
            node_type: FANodeType::Start,
            id: id1,
            transfers: vec![NFATransferInfo {
                from_id: id1,
                to_id: id2,
                condition: Some(c),
            }],
        };
        let end_node = NFANode {
            node_type: FANodeType::End,
            id: id2,
            transfers: vec![],
        };
        let mut map = HashMap::new();
        map.insert(start_node.id, start_node);
        map.insert(end_node.id, end_node);

        let mut set = HashSet::new();
        set.insert(c);
        NFA {
            nodes: map,
            start_id: id1,
            end_id: id2,
            character_set: set,
            id_alloc: id_alloc.clone(),
        }
    }

    pub fn connect(mut self, rhs: Self) -> Result<Self, ()> {
        if self.id_alloc != rhs.id_alloc {
            Err(())
        } else {
            let ns: Vec<(u64, NFANode)> = rhs.nodes.into_iter().collect();
            for n in ns {
                self.nodes.insert(n.0, n.1);
            }
            let a = self.nodes.get_mut(&self.end_id).unwrap();
            a.transfers.push(NFATransferInfo {
                from_id: self.end_id,
                to_id: rhs.start_id,
                condition: None,
            });
            a.node_type = Normal;
            self.nodes.get_mut(&rhs.start_id).unwrap().node_type = Normal;
            self.end_id = rhs.end_id;

            let cs: Vec<char> = rhs.character_set.into_iter().collect();
            for c in cs {
                self.character_set.insert(c);
            }
            Ok(self)
        }
    }

    pub fn or(mut self, rhs: Self) -> Result<Self, ()> {
        if self.id_alloc != rhs.id_alloc {
            return Err(());
        }
        let id1 = (*self.id_alloc).borrow_mut().next_id();
        let id2 = (*self.id_alloc).borrow_mut().next_id();
        let first = NFANode {
            node_type: FANodeType::Start,
            id: id1,
            transfers: vec![
                NFATransferInfo {
                    from_id: id1,
                    to_id: self.start_id,
                    condition: None,
                },
                NFATransferInfo {
                    from_id: id1,
                    to_id: rhs.start_id,
                    condition: None,
                },
            ],
        };
        let end = NFANode {
            node_type: FANodeType::End,
            id: id2,
            transfers: vec![],
        };

        let ns: Vec<(u64, NFANode)> = rhs.nodes.into_iter().collect();
        for n in ns {
            self.nodes.insert(n.0, n.1);
        }
        self.nodes.insert(id1, first);
        self.nodes.insert(id2, end);

        self.nodes
            .get_mut(&self.end_id)
            .unwrap()
            .transfers
            .push(NFATransferInfo {
                from_id: self.end_id,
                to_id: id2,
                condition: None,
            });
        self.nodes
            .get_mut(&rhs.end_id)
            .unwrap()
            .transfers
            .push(NFATransferInfo {
                from_id: rhs.end_id,
                to_id: id2,
                condition: None,
            });

        self.nodes.get_mut(&self.start_id).unwrap().node_type = Normal;
        self.nodes.get_mut(&self.end_id).unwrap().node_type = Normal;
        self.nodes.get_mut(&rhs.start_id).unwrap().node_type = Normal;
        self.nodes.get_mut(&rhs.end_id).unwrap().node_type = Normal;

        self.start_id = id1;
        self.end_id = id2;

        let cs: Vec<char> = rhs.character_set.into_iter().collect();
        for c in cs {
            self.character_set.insert(c);
        }

        Ok(self)
    }

    //exp*
    pub fn asterisk_closure(&mut self) {
        let id1 = (*self.id_alloc).borrow_mut().next_id();
        let id2 = (*self.id_alloc).borrow_mut().next_id();
        let first = NFANode {
            node_type: FANodeType::Start,
            id: id1,
            transfers: vec![
                NFATransferInfo {
                    from_id: id1,
                    to_id: self.start_id,
                    condition: None,
                },
                NFATransferInfo {
                    from_id: id1,
                    to_id: id2,
                    condition: None,
                },
            ],
        };
        let end = NFANode {
            node_type: FANodeType::End,
            id: id2,
            transfers: vec![],
        };
        self.nodes
            .get_mut(&self.end_id)
            .unwrap()
            .transfers
            .push(NFATransferInfo {
                from_id: self.end_id,
                to_id: self.start_id,
                condition: None,
            });
        self.nodes
            .get_mut(&self.end_id)
            .unwrap()
            .transfers
            .push(NFATransferInfo {
                from_id: self.end_id,
                to_id: id2,
                condition: None,
            });
        self.nodes.insert(id1, first);
        self.nodes.insert(id2, end);
        self.nodes.get_mut(&self.start_id).unwrap().node_type = Normal;
        self.nodes.get_mut(&self.end_id).unwrap().node_type = Normal;
        self.start_id = id1;
        self.end_id = id2;
    }

    //exp+
    pub fn plus_closure(&mut self) {
        let id1 = (*self.id_alloc).borrow_mut().next_id();
        let id2 = (*self.id_alloc).borrow_mut().next_id();
        let first = NFANode {
            node_type: FANodeType::Start,
            id: id1,
            transfers: vec![NFATransferInfo {
                from_id: id1,
                to_id: self.start_id,
                condition: None,
            }],
        };
        let end = NFANode {
            node_type: FANodeType::End,
            id: id2,
            transfers: vec![],
        };
        self.nodes
            .get_mut(&self.end_id)
            .unwrap()
            .transfers
            .push(NFATransferInfo {
                from_id: self.end_id,
                to_id: self.start_id,
                condition: None,
            });
        self.nodes
            .get_mut(&self.end_id)
            .unwrap()
            .transfers
            .push(NFATransferInfo {
                from_id: self.end_id,
                to_id: id2,
                condition: None,
            });
        self.nodes.insert(id1, first);
        self.nodes.insert(id2, end);
        self.nodes.get_mut(&self.start_id).unwrap().node_type = Normal;
        self.nodes.get_mut(&self.end_id).unwrap().node_type = Normal;
        self.start_id = id1;
        self.end_id = id2;
    }

    //exp?
    pub fn question_closure(&mut self) {
        let id1 = (*self.id_alloc).borrow_mut().next_id();
        let id2 = (*self.id_alloc).borrow_mut().next_id();
        let first = NFANode {
            node_type: FANodeType::Start,
            id: id1,
            transfers: vec![
                NFATransferInfo {
                    from_id: id1,
                    to_id: self.start_id,
                    condition: None,
                },
                NFATransferInfo {
                    from_id: id1,
                    to_id: id2,
                    condition: None,
                },
            ],
        };
        let end = NFANode {
            node_type: FANodeType::End,
            id: id2,
            transfers: vec![],
        };
        self.nodes
            .get_mut(&self.end_id)
            .unwrap()
            .transfers
            .push(NFATransferInfo {
                from_id: self.end_id,
                to_id: id2,
                condition: None,
            });
        self.nodes.insert(id1, first);
        self.nodes.insert(id2, end);
        self.nodes.get_mut(&self.start_id).unwrap().node_type = Normal;
        self.nodes.get_mut(&self.end_id).unwrap().node_type = Normal;
        self.start_id = id1;
        self.end_id = id2;
    }
}

impl DFA {
    pub fn match_str(&self, str: &str) -> bool {
        let mut now_node = self.start_id;
        for c in str.chars() {
            let find_node = self
                .nodes
                .get(&now_node)
                .unwrap()
                .transfers
                .iter()
                .find(|x| x.condition == c);
            match find_node {
                None => {
                    return false;
                }
                Some(x) => {
                    now_node = x.to_id;
                }
            }
        }
        return true;
    }
}

pub fn nfa_to_dfa(nfa: &NFA) -> DFA {
    let q0 = epsilon_closure(nfa);
    let mut Q = Vec::new();
    let mut work_list = VecDeque::new();
    Q.push(q0.clone());
    work_list.push_front(q0.clone());

    let mut form = HashMap::<(usize, char), Option<usize>>::new();
    let mut Q_index: usize = 0;

    while !work_list.is_empty() {
        let q = work_list.pop_front().unwrap();
        for c in nfa.character_set.iter() {
            let t = epsilon_closure_dfs_delta(nfa, &q, *c);
            if t.len() != 0 {
                if !Q.contains(&t) {
                    Q.push(t.clone());
                    work_list.push_front(t.clone());
                    form.insert((Q_index, *c), Some(Q.len() - 1));
                } else {
                    for (idx, e) in Q.iter().enumerate() {
                        if *e == t {
                            form.insert((Q_index, *c), Some(idx));
                        }
                    }
                }
            } else {
                form.insert((Q_index, *c), None);
            }
        }
        Q_index += 1;
    }

    let mut nodes = HashMap::<u64, DFANode>::new();
    nodes.insert(
        0,
        DFANode {
            node_type: FANodeType::Start,
            id: 0,
            transfers: vec![],
        },
    );
    for i in 1..Q.len() {
        let ty = if Q
            .get(i)
            .unwrap()
            .iter()
            .any(|x| nfa.nodes.get(x).unwrap().node_type == End)
        {
            End
        } else {
            Normal
        };

        nodes.insert(
            i as u64,
            DFANode {
                node_type: ty,
                id: i as u64,
                transfers: vec![],
            },
        );
    }

    for ((k, c), v) in form {
        if let None = v {
            continue;
        } else {
            nodes
                .get_mut(&(k as u64))
                .unwrap()
                .transfers
                .push(DFATransferInfo {
                    from_id: k as u64,
                    to_id: v.unwrap() as u64,
                    condition: c,
                });
        }
    }

    // println!("{:?}", Q);
    // println!("{:?}", form);

    let dfa = DFA { nodes, start_id: 0 };

    println!("{:?}", dfa);

    return dfa;
}

pub fn epsilon_closure(nfa: &NFA) -> HashSet<u64> {
    let mut q0 = HashSet::new();
    epsilon_closure_dfs(nfa, &mut q0, nfa.start_id);
    q0.insert(nfa.start_id);
    return q0;
}

pub fn epsilon_closure_dfs_delta(nfa: &NFA, q: &HashSet<u64>, c: char) -> HashSet<u64> {
    let mut q0 = HashSet::new();
    for e in q {
        let trans = &nfa.nodes.get(e).unwrap().transfers;
        for t in trans {
            if Some(c) == t.condition {
                q0.insert(t.to_id);
                epsilon_closure_dfs(nfa, &mut q0, t.to_id);
            }
        }
    }
    return q0;
}

fn epsilon_closure_dfs(nfa: &NFA, q0: &mut HashSet<u64>, now: u64) {
    let edges = &nfa.nodes.get(&now).unwrap().transfers;
    for e in edges {
        if let None = e.condition {
            q0.insert(e.to_id);
            epsilon_closure_dfs(nfa, q0, e.to_id);
        }
    }
}
