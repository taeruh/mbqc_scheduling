/*!
**The API of this library is not ensured to be stable, except maybe the [interface] and
[probabilistic] modules.**

The [interface] module contains the API to run the algorithm, together with the
[probabilistic] module which provides some setups for the algorithm.

The modules [scheduler] contain the main logic [search]: [scheduler] implements the
scheduling process and [search] implements based on that the search for the optimal
schedule patterns.
*/

macro_rules! non_semantic_default {
    () => {
        "Note that semantically, this impl makes not much sense. It is rather useful for \
         initialization."
    };
}

pub mod interface;
pub mod probabilistic;
pub mod scheduler;
pub mod search;
pub mod timer;
