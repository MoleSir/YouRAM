use std::collections::HashMap;
use reda_lib::model::{LibCell, LibExpr, LibLibrary, LibPinDirection};
use reda_sp::Spice;
use crate::{circuit::{Bitcell, ColumnTriGate, Dff, DriveStrength, Leafcell, LogicGate, LogicGateKind, Port, PortDirection, Precharge, SenseAmp, Shr, WriteDriver, BITCELL_NAME, COLUMN_TRI_GATE_NAME, PRECHARGE_NAME, SENSE_AMP_NAME, WRITE_DRIVER_NAME}, ErrorContext, YouRAMResult};
use super::PdkError;

pub struct PdkCells {
    pub logicgates: HashMap<(LogicGateKind, DriveStrength), Shr<LogicGate>>,
    pub dffs: HashMap<DriveStrength, Shr<Dff>>,
    pub bitcell: Shr<Leafcell>,
    pub sense_amp: Shr<Leafcell>,
    pub write_driver: Shr<Leafcell>,
    pub column_trigate: Shr<Leafcell>,
    pub precharge: Shr<Leafcell>,
}

impl PdkCells {
    pub fn load(library: &LibLibrary, stdcell_spice: &Spice, leafcell_spice: &Spice) -> YouRAMResult<Self> {
        // extract logicgates & dff
        let mut logicgates = HashMap::new();
        let mut dffs = HashMap::new();
        for cell in library.cells.iter() {
            if let Some(dff) = Self::extract_dff(cell, &stdcell_spice) {
                let key = dff.drive_strength;
                dffs.insert(key, Shr::new(dff));
            } else if let Some(logicgate) = Self::extract_logicgate(cell, &stdcell_spice) {
                let key = (logicgate.kind, logicgate.drive_strength);
                logicgates.insert(key, Shr::new(logicgate));
            }
        }

        // extract bitcell
        let bitcell 
            = Shr::new(Self::extract_bitcell(&leafcell_spice).context("extract bitcell")?.into());
        let sense_amp
            = Shr::new(Self::extract_sense_amp(&leafcell_spice).context("extract sens_amp")?.into());
        let write_driver
            = Shr::new(Self::extract_write_driver(&leafcell_spice).context("extract write_driver")?.into());
        let column_trigate
            = Shr::new(Self::extract_column_trigate(&leafcell_spice).context("extract column_trigate")?.into());    
        let precharge
            = Shr::new(Self::extract_precharge(&leafcell_spice).context("extract precharge")?.into()); 

        Ok(Self {
            logicgates,
            dffs,
            bitcell,
            sense_amp,
            write_driver,
            column_trigate,
            precharge,
        })   
    }
}

impl PdkCells {
    pub fn extract_bitcell(spice: &Spice) -> Result<Bitcell, PdkError> {
        let subckt = spice.subckts.iter()
            .find(|s| s.name == BITCELL_NAME)
            .ok_or_else(|| PdkError::UnexitLeafCell(BITCELL_NAME))?
            .clone();

        if subckt.ports.len() != 5 {
            return Err(PdkError::UnmatchLeafCellPinSize(5, subckt.ports.len(), BITCELL_NAME));
        }

        let bl = Port::new(subckt.ports[0].clone(), PortDirection::InOut);
        let br = Port::new(subckt.ports[1].clone(), PortDirection::InOut);
        let wl = Port::new(subckt.ports[2].clone(), PortDirection::Input);
        let vdd = Port::new(subckt.ports[3].clone(), PortDirection::Vdd);
        let gnd = Port::new(subckt.ports[4].clone(), PortDirection::Gnd);

        Ok(Bitcell::new(bl, br, wl, vdd, gnd, subckt))
    }

    pub fn extract_sense_amp(spice: &Spice) -> Result<SenseAmp, PdkError> {
        let subckt = spice.subckts.iter()
            .find(|s| s.name == SENSE_AMP_NAME)
            .ok_or_else(|| PdkError::UnexitLeafCell(SENSE_AMP_NAME))?
            .clone();

        if subckt.ports.len() != 6 {
            return Err(PdkError::UnmatchLeafCellPinSize(6, subckt.ports.len(), SENSE_AMP_NAME));
        }

        let bl   = Port::new(subckt.ports[0].clone(), PortDirection::InOut);
        let br   = Port::new(subckt.ports[1].clone(), PortDirection::InOut);
        let dout = Port::new(subckt.ports[2].clone(), PortDirection::Output);
        let en   = Port::new(subckt.ports[3].clone(), PortDirection::Input);
        let vdd  = Port::new(subckt.ports[4].clone(), PortDirection::Vdd);
        let gnd  = Port::new(subckt.ports[5].clone(), PortDirection::Gnd);

        Ok(SenseAmp::new(bl, br, dout, en, vdd, gnd, subckt))
    }

