use crate::{CodeSection, LocalLabel, Session};
use std::collections::HashMap;

// You must call this function after the bytecode optimization!
pub fn insert_ref_count<'hir, 'mir>(session: Session<'hir, 'mir>) -> Session<'hir, 'mir> {
    session  // TODO
//     let mut new_code = HashMap::with_capacity(session.object_file.code.len());
// 
//     for (label, code) in session.object_file.code.drain() {
//         let lowered = insert_ref_count_to_code_section(code);
//         new_code.insert(label, lowered);
//     }
// 
//     session.object_file.code = new_code;
//     session
}

// fn insert_ref_count_to_code_section(mut code: CodeSection) -> CodeSection {
//     let mut start_block = code.basic_blocks.get_mut(&LocalLabel::start()).unwrap();
// 
//     for i in 0..code.params.unwrap_or(0) {
//         // TODO: inc_ref_count(SSA(i))
//         todo!()
//     }
// 
//     for (label, block) in code.basic_blocks.drain() {
//         todo!()
//     }
// 
//     todo!()
// }
