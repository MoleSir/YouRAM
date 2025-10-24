use reda_lib::model::{LibLibrary, LibOperatingConditions};
use reda_unit::{Capacitance, Temperature, Time, Voltage};
use crate::{ErrorContext, YouRAMResult};
use super::{PdkError, Process, Pvt};

#[derive(Debug, Clone)]
pub struct PdkInformation {
    pub name: String,

    pub pvt: Pvt,

    pub nom_process: Option<f64>,
    pub nom_temperature: Option<Temperature>,
    pub nom_voltage: Option<Voltage>,

    pub default_inout_pin_cap: Option<Capacitance>,
    pub default_input_pin_cap: Option<Capacitance>,
    pub default_output_pin_cap: Option<Capacitance>,
    pub default_fanout_load: Option<Capacitance>,
    pub default_max_transition: Option<Time>,
}

impl PdkInformation {
    pub fn load(library: &LibLibrary) -> YouRAMResult<Self> {
        let time_unit = library.time_unit;
        let capacitance_unit = library.capacitive_load_unit.unwrap_or_default();

        let nom_process = library.nom_process;
        let nom_temperature = library.nom_temperature.map(Temperature::from);
        let nom_voltage = library.nom_voltage.map(Voltage::from);

        let default_inout_pin_cap = library.default_inout_pin_cap
            .map(|value| Capacitance::from(capacitance_unit.value() * value) );
        let default_input_pin_cap = library.default_input_pin_cap
            .map(|value| Capacitance::from(capacitance_unit.value() * value) );
        let default_output_pin_cap = library.default_output_pin_cap
            .map(|value| Capacitance::from(capacitance_unit.value() * value) );
        let default_fanout_load = library.default_fanout_load
            .map(|value| Capacitance::from(capacitance_unit.value() * value) );
        let default_max_transition = library.default_max_transition
            .map(|value| Time::from(time_unit.value() * value) );

        let pvt = Self::extract_pvt(library).context("extract pvt")?;

        Ok(Self {
            name: library.name.clone(),
            pvt,
            nom_process,
            nom_temperature,
            nom_voltage,
            default_inout_pin_cap,
            default_input_pin_cap,
            default_output_pin_cap,
            default_fanout_load,
            default_max_transition,
        })
        
    }

    fn extract_pvt(library: &LibLibrary) -> YouRAMResult<Pvt> {
        let voltage_unit = library.voltage_unit;

        let transform_to_pvt = |oc: &LibOperatingConditions| {
            let process = match oc.process {
                1.0 => Process::TypeType,
                p if p > 1.0 => Process::FastFast,
                _ => Process::SlowSlow,
            };
            Pvt::new(process, oc.voltage * voltage_unit.value(), oc.temperature)
        };

        match library.default_operating_conditions.as_ref() {
            Some(default_operating_conditions) => {
                for oc in library.operating_conditions.iter() {
                    if oc.name.as_str() == default_operating_conditions {
                        return Ok(transform_to_pvt(oc));
                    }
                }

                Err(PdkError::DefaultOperatingConditionsNotFound(default_operating_conditions.to_string()))?
            }
            None => {
                // Find first operating_conditions
                let oc = library.operating_conditions
                    .first()
                    .ok_or_else(|| PdkError::OperatingConditionsNotFound)?;

                Ok(transform_to_pvt(oc))
            }
        }
    }
}