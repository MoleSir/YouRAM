mod buffer;
mod decoder;
mod controllogic;
mod bitcellarray;
mod bitcellarrayrec;
pub use buffer::*;
pub use decoder::*;
pub use controllogic::*;
pub use bitcellarray::*;
pub use bitcellarrayrec::*;
use tracing::debug;

use std::{collections::{HashMap, HashSet}, mem::MaybeUninit, ops::Deref};
use crate::{YouRAMResult, ErrorContext};

use super::{CircuitError, CircuitFactory, Design, DriveStrength, Instance, Leafcell, ModuleArg, Net, Pin, Port, PortDirection, Shr, ShrCircuit, ShrString, Stdcell, StdcellKind};

pub trait Modular: Design {
    fn instances(&self) -> &[Shr<Instance>];
    fn sub_modules(&self) -> &HashSet<ShrModule>;
    fn sub_stdcells(&self) -> &HashSet<Shr<Stdcell>>;
    fn sub_leafcells(&self) -> &HashSet<Shr<Leafcell>>;
}

pub struct Module<A> {
    pub name: ShrString,
    pub ports: Vec<Shr<Port>>,
    pub instances: Vec<Shr<Instance>>,
    
    pub sub_modules: HashSet<ShrModule>,
    pub sub_stdcells: HashSet<Shr<Stdcell>>,
    pub sub_leafcells: HashSet<Shr<Leafcell>>,

    pub nets: HashMap<ShrString, Shr<Net>>,
    pub args: A,
}

pub type ShrModule = Shr<Box<dyn Modular>>;

pub trait AsInstance<A> : Clone {
    fn as_instance(self, module: &Module<A>) -> Result<Shr<Instance>, CircuitError>;
}

pub trait AsPin : Clone {
    fn as_pin(self, instance: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError>;
}

macro_rules! impl_link_instance {
    ($fn_name:ident, $factory_fn:ident, [$($port:ident),+]) => {
        pub fn $fn_name(
            &mut self,
            factory: &mut CircuitFactory,
            name: impl Into<ShrString>,
            $($port: impl Into<ShrString>),+
        ) -> YouRAMResult<Shr<Instance>> {
            let name: ShrString = name.into();
            (|| -> YouRAMResult<Shr<Instance>> {
                let cell = factory.$factory_fn();
                self.sub_leafcells.insert(cell.clone());
                let instance = self.add_instance(name.clone(), cell)?;
                self.connect_instance(instance.clone(), [$($port.into()),+].into_iter())?;
                Ok(instance)
            })()
            .with_context(|| format!("link leafcell {} to circuit {}", name, self.name))
        }
    };
}

impl<A> Module<A> {
    pub fn new<S: Into<ShrString>>(name: S, args: A) -> Self {
        Self {
            name: name.into(), 
            ports: Vec::new(),
            instances: Vec::new(),
            sub_modules: HashSet::new(),
            sub_stdcells: HashSet::new(),
            sub_leafcells: HashSet::new(),
            nets: HashMap::new(),
            args
        }
    }

    pub fn add_port<S: Into<ShrString>>(&mut self, name: S, direction: PortDirection) -> YouRAMResult<Shr<Port>> {
        let name: ShrString = name.into();
        debug!("add port {} to circuit {}", name, self.name);
        
        (|| {
            if self.ports.iter().any(|port| port.read().name == name) {
                Err(CircuitError::AddDuplicatePort(name.to_string()))
            } else {
                // add a port and net with the same name
                let port = Port::new(name.clone(), direction);
                let net = self.add_net(port.read().name.clone());
                port.wrire().set_connected_net(net.clone());
                net.wrire().add_connection(port.clone());
                self.ports.push(port.clone());
                Ok(port)
            }
        })()
        .with_context(|| format!("add port {} to circuit {}", name, self.name))
    }

    pub fn add_module(&mut self, arg: impl ModuleArg + 'static, factory: &mut CircuitFactory) -> YouRAMResult<Shr<Box<dyn Modular>>> {   
        (|| -> YouRAMResult<Shr<Box<dyn Modular>>> {
            debug!("add sub module to circuit {}", self.name);
            let module = factory.module(arg)?;
            self.sub_modules.insert(module.clone());
            Ok(module) 
        })()  
        .with_context(|| format!("add sub module to circuit {}", self.name))
    }

