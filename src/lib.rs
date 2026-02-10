#[cfg(feature = "python")]
mod python;

mod capi;

pub use capi::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn pybinwalk(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    python::register(m)
}
