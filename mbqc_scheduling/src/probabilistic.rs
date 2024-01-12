/// last_cur_mem, last_max_mem, cur_mem, num_remaining_nodes, num_total_nodes
pub type AcceptFn = Box<dyn Fn(f64, f64, f64, f64, f64) -> f64 + Send + Sync>;

pub fn standard_accept_func(
    last_max_mem: f64,
    _: f64,
    cur_mem: f64,
    num_remaining_nodes: f64,
    num_total_nodes: f64,
) -> f64 {
    (last_max_mem + 1.) / (cur_mem + 1.)
        * (1e-3
            + 1.3e-1 * (num_total_nodes + 1.)
                / (num_total_nodes - num_remaining_nodes + 1.))
}

#[derive(Clone)]
pub struct Weights {
    pub last_max_mem: f64,
    pub last_cur_mem: f64,
    pub cur_mem: f64,
    pub num_measure_nodes: f64,
    pub num_total_nodes: f64,
}

#[derive(Clone)]
pub struct Shifts {
    pub last_mem: f64,
    pub cur_mem: f64,
    pub time: f64,
    pub num_measure_nodes: f64,
    pub num_total_nodes: f64,
}

pub enum AcceptFunc {
    Standard,
    CreateFunc { weights: Weights, shifts: Shifts },
    Custom(AcceptFn),
}

impl AcceptFunc {
    pub fn get_accept_func(self) -> AcceptFn {
        match self {
            AcceptFunc::Standard => Box::new(standard_accept_func),
            AcceptFunc::CreateFunc { weights, shifts } => Box::new(
                move |last_max_mem,
                      last_cur_mem,
                      cur_mem,
                      num_remaining_nodes,
                      num_total_nodes| {
                    (weights.last_max_mem * last_max_mem
                        + weights.last_cur_mem * last_cur_mem
                        + shifts.last_mem)
                        / (weights.cur_mem * cur_mem + shifts.cur_mem)
                        * (shifts.time
                            + (weights.num_total_nodes * num_total_nodes
                                + shifts.num_total_nodes)
                                / (weights.num_measure_nodes
                                    * (num_total_nodes - num_remaining_nodes)
                                    + shifts.num_measure_nodes))
                },
            ),
            AcceptFunc::Custom(f) => f,
        }
    }
}