    pub fn extract_write_driver(spice: &Spice) -> Result<WriteDriver, PdkError> {
        let subckt = spice.subckts.iter()
            .find(|s| s.name == WRITE_DRIVER_NAME)
            .ok_or_else(|| PdkError::UnexitLeafCell(WRITE_DRIVER_NAME))?
            .clone();

        if subckt.ports.len() != 6 {
            return Err(PdkError::UnmatchLeafCellPinSize(6, subckt.ports.len(), WRITE_DRIVER_NAME));
        }

        let din = Port::new(subckt.ports[0].clone(), PortDirection::Input);
        let bl  = Port::new(subckt.ports[1].clone(), PortDirection::InOut);
        let br  = Port::new(subckt.ports[2].clone(), PortDirection::InOut);
        let en  = Port::new(subckt.ports[3].clone(), PortDirection::Input);
        let vdd = Port::new(subckt.ports[4].clone(), PortDirection::Vdd);
        let gnd = Port::new(subckt.ports[5].clone(), PortDirection::Gnd);

        Ok(WriteDriver::new(din, bl, br, en, vdd, gnd, subckt))
    }

    pub fn extract_column_trigate(spice: &Spice) -> Result<ColumnTriGate, PdkError> {
        let subckt = spice.subckts.iter()
            .find(|s| s.name == COLUMN_TRI_GATE_NAME)
            .ok_or_else(|| PdkError::UnexitLeafCell(COLUMN_TRI_GATE_NAME))?
            .clone();

        if subckt.ports.len() != 7 {
            return Err(PdkError::UnmatchLeafCellPinSize(7, subckt.ports.len(), COLUMN_TRI_GATE_NAME));
        }

        let bl    = Port::new(subckt.ports[0].clone(), PortDirection::InOut);
        let br    = Port::new(subckt.ports[1].clone(), PortDirection::InOut);
        let bl_o  = Port::new(subckt.ports[2].clone(), PortDirection::InOut);
        let br_o  = Port::new(subckt.ports[3].clone(), PortDirection::InOut);
        let sel   = Port::new(subckt.ports[4].clone(), PortDirection::Input);
        let vdd   = Port::new(subckt.ports[5].clone(), PortDirection::Vdd);
        let gnd   = Port::new(subckt.ports[6].clone(), PortDirection::Gnd);

        Ok(ColumnTriGate::new(bl, br, bl_o, br_o, sel, vdd, gnd, subckt))
    }

    pub fn extract_precharge(spice: &Spice) -> Result<Precharge, PdkError> {
        let subckt = spice.subckts.iter()
            .find(|s| s.name == PRECHARGE_NAME)
            .ok_or_else(|| PdkError::UnexitLeafCell(PRECHARGE_NAME))?
            .clone();

        if subckt.ports.len() != 4 {
            return Err(PdkError::UnmatchLeafCellPinSize(4, subckt.ports.len(), PRECHARGE_NAME));
        }

        let bl    = Port::new(subckt.ports[0].clone(), PortDirection::InOut);
        let br    = Port::new(subckt.ports[1].clone(), PortDirection::InOut);
        let enable   = Port::new(subckt.ports[2].clone(), PortDirection::Input);
        let vdd   = Port::new(subckt.ports[3].clone(), PortDirection::Vdd);

        Ok(Precharge::new(bl, br, enable, vdd, subckt))   
    }
}

