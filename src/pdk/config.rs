use std::{collections::HashMap, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use crate::{ErrorContext, YouRAMResult};
use super::Process;

pub const PDK_CONFIG: &'static str = "config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct PdkConfig {
    #[serde(skip)]
    pub pdk_path: PathBuf,

    pub stdcell_liberty: PathBuf,
    pub stdcell_spice: PathBuf,
    pub leafcell_spice: PathBuf,
    pub models: HashMap<Process, PdkModelPath>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdkModelPath {
    pub nmos: PathBuf,
    pub pmos: PathBuf, 
}

impl PdkConfig {
    pub fn load<P: AsRef<Path>>(pdk_path: P) -> YouRAMResult<Self> {
        let pdk_path: &Path = pdk_path.as_ref();
        let config_path = pdk_path.join(PDK_CONFIG);
        let config_content = std::fs::read_to_string(config_path).context("read pdk config")?;
        let mut config: PdkConfig = serde_json::from_str(&config_content).context("parse pdk config")?;
        config.pdk_path = pdk_path.into();

        Ok(config)
    }

    pub fn nmos_model_path(&self, process: Process) -> Option<PathBuf> {
        let models = self.models.get(&process)?;
        Some(self.json_to_pdk(&models.nmos))
    }

    pub fn pmos_model_path(&self, process: Process) -> Option<PathBuf> {
        let models = self.models.get(&process)?;
        Some(self.json_to_pdk(&models.pmos))
    }


    pub fn stdcell_liberty_path(&self) -> PathBuf {
        self.json_to_pdk(&self.stdcell_liberty)
    }

    pub fn stdcell_spice_path(&self) -> PathBuf {
        self.json_to_pdk(&self.stdcell_spice)
    }

    pub fn leafcell_spice_path(&self) -> PathBuf {
        self.json_to_pdk(&self.leafcell_spice)
    }

    #[inline]
    fn json_to_pdk(&self, sub_path: impl AsRef<Path>) -> PathBuf {
        self.pdk_path.join(sub_path.as_ref())
    }
}