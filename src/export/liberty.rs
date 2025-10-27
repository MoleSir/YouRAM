use std::{fs::File, path::Path, sync::Arc};
use reda_lib::model::LibLuTable;
use reda_unit::{Capacitance, Temperature, Time, Voltage};
use tracing::info;
use std::io::{BufWriter, Write};
use std::fmt::Write as FmtWrite;
use crate::circuit::DriveStrength;
use crate::{charz::TimingCharz, circuit::{Shr, Sram}, pdk::{Pdk, Process, Pvt}, simulate::SpiceCommand, YouRAMResult};

pub fn write_liberty(
    sram: Shr<Sram>, 
    path: impl AsRef<Path>, 
    period: Time, 
    pdk: Arc<Pdk>, 
    command: Box<dyn SpiceCommand>, 
    temp_folder: impl AsRef<Path>,
) -> YouRAMResult<()> {
    // collect all 
    let input_net_transitions = pdk.timing_input_net_transitions();
    let output_net_capacitances = pdk.timing_output_net_capacitances();
    let pvt = pdk.pvt();
    let temp_folder: &Path = temp_folder.as_ref();

    let all_result = TimingCharz::config()
        .sram(sram.clone())
        .period(period)
        .pvt(pvt.clone())
        .input_net_transitions(input_net_transitions)
        .output_net_capacitances(output_net_capacitances)
        .pdk(pdk.clone())
        .command_box(command)
        .temp_folder(temp_folder)
        .analyze()?;

    let mut delay_lhs = vec![];
    let mut delay_hls = vec![];
    let mut slew_lhs = vec![];
    let mut slew_hls = vec![];

    for result_same_slew in all_result.into_iter() {
        let mut delay_lh_in_same_input_slew = vec![];
        let mut delay_hl_in_same_input_slew = vec![];
        let mut slew_lh_in_same_input_slew = vec![];
        let mut slew_hl_in_same_input_slew = vec![];
        for result in result_same_slew.into_iter() {
            delay_hl_in_same_input_slew.push(result.delay_hl);
            delay_lh_in_same_input_slew.push(result.delay_lh);
            slew_hl_in_same_input_slew.push(result.slew_hl);
            slew_lh_in_same_input_slew.push(result.slew_lh);
        }
        delay_lhs.push(delay_hl_in_same_input_slew);
        delay_hls.push(delay_lh_in_same_input_slew);
        slew_lhs.push(slew_hl_in_same_input_slew);
        slew_hls.push(slew_lh_in_same_input_slew);
    }

    // write to path
    let path = path.as_ref();
    info!("write circuit {} to {:?}", sram.read().name, path);
    let mut writor = LibertyWritor::new(
        sram, 
        pvt.clone(), 
        pdk.clone(), 
        input_net_transitions.to_vec(), 
        output_net_capacitances.to_vec(), 
        delay_lhs,
        delay_hls,
        slew_lhs,
        slew_hls,
        path,
    )?;
    writor.write()?;

    Ok(())
}

struct LibertyWritor {
    sram: Shr<Sram>,
    pvt: Pvt,
    pdk: Arc<Pdk>, 
    input_net_transitions: Vec<Time>,
    output_net_capacitances: Vec<Capacitance>,
    delay_lhs: Vec<Vec<Time>>,
    delay_hls: Vec<Vec<Time>>,
    slew_lhs: Vec<Vec<Time>>,
    slew_hls: Vec<Vec<Time>>,
    writor: BufWriter<File>,
}

impl LibertyWritor {
    fn new(
        sram: Shr<Sram>, 
        pvt: Pvt, 
        pdk: Arc<Pdk>, 
        input_net_transitions: Vec<Time>,
        output_net_capacitances: Vec<Capacitance>,
        delay_lhs: Vec<Vec<Time>>,
        delay_hls: Vec<Vec<Time>>,
        slew_lhs: Vec<Vec<Time>>,
        slew_hls: Vec<Vec<Time>>,
        path: impl AsRef<Path>, 
    ) -> YouRAMResult<Self> {
        let file = File::create(path)?;
        let writor = BufWriter::new(file);



        Ok(Self {
            sram, pvt, pdk, input_net_transitions, output_net_capacitances, delay_hls, delay_lhs, slew_hls, slew_lhs, writor
        })
    }

