use crate::circuit::{CircuitError, Design, Pin, Shr, ShrCircuit, ShrString};
use super::Net;

pub struct Instance {
    pub name: ShrString,
    pub template_circuit: ShrCircuit,
    pub pins: Vec<Shr<Pin>>,
}

impl Instance {
    pub fn new<S, C>(name: S, template_circuit: Shr<C>) -> Shr<Instance> 
    where 
        S: Into<ShrString>,
        C: Design,
        Shr<C>: Into<ShrCircuit>,
    {
        let pins = template_circuit.read().ports().iter().map(|port| {
            Pin::new(port.read().name.clone(), port.clone())
        })
        .collect();

        let template_circuit: ShrCircuit = template_circuit.into();
        let name: ShrString = name.into();
        
        Shr::new ( Self { name: name.into(), template_circuit, pins } )
    }

    pub fn get_pin(&self, name: &str) -> Option<Shr<Pin>> {
        for pin in self.pins.iter() {
            if pin.read().name == name {
                return Some(pin.clone());
            }
        }
        None
    }

    pub fn connect_nets(&mut self, nets: &[Shr<Net>]) -> Result<(), CircuitError> {
        if self.pins.len() != nets.len() {
            Err(CircuitError::PinSizeUnmatch(self.pins.len(), nets.len()))
        } else {
            for (pin, net) in self.pins.iter().zip(nets.iter()) {
                pin.wrire().net = Some(net.clone());
            }
            Ok(())
        }
    }

    pub fn connect_net(&mut self, pin_name: &str, net: Shr<Net>) {
        for pin in self.pins.iter_mut() {
            if pin.read().name == pin_name {
                pin.wrire().net = Some(net);
                break;
            }
        }
    }
}