    pub fn add_stdcell(&mut self, kind: StdcellKind, drive_strength: DriveStrength, factory: &mut CircuitFactory) -> YouRAMResult<Shr<Stdcell>> {
        (|| -> YouRAMResult<Shr<Stdcell>> {
            debug!("add stdcell {}, {} to circuit {}", kind, drive_strength, self.name);
            let stdcell = factory.stdcell(kind, drive_strength)?;
            self.sub_stdcells.insert(stdcell.clone());
            Ok(stdcell)
        })()  
        .with_context(|| format!("add stdcell ({},{}) to circuit {}", kind, drive_strength, self.name))
    }

    pub fn add_instance<S, C>(&mut self, name: S, template_circuit: Shr<C>) -> YouRAMResult<Shr<Instance>> 
    where 
        S: Into<ShrString>,
        C: Design,
        Shr<C>: Into<ShrCircuit>,
    {
        let name: ShrString = name.into();
        debug!("add instance {} to circuit {}", name, self.name);

        (|| {    
            if self.instances.iter().any(|inst| inst.read().name == name) {
                return Err(CircuitError::AddDuplicateInstance(name.to_string()));
            }
    
            // Create a new instance
            let instance = Instance::new(name.clone(), template_circuit);
            self.instances.push(instance.clone());
            
            Ok(instance)
        })()
        .with_context(|| format!("add instance {} to circuit {}", name, self.name))
    }

    impl_link_instance!(link_bitcell_instance, bitcell, [bl, br, wl, vdd, gnd]);
    impl_link_instance!(link_senseamp_instance, sense_amp, [bl, br, dout, en, vdd, gnd]);
    impl_link_instance!(link_writedriver_instance, write_driver, [din, bl, br, en, vdd, gnd]);
    impl_link_instance!(link_column_trigate_instance, column_trigate, [bl_in, br_in, bl_out, br_out, sel, vdd, gnd]);

    pub fn link_inv_instance(
        &mut self,
        name: impl Into<ShrString>, 
        stdcell: Shr<Stdcell>, 
        nets: [impl Into<ShrString>; 4]
    ) -> YouRAMResult<Shr<Instance>> {
        let [input, output, vdd, gnd] = nets;
        self.link_stdcell_instance(name, stdcell, vec![input], output, vdd, gnd)
    }

    pub fn link_stdcell_instance(
        &mut self, 
        name: impl Into<ShrString>, 
        stdcell: Shr<Stdcell>, 
        input_nets: Vec<impl Into<ShrString>>, 
        output_net: impl Into<ShrString>,
        vdd_net: impl Into<ShrString>,
        gnd_net: impl Into<ShrString>,
    ) -> YouRAMResult<Shr<Instance>> {
        let name: ShrString = name.into();
        (|| -> YouRAMResult<Shr<Instance>>  {
            let stdcell_ref = stdcell.read();
            let expect_input_len = stdcell_ref.input_port_indexs.len();
            let port_len = expect_input_len + 3;

            if expect_input_len != input_nets.len() {
                return Err(CircuitError::StdcellInputPinSizeUnmatch(expect_input_len, input_nets.len()))?;
            }

            let instance = self.add_instance(name.clone(), stdcell.clone())?;

            let mut nets: Vec<MaybeUninit<ShrString>> = Vec::with_capacity(port_len);
            for _ in 0..port_len {
                nets.push(MaybeUninit::uninit());
            }

            for (input_index, input_net) in input_nets.into_iter().enumerate() {
                let idx = stdcell_ref.input_port_indexs[input_index];
                unsafe { nets.get_unchecked_mut(idx).write(input_net.into()); }
            }
            
            unsafe { nets.get_unchecked_mut(stdcell_ref.output_port_index).write(output_net.into()); }
            unsafe { nets.get_unchecked_mut(stdcell_ref.vdd_port_index).write(vdd_net.into()); }
            unsafe { nets.get_unchecked_mut(stdcell_ref.gnd_port_index).write(gnd_net.into()); }
            
            let nets = unsafe {
                std::mem::transmute::<Vec<MaybeUninit<ShrString>>, Vec<ShrString>>(nets)
            };
            
            self.connect_instance(instance.clone(), nets.into_iter())?;

            Ok(instance)
        })()
        .with_context(|| format!("connect stdcell instance {} to circuit {}", name, self.name))
    }

