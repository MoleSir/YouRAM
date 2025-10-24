use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use reda_sp::ToSpice;
use tracing::{debug, info};
use crate::circuit::{CircuitError, ShrCircuit, ShrString};
use crate::YouRAMResult;

pub fn write_spice<P: AsRef<Path>, C: Into<ShrCircuit>>(circuit: C, path: P) -> YouRAMResult<()> {
    let circuit = circuit.into();
    let path = path.as_ref();
    info!("write circuit {} to {:?}", circuit.name(), path);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let mut exported = HashSet::new();
    write_spice_recursive(&mut writer, &circuit, &mut exported)?;
    Ok(())
}


fn write_spice_recursive<W: Write>(
    writer: &mut W,
    circuit: &ShrCircuit,
    exported: &mut HashSet<ShrString>,
) -> YouRAMResult<()> {
    if exported.get(&circuit.name()).is_some() {
        return Ok(());
    }

    match circuit {
        ShrCircuit::Module(module) => {
            let module_ref = module.read();
            debug!("write module {}", module_ref.name());
        
            for sub_circuit in module_ref.sub_circuits() {
                write_spice_recursive(writer, sub_circuit, exported)?;
            }
        
            // .SUBCKT header
            let ports = module_ref.ports();
            let port_names: Vec<_> = ports.iter().map(|p| p.read().name.to_string()).collect();
            writeln!(writer, ".SUBCKT {} {}", module_ref.name(), port_names.join(" "))?;
        
            // instance
            for inst in module_ref.instances() {            
                let inst = inst.read();
                
                let mut pin_nets = Vec::new();
                for pin in inst.pins.iter() {
                    let pin_ref = pin.read();
                    match &pin_ref.net {
                        // Some(net) => pin_nets.push(format!("{}={}", pin_ref.name, net.read().name)),
                        Some(net) => pin_nets.push(net.read().name.to_string()),
                        None => return Err(CircuitError::InstanceNotConnected(inst.name.to_string()))?,
                    }
                }
        
                let subckt_name = inst.template_circuit.name();
                writeln!(writer, "X{} {} {}", inst.name, pin_nets.join(" "), subckt_name)?;
            }
        
            // connect net
            for (i, (net1, net2)) in module_ref.connected_nets().iter().enumerate() {
                writeln!(writer, 
                    "Rconnect{} {} {} {}", 
                    i, net1.read().name, net2.read().name, 0.001
                )?;
            }
        
            writeln!(writer, ".ENDS {}", module_ref.name())?;
        }
        ShrCircuit::Primitive(primitive) => {
            writeln!(writer, "{}", primitive.read().netlist().to_spice())?;
        }
    };

    write!(writer, "\n\n")?;
    exported.insert(circuit.name());

    Ok(())
}

