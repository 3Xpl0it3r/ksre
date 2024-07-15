#[derive(Debug, Clone, Copy)]
pub(crate) enum PState {
    R, // for running
    S, // for sleeping
    D, // for sleeping in an uninterruptible wait
    Z, // zombie
    T, // traced or stopped
    Unknown,
}

// (R is running, S is sleeping, D is sleeping in an uninterruptible wait, Z is zombie