    fn write(&mut self) -> YouRAMResult<()> {
        self.write_library()?;
        Ok(())
    }

    fn write_library(&mut self) -> YouRAMResult<()> {
        self.write_line(0, &format!("library ({}_lib) {{", self.pdk.name()))?;
        self.write_line(1, "delay_model : \"table_lookup\";")?;

        self.write_units()?;
        self.write_defaults()?;
        self.write_luttemplate()?;
        self.write_bus()?;
        self.write_cell()?;

        self.write_line(0, "}")?;
        Ok(())    
    }

    fn write_cell(&mut self) -> YouRAMResult<()> {
        self.write_line(1, &format!("cell ({}) {{", self.sram.read().name))?;
        self.write_line(2, "memory() {")?;
        self.write_line(3, "type : ram;")?;
        self.write_line(3, &format!("address_width : {};", self.sram.read().address_width()))?;
        self.write_line(3, &format!("word_width : {};", self.sram.read().word_width()))?;
        self.write_line(2, "}")?; // memory
        self.write_enter()?;

        self.write_line(2, "interface_timing : true;")?;
        self.write_line(2, "dont_use  : true;")?;
        self.write_line(2, "map_only   : true;")?;
        self.write_line(2, "dont_touch : true;")?;
        self.write_enter()?;

        self.write_pgpin()?;
        self.write_word_bus()?;
        self.write_address_bus()?;
        self.write_control_pins()?;

        self.write_line(1, "}")?; // cell

        Ok(())
    }

    fn write_units(&mut self) -> YouRAMResult<()> {
        self.write_line(1, "time_unit : \"1ns\";")?;
        self.write_line(1, "voltage_unit : \"1V\"")?;
        self.write_line(1, "current_unit : \"1mA\";")?;
        self.write_line(1, "resistance_unit : \"1kohm\";")?;
        self.write_line(1, "capacitive_load_unit(1, pF);")?;
        self.write_line(1, "leakage_power_unit : \"1mW\";")?;
        self.write_line(1, "pulling_resistance_unit :\"1kohm\";")?;
        self.write_line(1, "operating_conditions(OC) {")?;
        self.write_line(2, &format!("process : {};", self.pvt_process()))?;
        self.write_line(2, &format!("voltage : {}", self.pvt_voltage()))?;
        self.write_line(2, &format!("temperature : {}", self.pvt_temp()))?;
        self.write_line(1, "}")?;
        self.write_enter()?;
        Ok(())
    }

    fn write_defaults(&mut self) -> YouRAMResult<()> {
        self.write_line(1, &format!("input_threshold_pct_fall      : {};", self.pdk.input_threshold_pct_fall() * 100.0))?;
        self.write_line(1, &format!("input_threshold_pct_fall      : {};", self.pdk.input_threshold_pct_fall() * 100.0))?;
        self.write_line(1, &format!("output_threshold_pct_fall     : {};", self.pdk.output_threshold_pct_fall() * 100.0))?;
        self.write_line(1, &format!("input_threshold_pct_rise      : {};", self.pdk.input_threshold_pct_rise() * 100.0))?;
        self.write_line(1, &format!("output_threshold_pct_rise     : {};", self.pdk.output_threshold_pct_rise() * 100.0))?;
        self.write_line(1, &format!("slew_lower_threshold_pct_fall : {};", self.pdk.slew_lower_threshold_pct_fall() * 100.0))?;
        self.write_line(1, &format!("slew_upper_threshold_pct_fall : {};", self.pdk.slew_upper_threshold_pct_fall() * 100.0))?;
        self.write_line(1, &format!("slew_lower_threshold_pct_rise : {};", self.pdk.slew_lower_threshold_pct_rise() * 100.0))?;
        self.write_line(1, &format!("slew_upper_threshold_pct_rise : {};", self.pdk.slew_upper_threshold_pct_rise() * 100.0))?;
        
        // TODO nom
        self.write_line(1, "default_cell_leakage_power    : 0.0;")?;
        self.write_line(1, "default_leakage_power_density : 0.0;")?;
        self.write_line(1, "default_input_pin_cap         : 1.0;")?;
        self.write_line(1, "default_inout_pin_cap         : 1.0;")?;
        self.write_line(1, "default_output_pin_cap        : 0.0;")?;
        self.write_line(1, "default_max_transition        : 0.5;")?;
        self.write_line(1, "default_fanout_load           : 1.0;")?;
        self.write_line(1, "default_max_fanout            : 4.0;")?;
        self.write_line(1, "default_connection_class      : universal;")?;

        self.write_line(1, &format!("voltage_map ({}, {});", Sram::vdd_pn(), self.pvt_voltage()))?;
        self.write_line(1, &format!("voltage_map ({}, 0);", Sram::gnd_pn()))?;
        self.write_line(1, "default_operating_conditions : OC;")?;

        self.write_enter()?;

        Ok(())
    } 

