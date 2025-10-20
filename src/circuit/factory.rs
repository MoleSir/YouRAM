use std::any::{Any, TypeId};
use std::{collections::HashMap, fmt::Debug};
use std::sync::{Arc, RwLock};
use tracing::info;
use crate::pdk::Pdk;
use crate::{ErrorContext, YouRAMResult};
use super::{CircuitError, Dff, DriveStrength, Leafcell, LogicGate, LogicGateKind, Module, Shr, ShrString};

pub trait ModuleArg: Sized + Debug + Send + Sync {
    fn module_name(&self) -> ShrString;
    fn create_module(self, factory: &mut CircuitFactory) -> YouRAMResult<Module<Self>>;
}

pub struct CircuitFactory {
    pub pdk: Arc<Pdk>,
    modules: HashMap<TypeId, HashMap<ShrString, Arc<dyn Any + Send + Sync>>>,
}

impl CircuitFactory {
    pub fn new(pdk: Arc<Pdk>) -> Self {
        Self {
            pdk,
            modules: HashMap::new(),
        }
    }

    pub fn module<A: ModuleArg + 'static>(&mut self, arg: A) -> YouRAMResult<Shr<Module<A>>> {
        let key = TypeId::of::<Module<A>>();
        let entry = self.modules.entry(key);
        let modules = entry.or_default();
        
        let name = arg.module_name();
        match modules.get(&name) {
            Some(module) => {
                let inner = module.clone().downcast_arc::<RwLock<Module<A>>>().unwrap();
                Ok(Shr::from_inner(inner))
            }
            None => {
                info!("create circuit '{}'", name);
                let module = arg.create_module(self).context(format!("create circuit '{}'", name))?;
                let module = Arc::new(RwLock::new(module));
                let entry = self.modules.entry(key);
                let modules = entry.or_default();
                modules.insert(name, module.clone());
                Ok(Shr::from_inner(module))
            }
        } 
    }

    pub fn logicgate(&self, kind: LogicGateKind, drive_strength: DriveStrength) -> Result<Shr<LogicGate>, CircuitError> {
        self.pdk.get_logicgate(kind, drive_strength)
            .ok_or_else(|| CircuitError::LogicGateNotFound(kind, drive_strength))
    }

    pub fn and(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<LogicGate>, CircuitError> {
        self.pdk.get_and(input_size, drive_strength)
            .ok_or_else(|| CircuitError::LogicGateNotFound(LogicGateKind::And(input_size), drive_strength))
    }

    pub fn nand(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<LogicGate>, CircuitError> {
        self.pdk.get_nand(input_size, drive_strength)
            .ok_or_else(|| CircuitError::LogicGateNotFound(LogicGateKind::Nand(input_size), drive_strength))
    }

    pub fn or(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<LogicGate>, CircuitError> {
        self.pdk.get_or(input_size, drive_strength)
            .ok_or_else(|| CircuitError::LogicGateNotFound(LogicGateKind::Or(input_size), drive_strength))
    }

    pub fn nor(&self, input_size: usize, drive_strength: DriveStrength) -> Result<Shr<LogicGate>, CircuitError> {
        self.pdk.get_or(input_size, drive_strength)
            .ok_or_else(|| CircuitError::LogicGateNotFound(LogicGateKind::Nor(input_size), drive_strength))
    }

    pub fn inv(&self, drive_strength: DriveStrength) -> Result<Shr<LogicGate>, CircuitError> {
        self.pdk.get_inv(drive_strength)
            .ok_or_else(|| CircuitError::LogicGateNotFound(LogicGateKind::Inv, drive_strength))
    }

    pub fn dff(&self, drive_strength: DriveStrength) -> Result<Shr<Dff>, CircuitError> {
        self.pdk.get_dff(drive_strength)
            .ok_or_else(|| CircuitError::DffNotFound(drive_strength))
    }

    pub fn bitcell(&self) -> Shr<Leafcell> {
        self.pdk.get_bitcell()
    }

    pub fn sense_amp(&self) -> Shr<Leafcell> {
        self.pdk.get_sense_amp()
    }

    pub fn column_trigate(&self) -> Shr<Leafcell> {
        self.pdk.get_column_trigate()
    }

    pub fn write_driver(&self) -> Shr<Leafcell> {
        self.pdk.get_write_driver()
    }

    pub fn precharge(&self) -> Shr<Leafcell> {
        self.pdk.get_precharge()
    }
}

trait DowncastArc {
    fn downcast_arc<T: Any + Send + Sync>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>>;
}

impl DowncastArc for dyn Any + Send + Sync {
    fn downcast_arc<T: Any + Send + Sync>(self: Arc<Self>) -> Result<Arc<T>, Arc<Self>> {
        if self.is::<T>() {
            let ptr = Arc::into_raw(self) as *const T;
            Ok(unsafe { Arc::from_raw(ptr) })
        } else {
            Err(self)
        }
    }
}
