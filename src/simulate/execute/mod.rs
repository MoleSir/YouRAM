mod ngspice;
mod spectre;
pub use ngspice::*;
pub use spectre::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::YouRAMResult;
use super::error::SimulateError;

pub enum Executer {
    Ngspice,
    Spectre,
}

impl Executer {
    pub fn from(execute: impl AsRef<str>) -> YouRAMResult<Self> {
    let execute = execute.as_ref();
        match execute {
            "ngspice" => Ok(Self::Ngspice),
            "spectre" => Ok(Self::Spectre),
            _ => Err(SimulateError::UnsupportExecute(execute.to_string()))?,
        }
    }

    pub fn execute(&self, sim_filepath: impl AsRef<Path>, temp_folder: impl AsRef<Path>) -> YouRAMResult<PathBuf> {
        match self {
            Self::Ngspice => NgSpice::execute(sim_filepath, temp_folder),
            Self::Spectre => Spectre::execute(sim_filepath, temp_folder),
        }
    }
}

pub trait Execute {
    /// Return the simulate command to execute 
    fn simulate_command(sim_filepath: impl AsRef<Path>, temp_folder: impl AsRef<Path>) -> YouRAMResult<String>;

    /// Return the meas filepath after simulate
    fn meas_result_filepath(sim_filepath: impl AsRef<Path>, temp_folder: impl AsRef<Path>) -> YouRAMResult<PathBuf> {
        let sim_filepath = sim_filepath.as_ref();
        let temp_folder = temp_folder.as_ref().to_path_buf();

        let filename = sim_filepath
            .file_name()
            .ok_or_else(|| SimulateError::InvalidPath(sim_filepath.to_path_buf()))?;
        let mut filename = PathBuf::from(filename);
        filename.set_extension("meas");

        Ok(temp_folder.join(filename)) 
    }

    fn execute(sim_filepath: impl AsRef<Path>, temp_folder: impl AsRef<Path>) -> YouRAMResult<PathBuf> {
        let sim_filepath = sim_filepath.as_ref();
        let temp_folder = temp_folder.as_ref();

        let command = Self::simulate_command(sim_filepath, temp_folder)?;
        let status = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .status()
            .map_err(|e| SimulateError::ExecuteError(command.clone(), e.to_string()))?;

        match status.code() {
            Some(0) => Ok(Self::meas_result_filepath(sim_filepath, temp_folder)?),
            Some(code) => Err(SimulateError::ExecuteError(command.clone(), format!("Command returns '{}'", code)))?,
            None => Err(SimulateError::ExecuteError(command.clone(), "Command quit unnormal".into()))?,
        }
    }
}