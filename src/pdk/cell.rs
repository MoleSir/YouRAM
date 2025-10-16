use reda_lib::model::{LibCell, LibExpr, LibPinDirection};
use reda_sp::Spice;
use crate::circuit::{Bitcell, DriveStrength, Port, PortDirection, Stdcell, StdcellKind, BITCELL_NAME};
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
}

impl Pdk {
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
        for port_name in subckt.ports.iter() {
            if let Some(pin) = cell.get_pin(&port_name) {
                let direction = match pin.direction {
                    LibPinDirection::Input => PortDirection::Input,
                    LibPinDirection::Output => PortDirection::Output,
                    _ => panic!(),
                };
                ports.push(Port::new(port_name.clone(), direction));
            } else if let Some(_) = cell.get_pg_pin(&port_name) {
                ports.push(Port::new(port_name.clone(), PortDirection::Source));
                
            }
        }

        let drive_strength = DriveStrength::try_from_cell(cell)?;

        Some(Stdcell {
            name: cell.name.clone(),
            drive_strength,
            kind,
            ports,
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
