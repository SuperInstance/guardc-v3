# guardc-v3

**GUARD → FLUX-C compiler.** Translates human-readable constraint definitions to a 60-opcode terminating bytecode.

Early-stage. 22 tests passing. Compiles all 10 industry presets. Not yet used in production.

## What GUARD Looks Like

```
GUARD coolant_temp in [-40, 150] with priority HIGH
GUARD engine_rpm in [0, 8000] with priority CRITICAL
```

## What It Produces

FLUX-C bytecode that the [flux-vm-v3](https://github.com/SuperInstance/flux-vm-v3) executes or JIT-compiles to native code. The compilation chain produces a SHA-256 hash at each stage:

```
source_hash → ast_hash → cir_hash → bytecode_hash → check_hash
```

Tampering with any link invalidates the chain.

## Test Results

```
cargo test
22 passed, 0 failed
```

All 10 industry presets compile successfully:
- aviation_adsb, automotive_can, maritime_nmea, medical_fhir
- energy_scada, nuclear_reactor, railway_ertms, robotics
- space_telemetry, underwater_acoustic

## Honest Limitations

- The GUARD DSL supports only simple range constraints (no conditional logic yet)
- No type inference — bounds are always f64
- The proof chain is SHA-256 hashes, not formal mathematical proofs
- No optimization passes — compiles straightforwardly
- Error messages are functional but not user-friendly

## Related

- [flux-vm-v3](https://github.com/SuperInstance/flux-vm-v3) — the VM that runs the bytecode
- [constraint-theory-ecosystem](https://github.com/SuperInstance/constraint-theory-ecosystem) — 96 language implementations

## License

MIT