impl PdkCells {
    pub fn extract_dff(cell: &LibCell, spice: &Spice) -> Option<Dff> {
        // ff exit?
        let ff = cell.ff.as_ref()?;
        // TODO: better way to check (din, clk, q, qn)
        if cell.input_pins().count() == 2 && cell.output_pins().count() == 2 {
            let subckt = spice.subckts.iter()
                .find(|s| s.name == cell.name)?
                .clone();

            let drive_strength = DriveStrength::try_from_cell(cell)?;

            let mut ports = vec![];
            let mut din_port_index = None;
            let mut clk_port_index = None;
            let mut q_port_index = None;
            let mut qn_port_index = None;
            let mut vdd_port_index = None;
            let mut gnd_port_index = None;
            for (port_index, port_name) in subckt.ports.iter().enumerate() {
                if let Some(pin) = cell.get_pin(&port_name) {
                    let direction = match pin.direction {
                        LibPinDirection::Input => {
                            if pin.clock == Some(true) {
                                clk_port_index = Some(port_index);
                            } else {
                                din_port_index = Some(port_index);
                            }
                            PortDirection::Input
                        }
                        LibPinDirection::Output => {
                            let function = pin.function.as_ref()?;
                            if let LibExpr::Var(name) = function {
                                if name.as_str() == ff.names[0].as_str() {
                                    q_port_index = Some(port_index);
                                } else if name.as_str() == ff.names[1].as_str() {
                                    qn_port_index = Some(port_index);
                                } else {
                                    panic!()
                                }
                            } else {
                                panic!();
                            }
                            PortDirection::Output
                        }
                        _ => panic!(),
                    };
                    ports.push(Port::new(port_name.clone(), direction));
                } else if let Some(pg_pin) = cell.get_pg_pin(&port_name) {
                    let name = pg_pin.voltage_name.to_lowercase();
                    match name.as_str() {
                        "vdd" => {
                            vdd_port_index = Some(port_index);
                            ports.push(Port::new(port_name.clone(), PortDirection::Vdd));
                        }
                        "gnd" | "vss" => {
                            gnd_port_index = Some(port_index);
                            ports.push(Port::new(port_name.clone(), PortDirection::Gnd));
                        }
                        // _ => return Err(PdkError::UnkownPgPinName(name))?,
                        _ => return None,
                    };
                }
            }

            Some(Dff {
                name: cell.name.clone().into(),
                drive_strength,
                ports,
                din_port_index: din_port_index?,
                clk_port_index: clk_port_index?,
                q_port_index: q_port_index?,
                qn_port_index: qn_port_index?,
                vdd_port_index: vdd_port_index?,
                gnd_port_index: gnd_port_index?,
                netlist: subckt,
            })
        } else {
            None
        }
    }

    // TODO: better way to extract logicgate!(add result)
    pub fn extract_logicgate(cell: &LibCell, spice: &Spice) -> Option<LogicGate> {
        // 1. 根据输出 pin function 判断类型
        if cell.output_pins().count() != 1 {
            return None;
        }
        let output_pin = cell.output_pins().nth(0)?;
        let function = output_pin.function.as_ref()?;
        let kind = Self::try_transform_expr(function)?;

        // 2. 查找 SPICE subckt
        let subckt = spice.subckts.iter()
            .find(|s| s.name == cell.name)?
            .clone();

        // 3. 构建端口列表
        let mut ports = vec![];
        let mut input_port_indexs = vec![];
        let mut output_port_index = None;
        let mut vdd_port_index = None;
        let mut gnd_port_index = None;

        for (port_index, port_name) in subckt.ports.iter().enumerate() {
            if let Some(pin) = cell.get_pin(&port_name) {
                let direction = match pin.direction {
                    LibPinDirection::Input => {
                        input_port_indexs.push(port_index);
                        PortDirection::Input
                    }
                    LibPinDirection::Output => {
                        output_port_index = Some(port_index);
                        PortDirection::Output
                    }
                    _ => panic!(),
                };
                ports.push(Port::new(port_name.clone(), direction));
            } else if let Some(pg_pin) = cell.get_pg_pin(&port_name) {
                // TODO: use voltage_name to regc vdd/gnd
                let name = pg_pin.voltage_name.to_lowercase();
                match name.as_str() {
                    "vdd" => {
                        vdd_port_index = Some(port_index);
                        ports.push(Port::new(port_name.clone(), PortDirection::Vdd));
                    }
                    "gnd" | "vss" => {
                        gnd_port_index = Some(port_index);
                        ports.push(Port::new(port_name.clone(), PortDirection::Gnd));
                    }
                    // _ => return Err(PdkError::UnkownPgPinName(name))?,
                    _ => return None,
                };
            }
        }

        if kind == LogicGateKind::Inv && ports.len() != 4 {
            return None;
        }

        let drive_strength = DriveStrength::try_from_cell(cell)?;

        Some(LogicGate {
            name: cell.name.clone().into(),
            drive_strength,
            kind,
            ports,
            input_port_indexs,
            output_port_index: output_port_index?,
            vdd_port_index: vdd_port_index?,
            gnd_port_index: gnd_port_index?,
            netlist: subckt,
        })
    }

