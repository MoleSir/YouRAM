mod ngspice;
mod spectre;
pub use ngspice::*;
use reda_unit::Number;
pub use spectre::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::{ErrorContext, YouRAMResult};
use super::error::SimulateError;
use super::Meas;

pub struct SpiceExector {
    pub simulate_path: PathBuf,
    pub measurements: Vec<Box<dyn Meas>>,
}

impl SpiceExector {
    pub fn simulate(&mut self, execute: impl ExecuteCommand, temp_folder: &Path) -> YouRAMResult<HashMap<String, Number>> {
        let result_path = execute.execute(&self.simulate_path, temp_folder).context("Execute simualte")?;
        self.get_meas_results(&result_path).context("Get meas result")
    }   

    fn get_meas_results(&mut self, result_path: &Path) -> YouRAMResult<HashMap<String, Number>> {
        let content = std::fs::read_to_string(result_path).context(format!("read result file '{:?}'", result_path))?;

        let mut results = HashMap::new();
        for meas in self.measurements.iter() {
            let value = meas.get_result(&content).map_err(SimulateError::MeasError)?;
            results.insert(meas.name().to_string(), value);
        }

        Ok(results)
    }
}

pub trait ExecuteCommand {
    /// Return the simulate command to execute 
    fn simulate_command(&self, sim_filepath: &Path, temp_folder: &Path) -> YouRAMResult<String>;

    /// Return the meas filepath after simulate
    fn meas_result_filepath(&self, sim_filepath: &Path, temp_folder: &Path) -> YouRAMResult<PathBuf> {
        let temp_folder = temp_folder.to_path_buf();

        let filename = sim_filepath
            .file_name()
            .ok_or_else(|| SimulateError::InvalidPath(sim_filepath.to_path_buf()))?;
        let mut filename = PathBuf::from(filename);
        filename.set_extension("meas");

        Ok(temp_folder.join(filename)) 
    }

    fn execute(&self, sim_filepath: &Path, temp_folder: &Path) -> YouRAMResult<PathBuf> {
        let sim_filepath = sim_filepath.as_ref();
        let temp_folder = temp_folder.as_ref();

        let command = self.simulate_command(sim_filepath, temp_folder)?;
        let status = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .status()
            .map_err(|e| SimulateError::ExecuteError(command.clone(), e.to_string()))?;

        match status.code() {
            Some(0) => Ok(self.meas_result_filepath(sim_filepath, temp_folder)?),
            Some(code) => Err(SimulateError::ExecuteError(command.clone(), format!("Command returns '{}'", code)))?,
            None => Err(SimulateError::ExecuteError(command.clone(), "Command quit unnormal".into()))?,
        }
    }
}

impl ExecuteCommand for Box<dyn ExecuteCommand> {
    fn simulate_command(&self, sim_filepath: &Path, temp_folder: &Path) -> YouRAMResult<String> {
        self.as_ref().simulate_command(sim_filepath, temp_folder)
    }
}