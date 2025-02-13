use pyo3::prelude::*;

mod vfs;
mod utility;
mod parser;
use crate::vfs::unionfs::*;

//This is the file that defines functions exposed to Python via the internal vanguard module.
//It is not intended to be used directly by the end user and is meant to facilitate the creation of an extensable frontend for Arsenal.

//I make no promises about the elegance of this code or anything else written in Rust, but I can assure you that it is faster than the Python equivalent.


/*Strike the first rune upon the engine's casing employing the chosen wrench.
Its tip should be anointed with the oil of engineering using the proper incantation when the auspices are correct.
Strike the second rune upon the engine's casing employing the arc-tip of the power-driver.
If the second rune is not good, a third rune may be struck in like manner to the first.
This is done according to the true ritual laid down by Scotti the Enginseer.
A libation should be offered.
If this sequence is properly observed the engines may be brought to full activation by depressing the large panel marked "ON".*/

/// A Python module implemented in Rust.
#[pymodule]
fn vanguard(m: &Bound<'_, PyModule>) -> PyResult<()> {
    //m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<Vfs>()?;
    Ok(())
}

// Formats the sum of two numbers as string.
//#[pyfunction]
//fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//    Ok((a + b).to_string())
//}