    fn write_luttemplate(&mut self) -> YouRAMResult<()> {
        self.write_line(1, "lu_table_template(CELL_TABLE) {")?;
        self.write_line(2, "variable_1 : input_net_transition;")?;
        self.write_line(2, "variable_2 : total_output_net_capacitance;")?;
        self.write_time_index()?;
        self.write_cap_index()?;
        self.write_line(1, "}")?;
        
        self.write_enter()?;

        self.write_line(1, "lu_table_template(CONSTRAINT_TABLE) {")?;
        self.write_line(2, "variable_1 : related_pin_transition;")?;
        self.write_line(2, "variable_2 : constrained_pin_transition;")?;
        self.write_line(1, "}")?;

        self.write_enter()?;

        Ok(())
    }

    fn write_bus(&mut self) -> YouRAMResult<()> {
        self.write_line(1, "type (data) {")?;
        self.write_line(2, "base_type : array;")?;
        self.write_line(2, "data_type : bit;")?;
        self.write_line(2, &format!("bit_width : {};", self.sram.read().word_width()))?;
        self.write_line(2, &format!("bit_from : {};", self.sram.read().word_width() - 1))?;
        self.write_line(2, "bit_to : 0;")?;
        self.write_line(1, "}")?;
        self.write_enter()?;

        self.write_line(1, "type (addr) {")?;
        self.write_line(2, "base_type : array;")?;
        self.write_line(2, "data_type : bit;")?;
        self.write_line(2, &format!("bit_width : {};", self.sram.read().address_width()))?;
        self.write_line(2, &format!("bit_from : {};", self.sram.read().address_width() - 1))?;
        self.write_line(2, "bit_to : 0;")?;
        self.write_line(1, "}")?;
        self.write_enter()?;

        Ok(())    
    }

    fn write_pgpin(&mut self) -> YouRAMResult<()> {
        self.write_line(2, &format!("pg_pin({}) {{", Sram::vdd_pn()))?;
        self.write_line(3, &format!("voltage_name : {};", Sram::vdd_pn()))?;
        self.write_line(3, "pg_type : primary_power;")?;
        self.write_line(2, "}")?;
        self.write_enter()?;

        self.write_line(2, &format!("pg_pin({}) {{", Sram::gnd_pn()))?;
        self.write_line(3, &format!("voltage_name : {};", Sram::gnd_pn()))?;
        self.write_line(3, "pg_type : primary_ground;")?;
        self.write_line(2, "}")?;
        self.write_enter()?;
        Ok(())    
    }

    fn write_word_bus(&mut self) -> YouRAMResult<()> {
        self.write_word_bus_input()?;
        self.write_word_bus_output()?;
        Ok(())
    }
    
