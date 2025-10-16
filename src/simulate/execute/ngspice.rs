use std::path::Path;
use super::Execute;

pub struct NgSpice;

impl Execute for NgSpice {
    fn simulate_command(sim_filepath: impl AsRef<Path>, temp_folder: impl AsRef<Path>) -> anyhow::Result<String> {
        let sim_filepath = sim_filepath.as_ref();
        let temp_folder = temp_folder.as_ref();

        Ok(format!(
            "ngspice -b -o {} {} > /dev/null 2>&1",
            Self::meas_result_filepath(sim_filepath, temp_folder)?.display(),
            sim_filepath.display()
        ))
    }
}