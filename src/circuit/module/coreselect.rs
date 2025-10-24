use youram_macro::module;
use crate::{circuit::{CircuitFactory, DriveStrength, LogicGateKind}, format_shr, YouRAMResult};

use super::DecoderArg;

#[module(
    chip_sel_bar:          ("csb", Input),
    address:               ("addr{address_width}", Input),
    data_output_core:      ("dout_core{core_size}[{word_width}]", Input),
    chip_sel_bar_core:     ("csb_core{core_size}", Output),
    data_output:           ("dout{word_width}", Output),

    vdd:                   ("vdd", Vdd),
    gnd:                   ("gnd", Gnd),
)]
pub struct CoreSelector {
    pub address_width: usize,
    pub word_width: usize,

    #[new(value = "2usize.pow(address_width as u32)")]
    pub core_size: usize,
}

impl CoreSelector {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let decoder = self.add_module(DecoderArg::new(self.args.address_width), factory)?;
        let inv = self.add_logicgate(LogicGateKind::Inv, DRIVE_STRENGHT, factory)?;
        let or = self.add_logicgate(LogicGateKind::Or(2), DRIVE_STRENGHT, factory)?;
        let and = self.add_logicgate(LogicGateKind::And(2), DRIVE_STRENGHT, factory)?;
        let select_or = self.add_logicgate(LogicGateKind::Or(self.args.core_size), DRIVE_STRENGHT, factory)?;

        let y_nets: Vec<_> = (0..self.args.core_size).map(|i| format_shr!("y{}", i)).collect();
        let ybar_nets: Vec<_> = (0..self.args.core_size).map(|i| format_shr!("ybar{}", i)).collect();

        // decoder: input `addr{}`, output `y{}`
        {
            let mut nets = vec![];
            nets.extend((0..self.args.address_width).map(|i| Self::address_pn(i)));
            nets.extend(y_nets.iter().cloned());
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("decoder", decoder, nets.into_iter())?;
        }

        // add inv of all decoder select: input `y{}`, output `ybar{}`
        for core_index in 0..self.args.core_size {
            self.link_inv_instance(
                format_shr!("csb_inv{}", core_index), 
                inv.clone(), 
                [y_nets[core_index].clone(), ybar_nets[core_index].clone(), Self::vdd_pn(), Self::gnd_pn()]
            )?;
        }

        // add csb control: `csb` and with all `ybar{}` to control each chip `csb_core{}`
        for core_index in 0..self.args.core_size {
            self.link_logicgate_instance(
                format_shr!("csb_or{}", core_index), 
                or.clone(), 
                vec![Self::chip_sel_bar_pn(), ybar_nets[core_index].clone()],
                Self::chip_sel_bar_core_pn(core_index),
                Self::vdd_pn(),
                Self::gnd_pn(),
            )?;
        }

        // for each output bit, and select
        for bit in 0..self.args.word_width {
            // Select dout_core0[bit], dout_core0[bit], dout_core0[bit] .. as dout[bit]
            // By y[0], y[1], y[2]... 
            // if `y_nets{}` is 0(this chip not selected), data output should 0 
            for core_index in 0..self.args.core_size {
                self.link_logicgate_instance(
                    format_shr!("dout_and_{}_{}", core_index, bit), 
                    and.clone(), 
                    vec![y_nets[core_index].clone(), Self::data_output_core_pn(core_index, bit)],
                    format_shr!("y_dout_core{}[{}]", core_index, bit),
                    Self::vdd_pn(),
                    Self::gnd_pn(), 
                )?;
            }

            // add or gate, select a bit from `y_dout_core[0..][bit] to ``dout{bit}` 
            self.link_logicgate_instance(
                format_shr!("dout_or{}", bit), 
                select_or.clone(), 
                (0..self.args.core_size).map(|core_index| format_shr!("y_dout_core{}[{}]", core_index, bit)).collect(),
                Self::data_output_pn(bit),
                Self::vdd_pn(),
                Self::gnd_pn(),
            )?;
        }

        Ok(())
    }
}

const DRIVE_STRENGHT: DriveStrength = DriveStrength::X1;