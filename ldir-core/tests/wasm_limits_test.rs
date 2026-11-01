//! Resource-limit seam tests for the Wasm plugin host (wasmtime 42).
//!
//! These pin the contract introduced by the wasmtime-wasi 38 → 42 bump:
//! `ResourceLimitEnforcer` must produce an engine that meters fuel, inject
//! the configured fuel budget into stores, and enforce output-size and
//! wall-clock limits around plugin execution.

#![cfg(feature = "wasm-plugins")]

use ldir_core::plugin::wasm_host::ResourceLimitEnforcer;
use ldir_core::wasm_plugins::manifest::ResourceLimits;
use wasmtime::{Engine, Instance, Module, Store};

fn enforcer(fuel: u64, output_kb: u32, time_ms: u32) -> ResourceLimitEnforcer {
    ResourceLimitEnforcer::new(&ResourceLimits {
        max_fuel: fuel,
        max_memory_mb: 64,
        max_time_ms: time_ms,
        max_output_kb: output_kb,
    })
}

/// A WAT module that spins forever, so finite fuel must trap it.
const INFINITE_LOOP: &str = r#"
    (module
        (func (export "spin")
            (loop $l (br $l))
        )
    )
"#;

#[test]
fn engine_config_enables_fuel_metering() {
    let enforcer = enforcer(1_000, 1, 1_000);
    let engine = Engine::new(&enforcer.create_engine_config()).unwrap();
    let module = Module::new(&engine, INFINITE_LOOP).unwrap();

    let mut store = Store::new(&engine, ());
    enforcer.configure_store(&mut store);
    assert_eq!(store.get_fuel().unwrap(), 1_000, "fuel budget not injected");

    let instance = Instance::new(&mut store, &module, &[]).unwrap();
    let spin = instance
        .get_typed_func::<(), ()>(&mut store, "spin")
        .unwrap();

    let result = spin.call(&mut store, ());
    assert!(
        result.is_err(),
        "infinite loop must trap once the fuel budget is exhausted"
    );
    assert!(
        store.get_fuel().unwrap() < 1_000,
        "fuel must have been consumed by the loop"
    );
}

#[test]
fn output_size_limit_is_enforced() {
    let enforcer = enforcer(1_000, 1, 1_000);
    assert!(enforcer.check_output_size(&[0u8; 512]).is_ok());
    assert!(enforcer.check_output_size(&[0u8; 2048]).is_err());
}

#[test]
fn wall_clock_limit_is_enforced() {
    let enforcer = enforcer(1_000, 1, 0);
    // max_time_ms = 0: any measurable execution exceeds the budget.
    let result =
        enforcer.execute_with_limits(|| std::thread::sleep(std::time::Duration::from_millis(5)));
    assert!(
        result.is_err(),
        "execution past the wall-clock budget must fail"
    );
}
