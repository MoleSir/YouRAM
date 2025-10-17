use std::{collections::HashMap, fmt::Debug};
use std::sync::Arc;
use tracing::info;
use crate::pdk::Pdk;
use crate::{ErrorContext, YouRAMResult};
use super::{CircuitError, DriveStrength, Leafcell, Modular, Module, Shr, ShrString, Stdcell, StdcellKind};

pub trait ModuleArg: Sized + Debug {
    fn module_name(&self) -> ShrString;
    fn create_module(self, factory: &mut CircuitFactory) -> YouRAMResult<Module<Self>>;
}

pub struct CircuitFactory {
    pub pdk: Arc<Pdk>,
    modules: HashMap<ShrString, Shr<Box<dyn Modular>>>,
}

impl CircuitFactory {
    pub fn new(pdk: Arc<Pdk>) -> Self {
        Self {
            pdk,
            modules: HashMap::new(),
        }
    }

    pub fn module<A: ModuleArg + 'static>(&mut self, arg: A) -> YouRAMResult<Shr<Box<dyn Modular>>> {
        let name = arg.module_name();
        if let Some(circuit) = self.modules.get(&name) {
            Ok(circuit.clone())
        } else {
            info!("create circuit '{}'", name);
            let module = arg
                .create_module(self)
                .context(format!("create circuit '{}'", name))?;
            let module: Shr<Box<dyn Modular>> = Shr::new(Box::new(module));
            self.modules.insert(name, module.clone());
            Ok(module)
        }
    }

    pub fn stdcell(&self, kind: StdcellKind, drive_strength: DriveStrength) -> Result<Shr<Stdcell>, CircuitError> {
        self.pdk.get_stdcell(kind, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(kind, drive_strength))
    }

    pub fn and(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<Stdcell>, CircuitError> {
        self.pdk.get_and(input_size, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::And(input_size), drive_strength))
    }

    pub fn nand(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<Stdcell>, CircuitError> {
        self.pdk.get_nand(input_size, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::Nand(input_size), drive_strength))
    }

    pub fn or(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<Stdcell>, CircuitError> {
        self.pdk.get_or(input_size, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::Or(input_size), drive_strength))
    }

    pub fn nor(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<Stdcell>, CircuitError> {
        self.pdk.get_or(input_size, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::Nor(input_size), drive_strength))
    }

    pub fn inv(&self, drive_strength: DriveStrength) -> Result<Shr<Stdcell>, CircuitError> {
        self.pdk.get_inv(drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::Inv, drive_strength))
    }

    pub fn bitcell(&self) -> Shr<Leafcell> {
        self.pdk.get_bitcell().clone()
    }

    pub fn sense_amp(&self) -> Shr<Leafcell> {
        self.pdk.get_sense_amp().clone()
    }

    pub fn column_trigate(&self) -> Shr<Leafcell> {
        self.pdk.get_column_trigate().clone()
    }

    pub fn write_driver(&self) -> Shr<Leafcell> {
        self.pdk.get_write_driver().clone()
    }
}