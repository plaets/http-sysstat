use systemstat::{System, Platform, CPULoad, data::DelayedMeasurement};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::interval_collector::{IntervalCollector, IntervalCollectorHandle};

pub fn create_cpu_collector() -> IntervalCollectorHandle<CPULoad> {
    let mut collector = IntervalCollector::new();
    collector
        .interval(Duration::from_millis(5000))
        .collect(|cpu: Arc<Mutex<Option<CPULoad>>>, measurement: Arc<Mutex<Option<Mutex<DelayedMeasurement<CPULoad>>>>>| {
            //yes, a double mutex
            //i want to die
            let sys = System::new();
            let measurement = measurement.clone();
            let mut measurement_guard = measurement.lock().unwrap(); //see if i care
            if let Some(measurement) = &*measurement_guard {
                let mut measurement_guard = measurement.lock().unwrap();       
                let cpu = cpu.clone();
                let mut cpu_guard = cpu.lock().unwrap();
                *cpu_guard = Some(measurement_guard.done().unwrap());
            }
            *measurement_guard = Some(Mutex::new(sys.cpu_load_aggregate().unwrap()));
        });

    collector.start()
}
