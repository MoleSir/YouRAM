use youram_macro::module;
use crate::{check_arg, circuit::{AndArrayArg, Bank, BankArg, CircuitFactory, ControlLogic, ControlLogicArg}, format_shr, YouRAMResult};

#[module(
    clock:         ("clk", Input),
    chip_sel_bar:  ("csb", Input),
    write_enable:  ("we", Input),
    row_select:    ("rsel{row_size}", Input),
    col_select:    ("csel{column_sel_size}", Input, "column_sel_size > 1"),

    data_input:    ("din{word_width}", Input),
    data_output:   ("dout{word_width}", Input),

    vdd:           ("vdd", Source),
    gnd:           ("gnd", Source),
)]
pub struct Core {
    pub row_size: usize,
    pub column_sel_size: usize,
    pub word_width: usize,

    #[new(value = "column_sel_size * word_width")]
    pub column_size: usize,
}

impl Core {
    pub const MAX_ROW_SIZE: usize = 64;
    pub const MAX_COLUMN_SIZE: usize = 128;
    pub const MAX_BITCELL_SIZE: usize = Self::MAX_ROW_SIZE * Self::MAX_COLUMN_SIZE;

    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.row_size <= Self::MAX_ROW_SIZE, "row sel size '{}' > {}", self.args.row_size, Self::MAX_ROW_SIZE);
        check_arg!(self.args.column_sel_size <= Self::MAX_COLUMN_SIZE, "column sel size '{}' > {}", self.args.column_sel_size, Self::MAX_COLUMN_SIZE);
        check_arg!(self.bitcell_size() <= Self::MAX_BITCELL_SIZE, "Too much bitcell size");

        let bank 
            = self.add_module(BankArg::new(self.args.row_size, self.args.column_sel_size, self.args.word_width), factory)?;
        let control_logic 
            = self.add_module(ControlLogicArg::new(), factory)?;
        let and_array 
            = self.add_module(AndArrayArg::new(self.args.row_size), factory)?;
        // TODO: wordline driver

        let rbl_net = ControlLogic::replical_bitline_pn();
        let wl_en_net = ControlLogic::wordline_enable_pn();
        let p_en_bar_net = ControlLogic::precharge_enable_bar_pn();
        let sa_en_net = ControlLogic::sense_amp_enable_pn();
        let we_en_net = ControlLogic::write_deriver_enable_pn();

        let wl_nets: Vec<_> = (0..self.args.row_size).map(|row| Bank::wordline_pn(row)).collect(); 

        // control_logic
        {
            let nets = vec![
                Self::clock_pn(),
                Self::chip_sel_bar_pn(),
                Self::write_enable_pn(),
                rbl_net.clone(),
                wl_en_net.clone(),
                p_en_bar_net.clone(),
                sa_en_net.clone(),
                we_en_net.clone(),
                Self::vdd_pn(),
                Self::gnd_pn(),
            ];
            self.link_module_instance("control_logic", control_logic, nets.into_iter())?;
        }

        // bank
        {
            let mut nets = vec![
                wl_en_net.clone(),
                p_en_bar_net.clone(),
                sa_en_net.clone(),
                we_en_net.clone(),
            ];
            nets.extend(wl_nets.iter().cloned());
            if bank.read().has_column_address() {
                nets.extend((0..self.args.column_sel_size).map(|c| Self::col_select_pn(c)));                
            }
            nets.extend((0..self.args.word_width).map(|i| Self::data_input_pn(i)));
            nets.extend((0..self.args.word_width).map(|i| Self::data_output_pn(i)));
            nets.push(rbl_net.clone());
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("bank", bank, nets.into_iter())?;
        }

        // and array 
        {
            let mut nets = vec![];
            nets.extend((0..self.args.row_size).map(|r| Self::row_select_pn(r)));
            nets.push(wl_en_net.clone());
            nets.extend(wl_nets.into_iter());
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("andarray", and_array, nets.into_iter())?;
        }

        Ok(())
    }
    
    pub fn bitcell_size(&self) -> usize {
        self.args.row_size * self.args.column_sel_size
    } 
}