    fn write_word_bus_input(&mut self) -> YouRAMResult<()> {
        // Mark: din?
        self.write_line(2, "bus(din) {")?;
        self.write_line(3, "bus_type  : data;")?;
        self.write_line(3, "direction  : input;")?;
        self.write_line(3, "memory_write() {")?;
        self.write_line(4, "address : addr")?;
        self.write_line(4, "clocked_on  : clk")?;
        self.write_line(3, "}")?; // memory_write
        self.write_line(3, &format!("pin(din[{}:0]) {{", self.sram.read().word_width() - 1))?;
        self.write_dff_timing(4)?;
        self.write_line(3, "}")?; // pin
        self.write_line(2, "}")?; // bus

        self.write_enter()?;

        Ok(())
    }

    fn write_word_bus_output(&mut self) -> YouRAMResult<()> {
        self.write_line(2, "bus(dout) {")?;
        self.write_line(3, "bus_type  : data;")?;
        self.write_line(3, "direction  : output;")?;
        // self.write_line(3, "max_capacitance : ");
        // self.write_line(3, "min_capacitance : ");
        self.write_line(3, "memory_read() {")?;
        self.write_line(4, "address : addr")?;
        self.write_line(3, "}")?; // memory_read()
        // Mark: dout
        self.write_line(3, &format!("pin(dout[{}:0]) {{",  self.sram.read().word_width() - 1))?;
        self.write_timing_charz(4)?;
        self.write_line(3, "}")?; // pin
        self.write_line(2, "}")?; // bus

        self.write_enter()?;
        
        Ok(())
    }

    fn write_address_bus(&mut self) -> YouRAMResult<()> {
        self.write_line(2, "bus(addr) {")?;
        self.write_line(3, "bus_type  : addr;")?;
        self.write_line(3, "direction  : input;")?;
        // self.write_line(2, "max_capacitance : ");
        // self.write_line(2, "min_capacitance : ");
        self.write_line(3, "memory_read() {")?;
        self.write_line(3, "address : addr")?;
        self.write_line(3, "}")?; // memory_read()
        self.write_line(3, &format!("pin(addr[{}:0]) {{", self.sram.read().address_width()))?;
        self.write_dff_timing(4)?;
        self.write_line(3, "}")?; // pin
        self.write_line(2, "}")?;// bus

        self.write_enter()?;

        Ok(())
    }

    fn write_control_pins(&mut self) -> YouRAMResult<()> {
        self.write_line(2, &format!("pin({}) {{", Sram::chip_sel_bar_pn()))?;
        self.write_line(3, "direction  : input;")?;
        self.write_dff_timing(3)?;
        self.write_line(2, "}")?; // pin
        self.write_enter()?;

        self.write_line(2, &format!("pin({}) {{", Sram::write_enable_pn()))?;
        self.write_line(3, "direction  : input;")?;
        self.write_dff_timing(3)?;
        self.write_line(2, "}")?; // pin
        self.write_enter()?;

        self.write_line(2, &format!("pin({}) {{", Sram::clock_pn()))?;
        self.write_line(3, "direction  : input;")?;
        self.write_line(2, "}")?; // pin
        self.write_enter()?;

        Ok(())
    }

    fn write_timing_charz(&mut self, indent: usize) -> YouRAMResult<()> {
        /*
            timing(){ 
                timing_sense : non_unate; 
                related_pin : "clk"; 
                timing_type : falling_edge; 
                cell_rise(CELL_TABLE) {
                    ...
                }
                cell_fall(CELL_TABLE) {
                    ...
                }
                rise_transition(CELL_TABLE) {
                    ...
                }
                fall_transition(CELL_TABLE) {
                    ...
                }
            }
        */

        self.write_line(indent, "timing() {")?;

        self.write_line(indent + 1, "timing_sense : non_unate;")?;
        self.write_line(indent + 1, &format!("related_pin  : \"{}\";", Sram::clock_pn()))?;
        self.write_line(indent + 1, "timing_type : rising_edge;")?;

        self.write_line(indent + 1, "cell_rise(CELL_TABLE) {")?;
        self.write_values(indent + 2, &Self::transform_times(&self.delay_lhs))?;
        self.write_line(indent + 1, "}")?; // cell_rise

        self.write_line(indent + 1, "cell_fall(CELL_TABLE) {")?;
        self.write_values(indent + 2, &Self::transform_times(&self.delay_hls))?;
        self.write_line(indent + 1, "}")?; // cell_fall

        self.write_line(indent + 1, "rise_transition(CELL_TABLE) {")?;
        self.write_values(indent + 2, &Self::transform_times(&self.slew_lhs))?;
        self.write_line(indent + 1, "}")?; // rise_transition

        self.write_line(indent + 1, "fall_transition(CELL_TABLE) {")?;
        self.write_values(indent + 2, &Self::transform_times(&self.slew_hls))?;
        self.write_line(indent + 1, "}")?; // fall_transition

        self.write_line(indent, "}")?;  // timing

        Ok(())
    }

