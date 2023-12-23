/*!
The [scheduler] module contains the main logic of the program. The other modules just
use it to solve the main problem we want to solve, wrap it into a parallelized runner and expose a 
*/

macro_rules! non_semantic_default {
    () => {
        "Note that semantically, this impl makes not much sense. It is rather useful \
         for initialization."
    };
}

pub mod cli;
pub mod run;
pub mod scheduler;
