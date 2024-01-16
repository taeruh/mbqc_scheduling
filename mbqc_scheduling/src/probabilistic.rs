//! Probabilistic accept functions (a little bit like a Markov chain), for the search
//! algorithm.

/// The type of the underlying accept function for [AcceptFunc].
///
/// Let `accept_func` be the accept function, then its signature is
/// ```ignore
/// fn accept_func(
///    bound_best_mem: f64, // the lowest memory that was already found for a full path
///                         // with less steps than the current path
///    last_max_mem: f64, // the maximum memory that was required so far
///    last_cur_mem: f64, // the memory that was required for the last step
///    cur_mem: f64, // the memory that is required for the current step
///    num_remaining_nodes: f64, // how many nodes are still left to measure
///    num_total_nodes: f64, // total number of nodes in the graph
/// ) -> f64 // the probability to accept the current step; has to be semi-positive;
///          // probabilities above 1. are allowed and mean that the path is always
///          // accepted
/// ```
pub type AcceptFn = Box<dyn Fn(f64, f64, f64, f64, f64, f64) -> f64 + Send + Sync>;

fn builtin_linear_space(
    bound_best_mem: f64,
    last_max_mem: f64,
    _: f64,
    cur_mem: f64,
    num_remaining_nodes: f64,
    num_total_nodes: f64,
) -> f64 {
    (bound_best_mem + 1.) / (f64::max(cur_mem, last_max_mem) + 1.)
        * 5e-3
        * (1e-3
            + 8.5e-2 * (num_total_nodes + 1.)
                / (num_total_nodes - num_remaining_nodes + 1.))
}

fn builtin_exponential_space(
    bound_best_mem: f64,
    last_max_mem: f64,
    _: f64,
    cur_mem: f64,
    num_remaining_nodes: f64,
    num_total_nodes: f64,
) -> f64 {
    ((bound_best_mem + 1.) / (f64::max(cur_mem, last_max_mem) + 1.)).exp()
        * 1e-1
        * (1e-3
            + 8.5e-2 * (num_total_nodes + 1.)
                / (num_total_nodes - num_remaining_nodes + 1.))
}

fn builtin_squared_space(
    bound_best_mem: f64,
    last_max_mem: f64,
    _: f64,
    cur_mem: f64,
    num_remaining_nodes: f64,
    num_total_nodes: f64,
) -> f64 {
    ((bound_best_mem + 1.) / (f64::max(cur_mem, last_max_mem) + 1.)).powi(2)
        * 1e-3
        * (1e-3
            + 8.5e-2 * (num_total_nodes + 1.)
                / (num_total_nodes - num_remaining_nodes + 1.))
}

fn create_parametrized_linear_space(
    weights: Weights,
    shifts: Shifts,
) -> impl Fn(f64, f64, f64, f64, f64, f64) -> f64 {
    move |bound_best_mem,
          last_max_mem,
          last_cur_mem,
          cur_mem,
          num_remaining_nodes,
          num_total_nodes| {
        (weights.bound_best_mem * bound_best_mem
            + weights.last_max_mem * last_max_mem
            + weights.last_cur_mem * last_cur_mem
            + shifts.upper_mem)
            / (weights.cur_mem * cur_mem + shifts.cur_mem)
            * (shifts.time
                + (weights.num_total_nodes * num_total_nodes + shifts.num_total_nodes)
                    / (weights.num_measure_nodes
                        * (num_total_nodes - num_remaining_nodes)
                        + shifts.num_measure_nodes))
    }
}

/// The weights for the parametrized accept function [AcceptFunc::ParametrizedLinearSpace].
#[derive(Clone)]
pub struct Weights {
    pub bound_best_mem: f64,
    pub last_max_mem: f64,
    pub last_cur_mem: f64,
    pub cur_mem: f64,
    pub num_measure_nodes: f64,
    pub num_total_nodes: f64,
}

/// The shifts for the parametrized accept function
/// [AcceptFunc::ParametrizedLinearSpace]
#[derive(Clone)]
pub struct Shifts {
    pub upper_mem: f64,
    pub cur_mem: f64,
    pub time: f64,
    pub num_measure_nodes: f64,
    pub num_total_nodes: f64,
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
    /// return (bound_best_mem + 1.) / (cur_mem + 1.)
    ///     * (1e-3
    ///         + 8.5e-2 * (num_total_nodes + 1.)
    ///             / (num_total_nodes - num_remaining_nodes + 1.));
    /// ```
    BuiltinLinearSpace,
    BuiltinExponentialSpace,
    BuiltinSquaredSpace,
    /// A parametrized version of [BuiltinBasic](AcceptFunc::BuiltinBasic). Following
    /// [AcceptFn], this function is defined as
    /// ```ignore
    /// return (weights.bound_best_mem * bound_best_mem
    ///     + weights.last_max_mem * last_max_mem
    ///     + weights.last_cur_mem * last_cur_mem
    ///     + shifts.upper_mem)
    ///     / (weights.cur_mem * cur_mem + shifts.cur_mem)
    ///     * (shifts.time
    ///         + (weights.num_total_nodes * num_total_nodes + shifts.num_total_nodes)
    ///             / (weights.num_measure_nodes * (num_total_nodes - num_remaining_nodes)
    ///                 + shifts.num_measure_nodes));
    /// ```
    ParametrizedLinearSpace {
        weights: Weights,
        shifts: Shifts,
    },
    /// A custom accept function.
    Custom(AcceptFn),
}

impl AcceptFunc {
    /// Returns the underlying accept function.
    pub fn get_accept_func(self) -> AcceptFn {
        match self {
            AcceptFunc::BuiltinLinearSpace => Box::new(builtin_linear_space),
            AcceptFunc::BuiltinExponentialSpace => Box::new(builtin_exponential_space),
            AcceptFunc::BuiltinSquaredSpace => Box::new(builtin_squared_space),
            AcceptFunc::ParametrizedLinearSpace { weights, shifts } => {
                Box::new(create_parametrized_linear_space(weights, shifts))
            },
            AcceptFunc::Custom(f) => f,
        }
    }
}
