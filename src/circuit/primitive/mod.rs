mod leafcell;
mod stdcell;

use std::sync::{Arc, RwLock};

pub use leafcell::*;
pub use stdcell::*;
use reda_sp::Subckt;
use super::{Design, Shr};

pub trait Primitive : Design + Send + Sync {
    fn netlist(&self) -> &Subckt;
}

impl Into<Shr<dyn Primitive>> for Shr<LogicGate> {
    fn into(self) -> Shr<dyn Primitive> {
        let inner = self.inner();
        let inner: Arc<RwLock<dyn Primitive>> = inner;
        Shr::from_inner(inner)
    }
}

impl Into<Shr<dyn Primitive>> for Shr<Dff> {
    fn into(self) -> Shr<dyn Primitive> {
        let inner = self.inner();
        let inner: Arc<RwLock<dyn Primitive>> = inner;
        Shr::from_inner(inner)
    }
}

impl Into<Shr<dyn Primitive>> for Shr<Leafcell> {
    fn into(self) -> Shr<dyn Primitive> {
        let inner = self.inner();
        let inner: Arc<RwLock<dyn Primitive>> = inner;
        Shr::from_inner(inner)
    }
}
