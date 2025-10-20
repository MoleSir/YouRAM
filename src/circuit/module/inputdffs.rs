use youram_macro::module;
use crate::{circuit::{CircuitFactory, DriveStrength}, format_shr, YouRAMResult};

#[module(
    clock:            ("clk", Input),

    chip_sel_bar:     ("csb", Input),
    write_enable:     ("we", Input),
    address:          ("addr{address_width}", Input),
    data_input:       ("din{word_width}", Input),

    chip_sel_bar_reg: ("csb_r", Output),
    write_enable_reg: ("we_r", Output),
    address_reg:      ("addr_r{address_width}", Output),
    data_input_reg:   ("din_r{word_width}", Output),

    vdd:              ("vdd", Source),
    gnd:              ("gnd", Source),
)]
pub struct InputDffs {
    pub address_width: usize,
    pub word_width: usize,
}

impl InputDffs {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        let dff = self.add_dff(DriveStrength::X1, factory)?;

        self.link_dff_instance("we_dff", dff.clone(), Self::write_enable_pn(), Self::clock_pn(), Self::write_enable_reg_pn(), "we_qn", Self::vdd_pn(), Self::gnd_pn())?;
        self.link_dff_instance("csb_dff", dff.clone(), Self::chip_sel_bar_pn(), Self::clock_pn(), Self::chip_sel_bar_reg_pn(), "csb_qn", Self::vdd_pn(), Self::gnd_pn())?;

        for address in 0..self.args.address_width {
            self.link_dff_instance(
                format_shr!("add_dff{}", address), dff.clone(), 
                Self::address_pn(address), 
                Self::clock_pn(), 
                Self::address_reg_pn(address), 
                format_shr!("addr{}_qn", address), 
                Self::vdd_pn(), 
                Self::gnd_pn()
            )?;
        }

        for address in 0..self.args.word_width {
            self.link_dff_instance(
                format_shr!("din_dff{}", address), dff.clone(), 
                Self::data_input_pn(address), 
                Self::clock_pn(), 
                Self::data_input_reg_pn(address), 
                format_shr!("din{}_qn", address), 
                Self::vdd_pn(), 
                Self::gnd_pn()
            )?;
        }

        Ok(())
    }
}