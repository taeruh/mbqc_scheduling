// TODO: This was originally written to be a generic part of a library, so that one can do
// automatic scheduling and manual scheduling. I don't think that we need this
// flexibiblity anymore, but only do the automatic scheduling. So we should simplify the
// API and make it more consice and possibly more efficient.
// EDIT: Actully, I might be a good idea to keep it generic, because I think we might want
// to change the automatic scheduling in the future (and be flexible with it).

#![doc = include_str!("../xdocs/scheduler.md")]

mod combinatoric;

pub use combinatoric::Partition;
use space::{AlreadyMeasured, Graph};
use time::{MeasurableSet, NotMeasurable, Partitioner, PathGenerator};
use tree::{Focus, FocusIterator, Step, Sweep};

macro_rules! update {
    ($bit:expr, $map:expr) => {
        $map.get($bit).unwrap_or($bit)
    };
    ($bit:expr; $map:expr) => {
        *$bit = *update!($bit, $map);
    };
}

pub mod space;
pub mod time;
pub mod tree;

/// A scheduler to generate allowed paths scheduling paths, capturing the required
/// quantum memory. Compare the [module documentation](crate::scheduler).
// PathGenerator has a reference to the dependency structure and Graph has a reference to
// the spacial structure (DependencyBuffer and GraphBuffer, respectively). Doing that
// enables us to do cheaper clones of the Scheduler
#[derive(Debug, Clone)]
pub struct Scheduler<'l, T> {
    time: PathGenerator<'l, T>,
    space: Graph<'l>,
}

impl<'l, T> Scheduler<'l, T> {
    /// Create a new scheduler.
    pub fn new(time: PathGenerator<'l, T>, space: Graph<'l>) -> Self {
        Self { time, space }
    }

    /// Get a reference to the underlying [PathGenerator].
    pub fn time(&self) -> &PathGenerator<'l, T> {
        &self.time
    }

    /// Get a reference to the underlying [Graph].
    pub fn space(&self) -> &Graph {
        &self.space
    }
}

impl<T: MeasurableSet> Focus<&[usize]> for Scheduler<'_, T> {
    type Error = InstructionError;

    fn focus_inplace(&mut self, measure_set: &[usize]) -> Result<(), Self::Error> {
        self.time.focus_inplace(measure_set)?;
        #[cfg(debug_assertions)]
        self.space.focus_inplace(measure_set)?;
        #[cfg(not(debug_assertions))]
        self.space.focus_inplace_unchecked(measure_set);
        Ok(())
    }

    fn focus(&mut self, measure_set: &[usize]) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let new_time = self.time.focus(measure_set)?;
        #[cfg(debug_assertions)]
        let new_space = self.space.focus(measure_set)?;
        #[cfg(not(debug_assertions))]
        let new_space = self.space.focus_unchecked(measure_set);
        Ok(Self {
            time: new_time,
            space: new_space,
        })
    }
}

impl FocusIterator for Scheduler<'_, Partitioner> {
    type IterItem = Vec<usize>;
    type LeafItem = usize;

    fn next_and_focus(&mut self) -> Option<(Self, Self::IterItem)>
    where
        Self: Sized,
    {
        let (new_time, mess) = self.time.next_and_focus()?;
        // we get the new mess set from time, and since the api does not allow updating
        // time without space, and vice versa, we can just unwrap here
        #[cfg(debug_assertions)]
        let new_space = self.space.focus(&mess).unwrap();
        #[cfg(not(debug_assertions))]
        let new_space = self.space.focus_unchecked(&mess);
        Some((
            Self {
                time: new_time,
                space: new_space,
            },
            mess,
        ))
    }

    fn at_leaf(&self) -> Option<Self::LeafItem> {
        self.time
            .measurable()
            .set()
            .is_empty()
            .then_some(self.space.max_memory())
    }
}

/// An error that can happen when instructing the [Scheduler].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum InstructionError {
    /// See [NotMeasurable].
    #[error(transparent)]
    NotMeasurable(#[from] NotMeasurable),
    /// See [AlreadyMeasured].
    #[error(transparent)]
    AlreadyMeasured(#[from] AlreadyMeasured),
}

#[doc = non_semantic_default!()]
impl Default for InstructionError {
    fn default() -> Self {
        Self::NotMeasurable(NotMeasurable::default())
    }
}

impl<'l> IntoIterator for Scheduler<'l, Partitioner> {
    type Item = Step<Vec<usize>, Option<usize>>;
    type IntoIter = Sweep<Self>;
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}
