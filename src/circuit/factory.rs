use std::{collections::HashMap, fmt::Debug};
use std::sync::Arc;
use anyhow::Context;
use tracing::info;
use crate::pdk::Pdk;
use super::{Circuit, CircuitError, DriveStrength, Module, Shr, ShrString, StdcellKind};

pub trait ModuleArg: Sized + Debug {
    fn module_name(&self) -> ShrString;
    fn create_module(self, factory: &mut CircuitFactory) -> anyhow::Result<Module<Self>>;
}

pub struct CircuitFactory {
    pub pdk: Arc<Pdk>,
    modules: HashMap<ShrString, Shr<Circuit>>,
}

impl CircuitFactory {
    pub fn new(pdk: Arc<Pdk>) -> Self {
        Self {
            pdk,
            modules: HashMap::new(),
        }
    }

    pub fn module<A: ModuleArg + 'static>(&mut self, arg: A) -> anyhow::Result<Shr<Circuit>> {
        let name = arg.module_name();
        if let Some(circuit) = self.modules.get(&name) {
            Ok(circuit.clone())
        } else {
            info!("create circuit '{}'", name);
            let module = arg
                .create_module(self)
                .context(format!("create circuit '{}'", name))?;
            let module = Shr::new(Circuit::Module(Box::new(module)));
            self.modules.insert(name, module.clone());
            Ok(module)
        }
    }

    pub fn stdcell(&self, kind: StdcellKind, drive_strength: DriveStrength) -> Result<Shr<Circuit>, CircuitError> {
        self.pdk.get_stdcell(kind, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(kind, drive_strength))
    }

    pub fn and(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<Circuit>, CircuitError> {
        self.pdk.get_and(input_size, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::And(input_size), drive_strength))
    }

    pub fn nand(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<Circuit>, CircuitError> {
        self.pdk.get_nand(input_size, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::Nand(input_size), drive_strength))
    }

    pub fn or(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<Circuit>, CircuitError> {
        self.pdk.get_or(input_size, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::Or(input_size), drive_strength))
    }

    pub fn nor(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<Circuit>, CircuitError> {
        self.pdk.get_or(input_size, drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::Nor(input_size), drive_strength))
    }

    pub fn inv(&self, drive_strength: DriveStrength) -> Result<Shr<Circuit>, CircuitError> {
        self.pdk.get_inv(drive_strength)
            .ok_or_else(|| CircuitError::StdcellNotFound(StdcellKind::Inv, drive_strength))
    }

    pub fn bitcell(&self) -> Shr<Circuit> {
        self.pdk.get_bitcell().clone()
    }

    pub fn sens_amp(&self) -> Shr<Circuit> {
        self.pdk.get_sense_amp().clone()
    }

    pub fn column_tri_gate(&self) -> Shr<Circuit> {
        self.pdk.get_column_tri_gate().clone()
    }

    pub fn write_driver(&self) -> Shr<Circuit> {
        self.pdk.get_write_driver().clone()
    }
}