    fn write_dff_timing(&mut self, indent: usize) -> YouRAMResult<()> {
        let dff = self.pdk.get_dff(DriveStrength::X1).unwrap();

        /*
            timing(){ 
                timing_type : setup_rising; 
                related_pin  : "clk"; 
                rise_constraint(CONSTRAINT_TABLE) {
                    ...
                fall_constraint(CONSTRAINT_TABLE) {
                    ...
                }
            }
            timing(){ 
                timing_type : hold_rising; 
                related_pin  : "clk0"; 
                rise_constraint(CONSTRAINT_TABLE) {
                    ...
                }
                fall_constraint(CONSTRAINT_TABLE) {
                    ...
                }
            }
        */
        let setup_rising = &dff.read().setup_rising_timing;
        self.write_line(indent, "timing() {")?;

        self.write_line(indent + 1, "timing_type : setup_rising;")?;
        self.write_line(indent + 1, &format!("related_pin  : \"{}\";", Sram::clock_pn()))?;

        self.write_line(indent + 1, "rise_constraint(CONSTRAINT_TABLE) {")?;
        self.write_lutable(indent + 2, setup_rising.rise_constraint.as_ref().unwrap())?;
        self.write_line(indent + 1, "}")?;  // rise_constraint

        self.write_line(indent + 1, "fall_constraint(CONSTRAINT_TABLE) {")?;
        self.write_lutable(indent + 2, setup_rising.fall_constraint.as_ref().unwrap())?;
        self.write_line(indent + 1, "}")?;  // rise_constraint

        self.write_line(indent, "}")?;  // timing
        
        //////////////////////////////////////////////////////////

        let hold_rising = &dff.read().hold_rising_timing;
        self.write_line(indent, "timing() {")?;

        self.write_line(indent + 1, "timing_type : hold_rising;")?;
        self.write_line(indent + 1, &format!("related_pin  : \"{}\";", Sram::clock_pn()))?;

        self.write_line(indent + 1, "rise_constraint(CONSTRAINT_TABLE) {")?;
        self.write_lutable(indent + 2, hold_rising.rise_constraint.as_ref().unwrap())?;
        self.write_line(indent + 1, "}")?;  // rise_constraint

        self.write_line(indent + 1, "fall_constraint(CONSTRAINT_TABLE) {")?;
        self.write_lutable(indent + 2, hold_rising.fall_constraint.as_ref().unwrap())?;
        self.write_line(indent + 1, "}")?;  // rise_constraint

        self.write_line(indent, "}")?; // timing
        
        Ok(())
    } 

    fn write_lutable(&mut self, indent: usize, lutable: &LibLuTable) -> YouRAMResult<()> {
        /*
            index_1: "",
            index_2: "",
            values(\
                "0.006, 0.008, 0.015",\
                "0.006, 0.008, 0.015",\
                "0.006, 0.008, 0.015"\
            );
        */
        self.write_index(indent, 1, lutable.index_1.as_ref().unwrap())?;
        self.write_index(indent, 2, lutable.index_2.as_ref().unwrap())?;
        self.write_values(indent, &lutable.values)?;
        
        Ok(())
    }

