use youram_macro::module;
use crate::{circuit::{CircuitFactory, DriveStrength, StdcellKind}, YouRAMResult};

#[module(
    clock:                 ("clk", Input),
    chip_sel_bar:          ("csb", Input),
    write_enable:          ("we", Input),
    replical_bitline:      ("rbl", InOut),

    wordline_enable:       ("wl_en", Output),
    precharge_enable_bar:  ("p_en_bar", Output),
    sense_amp_enable:      ("sa_en", Output),
    write_deriver_enable:  ("we_en", Output),
    
    voltage:               ("vdd", Source),
    groud:                 ("gnd", Source),
)]
pub struct ControlLogic {
}

impl ControlLogicArg {
    pub fn new() -> Self { ControlLogicArg {} }
}

impl ControlLogic {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        // add circuits
        let inv = self.add_stdcell(StdcellKind::Inv, DRIVE_STRENGTH, factory)?;
        let or2 = self.add_stdcell(StdcellKind::Or(2), DRIVE_STRENGTH, factory)?;
        let and2 = self.add_stdcell(StdcellKind::And(2), DRIVE_STRENGTH, factory)?;
        let and3 = self.add_stdcell(StdcellKind::And(3), DRIVE_STRENGTH, factory)?;
        let nand3 = self.add_stdcell(StdcellKind::Nand(3), DRIVE_STRENGTH, factory)?;        

        // input inv
        self.link_inv_instance("clk_inv", inv.clone(), [Self::clock_pn(), "clk_bar".into(), Self::voltage_pn(), Self::groud_pn()])?;
        self.link_inv_instance("we_inv",  inv.clone(), [Self::write_enable_pn(), "we_bar".into(),  Self::voltage_pn(), Self::groud_pn()])?;
        self.link_inv_instance("csb_inv", inv.clone(), [Self::chip_sel_bar_pn(), "csb_bar".into(), Self::voltage_pn(), Self::groud_pn()])?;
        self.link_inv_instance("rbl_inv", inv.clone(), [Self::replical_bitline_pn(), "rbl_bar".into(), Self::voltage_pn(), Self::groud_pn()])?;

        // word line
        self.link_stdcell_instance("wl_and2", and2.clone(), 
            vec!["csb_bar".into(), Self::write_enable_pn()], "wl_net1", 
            Self::voltage_pn(), Self::groud_pn())?;
        self.link_stdcell_instance("wl_and3", and3.clone(), 
            vec!["csb_bar".into(), "clk_bar".into(), Self::replical_bitline_pn()], "wl_net2", 
            Self::voltage_pn(), Self::groud_pn())?;
        self.link_stdcell_instance("wl_or2", or2.clone(), 
            vec!["wl_net1", "wl_net2"], Self::wordline_enable_pn(), 
            Self::voltage_pn(), Self::groud_pn())?;

        // precharge
        self.link_stdcell_instance("p_nand3", nand3.clone(), 
            vec!["csb_bar".into(), "we_bar".into(), Self::clock_pn()], Self::precharge_enable_bar_pn(), 
            Self::voltage_pn(), Self::groud_pn())?;

        // sense amp
        self.link_stdcell_instance("sa_and2_1", and2.clone(), 
            vec!["csb_bar", "we_bar"], "sa_net1", 
            Self::voltage_pn(), Self::groud_pn())?;
        self.link_stdcell_instance("sa_and2_2", and2.clone(), 
            vec!["clk_bar", "rbl_bar"], "sa_net2", 
            Self::voltage_pn(), Self::groud_pn())?;
        self.link_stdcell_instance("sa_and2", and2.clone(), 
            vec!["sa_net1", "sa_net2"], Self::sense_amp_enable_pn(), 
            Self::voltage_pn(), Self::groud_pn())?;

        // write deriver 
        self.link_stdcell_instance("we_and2", and2.clone(), 
            vec!["csb_bar".into(), Self::write_enable_pn()], Self::write_deriver_enable_pn(), 
            Self::voltage_pn(), Self::groud_pn())?;

        Ok(())
    }
}

const DRIVE_STRENGTH: DriveStrength = DriveStrength::X1;
// const POWER_DRIVE_STRENGTH: DriveStrength = DriveStrength::X2;