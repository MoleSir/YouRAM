use crate::YouRAMResult;
use super::Execute;
use std::path::Path;

pub struct Spectre;

impl Execute for Spectre {
    fn simulate_command(sim_filepath: impl AsRef<Path>, temp_folder: impl AsRef<Path>) -> YouRAMResult<String> {
        Ok(format!(
            "spectre {} -outdir {} > /dev/null 2>&1",
            sim_filepath.as_ref().display(),
            temp_folder.as_ref().display(),
        ))
    }
}