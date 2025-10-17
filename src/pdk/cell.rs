use reda_lib::model::{LibCell, LibExpr, LibPinDirection};
use reda_sp::Spice;
use crate::circuit::{Bitcell, ColumnTriGate, DriveStrength, Port, PortDirection, SenseAmp, Stdcell, StdcellKind, WriteDriver, BITCELL_NAME, COLUMN_TRI_GATE_NAME, SENSE_AMP_NAME, WRITE_DRIVER_NAME};
use super::{Pdk, PdkError};

impl Pdk {
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
        let vdd = Port::new(subckt.ports[3].clone(), PortDirection::Source);
        let gnd = Port::new(subckt.ports[4].clone(), PortDirection::Source);

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
        let vdd  = Port::new(subckt.ports[4].clone(), PortDirection::Source);
        let gnd  = Port::new(subckt.ports[5].clone(), PortDirection::Source);

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
        let vdd = Port::new(subckt.ports[4].clone(), PortDirection::Source);
        let gnd = Port::new(subckt.ports[5].clone(), PortDirection::Source);

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
        let vdd   = Port::new(subckt.ports[5].clone(), PortDirection::Source);
        let gnd   = Port::new(subckt.ports[6].clone(), PortDirection::Source);

        Ok(ColumnTriGate::new(bl, br, bl_o, br_o, sel, vdd, gnd, subckt))
    }
}

impl Pdk {
    // TODO: better way to extract stdcell!
    pub fn extract_stdcell(cell: &LibCell, spice: &Spice) -> Option<Stdcell> {
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
            } else if let Some(_) = cell.get_pg_pin(&port_name) {
                // TODO, how to recg vdd or gnd?
                if vdd_port_index.is_none() {
                    vdd_port_index = Some(port_index);
                } else {
                    gnd_port_index = Some(port_index);
                }
                ports.push(Port::new(port_name.clone(), PortDirection::Source));
                
            }
        }

        if kind == StdcellKind::Inv && ports.len() != 4 {
            return None;
        }

        let drive_strength = DriveStrength::try_from_cell(cell)?;

        Some(Stdcell {
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

    pub fn try_transform_expr(expr: &LibExpr) -> Option<StdcellKind> {
        match expr {
            LibExpr::Not(inner) => match &**inner {
                LibExpr::Var(_) => Some(StdcellKind::Inv),
                LibExpr::And(_) => Self::analyze_and_or(inner, true).map(StdcellKind::Nand),
                LibExpr::Or(_) => Self::analyze_and_or(inner, true).map(StdcellKind::Nor),
                _ => None,
            },
            LibExpr::And(_) => Self::analyze_and_or(expr, false).map(StdcellKind::And),
            LibExpr::Or(_) => Self::analyze_and_or(expr, false).map(StdcellKind::Or),
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

    fn str_to_kind(s: &str) -> Option<StdcellKind> {
        let expr = LibExpr::from_str(s).unwrap();
        Pdk::try_transform_expr(&expr)
    }

    #[test]
    fn test_std_cell_kind() {

        // 单变量 NOT -> Inv
        assert_eq!(str_to_kind("!A").unwrap(), StdcellKind::Inv);

        // 简单 AND
        assert_eq!(str_to_kind("(A1 & A2)").unwrap(), StdcellKind::And(2));

        // 嵌套 AND
        assert_eq!(str_to_kind("((A1 & A2) & A3)").unwrap(), StdcellKind::And(3));
        assert_eq!(str_to_kind("(((A1 & A2) & A3) & A4)").unwrap(), StdcellKind::And(4));

        // 简单 NAND
        assert_eq!(str_to_kind("!(A1 & A2)").unwrap(), StdcellKind::Nand(2));

        // 嵌套 NAND
        assert_eq!(str_to_kind("!((A1 & A2) & A3)").unwrap(), StdcellKind::Nand(3));
        assert_eq!(str_to_kind("!(((A1 & A2) & A3) & A4)").unwrap(), StdcellKind::Nand(4));

        // 简单 OR
        assert_eq!(str_to_kind("(A1 | A2)").unwrap(), StdcellKind::Or(2));

        // 简单 NOR
        assert_eq!(str_to_kind("!(A1 | A2)").unwrap(), StdcellKind::Nor(2));

        // 嵌套 OR
        assert_eq!(str_to_kind("((A1 | A2) | A3)").unwrap(), StdcellKind::Or(3));

        // 嵌套 NOR
        assert_eq!(str_to_kind("!((A1 | A2) | A3)").unwrap(), StdcellKind::Nor(3));
    }
}
