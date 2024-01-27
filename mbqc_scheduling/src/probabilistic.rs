//! Probabilistic accept functions (a little bit like a Markov chain), for the search
//! algorithm.

/// The type of the underlying accept function for [AcceptFunc].
///
/// Let `accept_func` be the accept function, then its signature is
/// ```ignore
/// fn accept_func(
///    bound_best_mem: f64, // the lowest memory that was already found for a full path
///                         // with less steps than the current path
///    minimal_mem: f64, // the lowest memory that was already found for a full path
///    last_max_mem: f64, // the maximum memory that was required so far
///    last_cur_mem: f64, // the memory that was required for the last step
///    cur_mem: f64, // the memory that is required for the current step
///    num_remaining_nodes: f64, // how many nodes are still left to measure
///    num_total_nodes: f64, // total number of nodes in the graph
/// ) -> f64 // the probability to accept the current step; has to be semi-positive;
///          // probabilities above 1. are allowed and mean that the path is always
///          // accepted
/// ```
pub type AcceptFn = Box<dyn Fn(f64, f64, f64, f64, f64, f64, f64) -> f64 + Send + Sync>;

#[inline]
fn builtin_heavyside(
    _: f64,
    minimal_mem: f64,
    last_max_mem: f64,
    _: f64,
    cur_mem: f64,
    num_remaining_nodes: f64,
    num_total_nodes: f64,
) -> f64 {
    let diff = minimal_mem - f64::max(cur_mem, last_max_mem);
    if diff < 0. {
        0.
    } else {
        num_total_nodes.powi(2)
            * (-(num_total_nodes * num_remaining_nodes
                / diff.powi(3)
                / (num_total_nodes - num_remaining_nodes)))
                .exp()
    }
}

fn create_parametrized_heavyside(
    cutoff: f64,
    lin_num_total_nodes_exp: i32,
    exp_num_total_nodes_exp: i32,
    exp_num_remaining_nodes_exp: i32,
    exp_diff_exp: i32,
    exp_num_measured_nodes_exp: i32,
) -> impl Fn(f64, f64, f64, f64, f64, f64, f64) -> f64 {
    move |_,
          minimal_mem,
          last_max_mem,
          _,
          cur_mem,
          num_remaining_nodes,
          num_total_nodes| {
        let diff = minimal_mem - f64::max(cur_mem, last_max_mem);
        if diff < cutoff {
            0.
        } else {
            num_total_nodes.powi(lin_num_total_nodes_exp)
                * (-(num_total_nodes.powi(exp_num_total_nodes_exp)
                    * num_remaining_nodes.powi(exp_num_remaining_nodes_exp)
                    / diff.powi(exp_diff_exp)
                    / (num_total_nodes - num_remaining_nodes)
                        .powi(exp_num_measured_nodes_exp)))
                .exp()
        }
    }
}

/// The possible accept functions.
///
/// Get the underlying accept function with [AcceptFunc::get_accept_func]. Compare
/// [AcceptFn], which describes the signature of the underlying accept function.
#[derive(Default)]
pub enum AcceptFunc {
    #[default]
    /// A fixed accept function that is used by default. Following the definitions in
    /// [AcceptFn], this function is defined as
    /// ```ignore
    /// let diff = minimal_mem - f64::max(cur_mem, last_max_mem);
    /// if diff < 0. {
    ///     0.
    /// } else {
    ///     num_total_nodes.powi(2)
    ///         * (-(num_total_nodes * num_remaining_nodes
    ///             / diff.powi(3)
    ///             / (num_total_nodes - num_remaining_nodes)))
    ///            .exp();
    /// ```
    /// It rather aggresively rejects potential time optimal paths in favor of faster
    /// finding memory optimal paths.
    BuiltinHeavyside,
    /// A parametrized version of [BuiltinHeavyside](AcceptFunc::BuiltinHeavyside).
    /// Following [AcceptFn], this function is defined as
    /// ```ignore
    /// let diff = minimal_mem - f64::max(cur_mem, last_max_mem);
    /// if diff < cutoff {
    ///    0.
    /// } else {
    ///    num_total_nodes.powi(lin_num_total_nodes_exp)
    ///        * (-(num_total_nodes.powi(exp_num_total_nodes_exp)
    ///            * num_remaining_nodes.powi(exp_num_remaining_nodes_exp)
    ///            / diff.powi(exp_diff_exp)
    ///            / (num_total_nodes - num_remaining_nodes)
    ///            .powi(exp_num_measured_nodes_exp)))
    ///        .exp();
    /// ```
    ParametrizedHeavyside {
        cutoff: f64,
        lin_num_total_nodes_exp: i32,
        exp_num_total_nodes_exp: i32,
        exp_num_remaining_nodes_exp: i32,
        exp_diff_exp: i32,
        exp_num_measured_nodes_exp: i32,
    },
    /// A custom accept function.
    Custom(AcceptFn),
}

impl AcceptFunc {
    /// Returns the underlying accept function.
    pub fn get_accept_func(self) -> AcceptFn {
        match self {
            AcceptFunc::BuiltinHeavyside => Box::new(builtin_heavyside),
            AcceptFunc::ParametrizedHeavyside {
                cutoff,
                lin_num_total_nodes_exp,
                exp_num_total_nodes_exp,
                exp_num_remaining_nodes_exp,
                exp_diff_exp,
                exp_num_measured_nodes_exp,
            } => Box::new(create_parametrized_heavyside(
                cutoff,
                lin_num_total_nodes_exp,
                exp_num_total_nodes_exp,
                exp_num_remaining_nodes_exp,
                exp_diff_exp,
                exp_num_measured_nodes_exp,
            )),
            AcceptFunc::Custom(f) => f,
        }
    }
}