    fn write_time_index(&mut self) -> YouRAMResult<()> {
        let mut ss = String::new();

        write!(ss, "index_1(\"")?;
    
        let mut first_flag = true;
        for v in self.input_net_transitions.iter() {
            if !first_flag {
                write!(ss, ", ").unwrap();
            } else {
                first_flag = false;
            }
            write!(ss, "{}", Self::time_value(*v))?;
        }
    
        write!(ss, "\")")?;
        
        self.write_line(2, &ss)?;

        Ok(())
    }

    fn write_cap_index(&mut self) -> YouRAMResult<()> {
        let mut ss = String::new();

        write!(ss, "index_2(\"")?;
    
        let mut first_flag = true;
        for v in self.output_net_capacitances.iter() {
            if !first_flag {
                write!(ss, ", ").unwrap();
            } else {
                first_flag = false;
            }
            write!(ss, "{}", Self::cap_value(*v))?;
        }
    
        write!(ss, "\")")?;
        
        self.write_line(2, &ss)?;

        Ok(())
    }

    fn write_index(&mut self, indent: usize, index: usize,  values: &[f64]) -> YouRAMResult<()> {
        let mut ss = String::new();

        write!(ss, "index_{index}(\"")?;
    
        let mut first_flag = true;
        for v in values.iter() {
            if !first_flag {
                write!(ss, ", ").unwrap();
            } else {
                first_flag = false;
            }
            write!(ss, "{}", *v)?;
        }
    
        write!(ss, "\")")?;
        
        self.write_line(indent, &ss)?;

        Ok(())
    }

    fn write_values(&mut self, indent: usize, values: &Vec<Vec<f64>>) -> YouRAMResult<()> {
        /*
            values(\
                "0.006, 0.008, 0.015",\
                "0.006, 0.008, 0.015",\
                "0.006, 0.008, 0.015"\
            );
        */
        self.write_line(indent, "values(")?;
        
        for (i, row) in values.iter().enumerate() {
            let mut line = String::new();
            write!(line, "    \"")?;
            
            for (j, v) in row.iter().enumerate() {
                if j > 0 {
                    write!(line, ", ")?;
                }
                write!(line, "{:.6}", v)?;
            }
            
            write!(line, "\"")?;

            if i < values.len() - 1 {
                write!(line, ",\\")?;
            }
            
            self.write_line(indent + 1, &line)?;
        }

        self.write_line(indent, ");")?;

        Ok(())
    }

}

impl LibertyWritor {
    fn write_line(&mut self, indent: usize, s: &str) -> YouRAMResult<()> {
        for _ in 0..indent {
            self.writor.write_all("    ".as_bytes())?;
        }
        self.writor.write_all(s.as_bytes())?;
        self.writor.write_all("\n".as_bytes())?;
        Ok(())
    } 

    fn write_enter(&mut self) -> YouRAMResult<()> {
        self.writor.write_all("\n".as_bytes())?;
        Ok(())
    }
}

impl LibertyWritor {
    fn transform_times(times: &Vec<Vec<Time>>) -> Vec<Vec<f64>> {
        times.iter().map(|ts| {
            ts.iter().map(|t| Self::time_value(*t)).collect()
        })
        .collect()
    }

    fn pvt_process(&self) -> f64 {
        // TODO: save process value?
        match self.pvt.process {
            Process::FastFast => 1.1,
            Process::SlowSlow => 0.9,
            Process::TypeType => 1.0 
        }
    }

    fn pvt_voltage(&self) -> f64 {
        Self::voltage_value(self.pvt.voltage)
    }

    fn pvt_temp(&self) -> f64 {
        Self::temp_value(self.pvt.temperature)
    }

    fn voltage_value(voltage: Voltage) -> f64 {
        // 1v
        voltage.value().to_f64()
    }

    fn time_value(time: Time) -> f64 {
        // 1ns
        time.value().to_f64() * 1e9
    }

    fn cap_value(cap: Capacitance) -> f64 {
        cap.value().to_f64() * 1e12
    }

    fn temp_value(temp: Temperature) -> f64 {
        temp.value().to_f64()
    }
}