    pub fn try_transform_expr(expr: &LibExpr) -> Option<LogicGateKind> {
        match expr {
            LibExpr::Not(inner) => match &**inner {
                LibExpr::Var(_) => Some(LogicGateKind::Inv),
                LibExpr::And(_) => Self::analyze_and_or(inner, true).map(LogicGateKind::Nand),
                LibExpr::Or(_) => Self::analyze_and_or(inner, true).map(LogicGateKind::Nor),
                _ => None,
            },
            LibExpr::And(_) => Self::analyze_and_or(expr, false).map(LogicGateKind::And),
            LibExpr::Or(_) => Self::analyze_and_or(expr, false).map(LogicGateKind::Or),
            _ => None,
        }
    }

    fn analyze_and_or(expr: &LibExpr, ignore_not: bool) -> Option<usize> {
        match expr {
            LibExpr::And(children) => {
                let mut count = 0;
                for c in children {
                    match c {
                        LibExpr::Var(_) | LibExpr::Const(_) => count += 1,
                        LibExpr::Not(inner) if ignore_not => {
                            if matches!(**inner, LibExpr::Var(_) | LibExpr::Const(_)) {
                                count += 1;
                            } else {
                                return None;
                            }
                        }
                        LibExpr::And(_) => count += Self::analyze_and_or(c, ignore_not)?,
                        _ => return None, // 出现 Or、Xor 等混合逻辑 → 不是纯 AND 树
                    }
                }
                Some(count)
            }
            LibExpr::Or(children) => {
                let mut count = 0;
                for c in children {
                    match c {
                        LibExpr::Var(_) | LibExpr::Const(_) => count += 1,
                        LibExpr::Not(inner) if ignore_not => {
                            if matches!(**inner, LibExpr::Var(_) | LibExpr::Const(_)) {
                                count += 1;
                            } else {
                                return None;
                            }
                        }
                        LibExpr::Or(_) => count += Self::analyze_and_or(c, ignore_not)?,
                        _ => return None, // 出现 And、Xor 等混合逻辑 → 不是纯 OR 树
                    }
                }
                Some(count)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    fn str_to_kind(s: &str) -> Option<LogicGateKind> {
        let expr = LibExpr::from_str(s).unwrap();
        PdkCells::try_transform_expr(&expr)
    }

    #[test]
    fn test_std_cell_kind() {

        // 单变量 NOT -> Inv
        assert_eq!(str_to_kind("!A").unwrap(), LogicGateKind::Inv);

        // 简单 AND
        assert_eq!(str_to_kind("(A1 & A2)").unwrap(), LogicGateKind::And(2));

        // 嵌套 AND
        assert_eq!(str_to_kind("((A1 & A2) & A3)").unwrap(), LogicGateKind::And(3));
        assert_eq!(str_to_kind("(((A1 & A2) & A3) & A4)").unwrap(), LogicGateKind::And(4));

        // 简单 NAND
        assert_eq!(str_to_kind("!(A1 & A2)").unwrap(), LogicGateKind::Nand(2));

        // 嵌套 NAND
        assert_eq!(str_to_kind("!((A1 & A2) & A3)").unwrap(), LogicGateKind::Nand(3));
        assert_eq!(str_to_kind("!(((A1 & A2) & A3) & A4)").unwrap(), LogicGateKind::Nand(4));

        // 简单 OR
        assert_eq!(str_to_kind("(A1 | A2)").unwrap(), LogicGateKind::Or(2));

        // 简单 NOR
        assert_eq!(str_to_kind("!(A1 | A2)").unwrap(), LogicGateKind::Nor(2));

        // 嵌套 OR
        assert_eq!(str_to_kind("((A1 | A2) | A3)").unwrap(), LogicGateKind::Or(3));

        // 嵌套 NOR
        assert_eq!(str_to_kind("!((A1 | A2) | A3)").unwrap(), LogicGateKind::Nor(3));
    }
}
