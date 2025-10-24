use crate::YouRAMResult;
use super::ExecuteCommand;
use std::path::Path;

#[derive(Clone)]
pub struct Spectre;

impl ExecuteCommand for Spectre {
    fn simulate_command(&self, sim_filepath: &Path, temp_folder: &Path) -> YouRAMResult<String> {
        Ok(format!(
            "spectre {} -outdir {} > /dev/null 2>&1",
            sim_filepath.display(),
            temp_folder.display(),
        ))
    }
}