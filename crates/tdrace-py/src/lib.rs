use pyo3::prelude::*;

pub mod config;
pub mod engine;
pub mod rasterizer;

use config::RewardConfig;
use engine::PyEngine;

#[pymodule]
fn _tdrace(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngine>()?;
    m.add_class::<RewardConfig>()?;
    Ok(())
}
