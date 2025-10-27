use std::path::Path;
use crate::YouRAMResult;
use super::SpiceCommand;

#[derive(Clone)]
pub struct NgSpice;

impl SpiceCommand for NgSpice {
    fn simulate_command(&self, sim_filepath: &Path, temp_folder: &Path) -> YouRAMResult<String> {
        let sim_filepath = sim_filepath.as_ref();
        let temp_folder = temp_folder.as_ref();

        Ok(format!(
            "ngspice -b -o {} {} > /dev/null 2>&1",
            self.meas_result_filepath(sim_filepath, temp_folder)?.display(),
            sim_filepath.display()
        ))
    }
}