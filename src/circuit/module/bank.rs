use youram_macro::module;
use crate::{check_arg, circuit::{CircuitFactory, DataPathArg, ReplicaBitcellArrayArg, ShrString}, format_shr, YouRAMResult};
use super::{BitcellArrayRecursiveArg, PrechargeArrayArg};

#[module(
    wordline_enbale:      ("wl_en", Input),
    precharge_enbale_bar: ("p_en_bar", Input),
    sense_amp_enable:     ("sa_en", Output),
    write_driver_enable:  ("we_en", Input),

    wordline:             ("wl{row_size}", Input),
    col_select:           ("csel{column_sel_size}", Input, "column_sel_size > 1"),

    data_input:           ("din{word_width}", Input),
    data_output:          ("dout{word_width}", Input),

    replical_bitline:     ("rbl", InOut),

    vdd:                  ("vdd", Source),
    gnd:                  ("gnd", Source),
)]
pub struct Bank {
    pub row_size: usize,
    pub column_sel_size: usize,
    pub word_width: usize,

    #[new(value = "column_sel_size * word_width")]
    pub column_size: usize,
}


impl Bank {
    pub fn build(&mut self, factory: &mut CircuitFactory) -> YouRAMResult<()> {
        check_arg!(self.args.row_size >= 1, "row size {} < 1", self.args.row_size);
        check_arg!(self.args.column_size >= 1, "column size {} < 1", self.args.column_size);
        
        let replical_bitcell_array 
            = self.add_module(ReplicaBitcellArrayArg::new(self.args.row_size), factory)?;
        let bitcell_array 
            = self.add_module(BitcellArrayRecursiveArg::new(self.args.row_size, self.args.column_size), factory)?;

        let data_path
            = self.add_module(DataPathArg::new(self.args.word_width, self.args.column_sel_size), factory)?;
        let precharge_array 
            = self.add_module(PrechargeArrayArg::new(self.args.column_size), factory)?;
        
        let bl_nets: Vec<_> = (0..self.args.column_size).map(|i| format_shr!("bl{}", i)).collect();
        let br_nets: Vec<_> = (0..self.args.column_size).map(|i| format_shr!("br{}", i)).collect();

        let rbr_net: ShrString = "rbr".into();
   
        // bitcell array
        {
            let mut nets = vec![];
            nets.extend(bl_nets.iter().cloned());
            nets.extend(br_nets.iter().cloned());
            nets.extend((0..self.args.row_size).map(|i| Self::wordline_pn(i)));
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("bitcell_array", bitcell_array, nets.into_iter())?;
        }

        // replical bitcell array
        {
            let mut nets = vec![];
            nets.push(Self::replical_bitline_pn());
            nets.push(rbr_net.clone());
            nets.push(Self::wordline_enbale_pn());
            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("replical_bitcell_array", replical_bitcell_array, nets.into_iter())?;   
        }  
   
        // precharge array
        {
            let mut nets = vec![];
            nets.extend(bl_nets.iter().cloned());
            nets.extend(br_nets.iter().cloned());
            nets.push(Self::precharge_enbale_bar_pn());
            nets.push(Self::vdd_pn());

            self.link_module_instance("precharge_array", precharge_array, nets.into_iter())?;
        }

        // precharge for rbl
        self.link_precharge_instance(factory, "precharge_rbl", 
            Self::replical_bitline_pn(), rbr_net.clone(), Self::precharge_enbale_bar_pn(), Self::vdd_pn())?;

        // datapath
        {
            let mut nets = vec![];
            nets.push(Self::sense_amp_enable_pn());
            nets.push(Self::write_driver_enable_pn());
            nets.extend(bl_nets.iter().cloned());
            nets.extend(br_nets.iter().cloned());
            if self.has_column_address() {
                nets.extend((0..self.args.column_sel_size).map(|i| Self::col_select_pn(i)));
            }

            nets.extend((0..self.args.word_width).map(|i| Self::data_input_pn(i)));
            nets.extend((0..self.args.word_width).map(|i| Self::data_output_pn(i)));

            nets.push(Self::vdd_pn());
            nets.push(Self::gnd_pn());

            self.link_module_instance("datapath", data_path, nets.into_iter())?;
        }

        // write driver for rbl
        self.link_writedriver_instance(
            factory, 
            "writedriver", 
            Self::gnd_pn(), 
            Self::replical_bitline_pn(),
            rbr_net.clone(),
            Self::write_driver_enable_pn(),
            Self::gnd_pn(),
            Self::gnd_pn(),
        )?;

        Ok(())
    }

    pub fn has_column_address(&self) -> bool {
        self.args.column_sel_size > 1
    }
}