    pub fn connect_instance<'a, T, S, I>(&mut self, instance: T, nets: I) -> YouRAMResult<()> 
    where 
        T: AsInstance<A>,
        S: Into<ShrString>,
        I: ExactSizeIterator<Item = S>,
    {        
        let instance = instance.as_instance(self)?;
        (|| -> YouRAMResult<()>  {
            if instance.read().pins.len() != nets.len() {
                Err(CircuitError::PinSizeUnmatch(instance.read().pins.len(), nets.len()))?;
            }
            
            debug!("connect instance {} to circuit {}", instance.read().name, self.name);
            for (pin, net) in instance.read().pins.iter().zip(nets) {
                self.connect_pin_with_net(instance.clone(), pin, net)?;
            }
    
            Ok(())
        })()
        .with_context(|| format!("connect instance {} to circuit {}", instance.read().name, self.name))
    }

    pub fn connect_instance_with_map<'a, T, P, S, I>(&mut self, instance: T, pin_to_nets: I) -> YouRAMResult<()> 
    where 
        T: AsInstance<A>,
        P: AsPin,
        S: Into<ShrString>,
        I: ExactSizeIterator<Item = (P, S)>,
    {
        let instance = instance.as_instance(self)?;
        (|| -> YouRAMResult<()>  {
            if instance.read().pins.len() != pin_to_nets.len() {
                Err(CircuitError::PinSizeUnmatch(instance.read().pins.len(), pin_to_nets.len()))?;
            }

            debug!("connect instance {} to circuit {}", instance.read().name, self.name);
            for (pin, net) in pin_to_nets {
                self.connect_pin_with_net(instance.clone(), pin, net)?;
            }
            Ok(())
        })()
        .with_context(|| format!("connect instance {} to circuit {}", instance.read().name, self.name))
    }

    pub fn connect_pin_with_net(&mut self, instance: impl AsInstance<A>, pin: impl AsPin, net: impl Into<ShrString>) -> Result<Shr<Net>, CircuitError> {
        let instance = instance.as_instance(self)?;
        
        let pin = pin.as_pin(&instance)?;
        let net = self.add_net(net);
        
        debug!("connect pin {} with net {}", pin.read().name, net.read().name);

        pin.wrire().set_connected_net(net.clone());
        net.wrire().add_connection(pin.clone());

        Ok(net)
    }

    pub fn add_net<S: Into<ShrString>>(&mut self, name: S) -> Shr<Net> {
        let name_str = name.into();
        if let Some(net) = self.nets.get(&name_str) {
            net.clone()
        } else {
            let net = Net::new(name_str.clone());
            self.nets.insert(name_str, net.clone());
            net
        }
    }
}

impl<A> AsInstance<A> for Shr<Instance> {
    fn as_instance(self, _: &Module<A>) -> Result<Self, CircuitError> {
        // Check instance
        Ok(self)
    }
} 

impl<A> AsInstance<A> for &str {
    fn as_instance(self, module: &Module<A>) -> Result<Shr<Instance>, CircuitError> {
        module.instances.iter()
            .find(|instance| instance.read().name == self)
            .cloned()
            .ok_or_else(|| CircuitError::InstanceNotFound(self.to_string(), module.name.to_string()))
    }
}

impl AsPin for &str {
    fn as_pin(self, instance: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError> {
        instance.read()
            .get_pin(self)
            .ok_or_else(|| CircuitError::PinNotFound(self.to_string(), instance.read().name.to_string()))
    }
}

impl AsPin for Shr<Pin> {
    fn as_pin(self, _: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError> {
        // check self
        Ok(self)
    }
}

impl AsPin for &Shr<Pin> {
    fn as_pin(self, _: &Shr<Instance>) -> Result<Shr<Pin>, CircuitError> {
        // check self
        Ok(self.clone())
    }
}

impl<A> Design for Module<A> {
    fn name(&self) -> ShrString {
        self.name.clone()
    }

    fn ports(&self) -> &[Shr<Port>] {
        &self.ports
    }
}

impl<A> Modular for Module<A> {
    fn sub_modules(&self) -> &HashSet<ShrModule> {
        &self.sub_modules
    }

    fn sub_stdcells(&self) -> &HashSet<Shr<Stdcell>> {
        &self.sub_stdcells
    }

    fn sub_leafcells(&self) -> &HashSet<Shr<Leafcell>> {
        &self.sub_leafcells
    }

    fn instances(&self) -> &[Shr<Instance>] {
        &self.instances
    }
}

impl Design for Box<dyn Modular> {
    fn name(&self) -> ShrString {
        self.deref().name()
    }

    fn ports(&self) -> &[Shr<Port>] {
        self.deref().ports()   
    }
}