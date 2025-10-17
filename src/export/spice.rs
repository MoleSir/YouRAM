use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use reda_sp::ToSpice;
use tracing::{debug, info};
use crate::circuit::{CircuitError, Design, Primitive, ShrModule, ShrString};
use crate::YouRAMResult;

pub fn write_spice<P: AsRef<Path>>(module: ShrModule, path: P) -> YouRAMResult<()> {
    let path = path.as_ref();
    info!("write circuit {} to {:?}", module.read().name(), path);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    let mut exported = HashSet::new();
    write_spice_recursive(&mut writer, &module, &mut exported)?;
    Ok(())
}

fn write_spice_recursive<W: Write>(
    writer: &mut W,
    module: &ShrModule,
    exported: &mut HashSet<ShrString>,
) -> YouRAMResult<()> {
    let module_ref = module.read();
    debug!("write module {}", module_ref.name());

    // write sub cell
    for leafcell in module_ref.sub_leafcells() {
        if exported.insert(leafcell.read().name()) {
            writeln!(writer, "{}", leafcell.read().netlist().to_spice())?;
            write!(writer, "\n\n")?;
        }            
    }
    for stdcell in module_ref.sub_stdcells() {
        if exported.insert(stdcell.read().name()) {
            writeln!(writer, "{}", stdcell.read().netlist().to_spice())?;
            write!(writer, "\n\n")?;
        }   
    }
    for sub_module in module_ref.sub_modules() {
        if exported.insert(sub_module.read().name()) {
            write_spice_recursive(writer, sub_module, exported)?;
            write!(writer, "\n\n")?;
        }
    }

    // .SUBCKT header
    let ports = module_ref.ports();
    let port_names: Vec<_> = ports.iter().map(|p| p.read().name.to_string()).collect();
    writeln!(writer, ".SUBCKT {} {}", module_ref.name(), port_names.join(" "))?;

    // Instance
    for inst in module_ref.instances() {            
        let inst = inst.read();
        
        let mut pin_nets = Vec::new();
        for pin in inst.pins.iter() {
            match &pin.read().net {
                Some(net) => pin_nets.push(net.read().name.to_string()),
                None => return Err(CircuitError::InstanceNotConnected(inst.name.to_string()))?,
            }
        }

        let subckt_name = inst.template_circuit.name();
        writeln!(writer, "X{} {} {}", inst.name, pin_nets.join(" "), subckt_name)?;
    }

    writeln!(writer, ".ENDS {}", module_ref.name())?;

    Ok(())
}
