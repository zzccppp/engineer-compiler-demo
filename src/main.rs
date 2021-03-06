use engineer_compiler_demo1::nfa::{
    epsilon_closure, epsilon_closure_dfs_delta, nfa_to_dfa, NFAIdAllocator, NFA,
};

use std::cell::RefCell;

use std::rc::Rc;

fn main() {
    let id_alloc = Rc::new(RefCell::new(NFAIdAllocator::default()));

    let mut temp = NFA::new_nfa_single_character(&id_alloc, 'b')
        .or(NFA::new_nfa_single_character(&id_alloc, 'c'))
        .unwrap();
    temp.asterisk_closure();
    let expression = NFA::new_nfa_single_character(&id_alloc, 'a')
        .connect(temp)
        .unwrap();
    println!("{:?}", expression);
    let x = epsilon_closure(&expression);
    println!("{:?}", x);
    let y = epsilon_closure_dfs_delta(&expression, &x, 'a');
    println!("{:?}", y);
    let z = epsilon_closure_dfs_delta(&expression, &y, 'b');
    println!("{:?}", z);

    //a(b|c)*
    let dfa = nfa_to_dfa(&expression);

    println!("{}", dfa.match_str("eracbds"));
}
