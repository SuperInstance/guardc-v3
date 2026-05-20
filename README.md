# guardc-v3

Compiler from GUARD constraint DSL to FLUX-C bytecode.

---

## What GUARD Looks Like

```
constraint coolant_temp: -40.0 <= x <= 150.0 severity CAUTION
constraint rpm: 0.0 <= x <= 8000.0 severity CRITICAL
constraint battery_voltage: 10.5 <= x <= 15.0 severity WARNING
```

GUARD is a small language for defining numeric constraints. Each constraint has:
- A name
- Lower and upper bounds
- A severity level (PASS / CAUTION / WARNING / CRITICAL)

The compiler turns these into FLUX-C bytecode that the [flux-vm](https://github.com/SuperInstance/flux-vm-v3) executes.

## Quick Start

```bash
git clone https://github.com/SuperInstance/guardc-v3
cd guardc-v3
cargo build --release
cargo test
```

### Compile a GUARD file

```bash
guardc compile constraints.guard -o output.flux
```

### Compile a preset directly

```bash
guardc preset automotive_can -o auto.flux
```

## Pipeline

```
GUARD source
    ↓  Lexer
Tokens
    ↓  Parser
AST (constraint definitions)
    ↓  Codegen
FLUX-C bytecode (60 opcodes)
    ↓  Proof module
SHA-256 source hash (anchors the proof chain)
```

## Presets

10 industry presets with realistic constraint sets. Each preset compiles to valid FLUX-C bytecode:

```bash
guardc list-presets
# automotive_can, aviation_adsb, medical_fhir, financial_fix,
# energy_scada, iot_mqtt, maritime_nmea, nuclear_reactor,
# railway_ertms, robotics
```

## GUARD Syntax Reference

```
constraint <name>: <lo> <= x <= <hi> severity <level>
```

- `name`: identifier (letters, digits, underscore)
- `lo`, `hi`: floating-point literals
- `level`: `PASS` | `CAUTION` | `WARNING` | `CRITICAL`

### Example: Automotive CAN Bus

```
constraint engine_rpm: 0.0 <= x <= 8000.0 severity CRITICAL
constraint coolant_temp: -40.0 <= x <= 150.0 severity CAUTION
constraint battery_voltage: 10.5 <= x <= 15.0 severity WARNING
constraint throttle_pos: 0.0 <= x <= 100.0 severity CAUTION
constraint fuel_level: 0.0 <= x <= 100.0 severity PASS
constraint speed_kmh: 0.0 <= x <= 300.0 severity WARNING
```

## Test Results

```
running 22 tests
test result: ok. 22 passed; 0 failed; 0 ignored
```

## What to Read Next

| If you want to... | Go to... |
|---|---|
| Execute the compiled bytecode | [flux-vm-v3](https://github.com/SuperInstance/flux-vm-v3) |
| See the full constraint ecosystem | [constraint-theory-ecosystem](https://github.com/SuperInstance/constraint-theory-ecosystem) |
| Use standalone fracture in Rust | [flux-fracture](https://github.com/SuperInstance/flux-fracture) |
| Use standalone fracture in C | [flux-fracture-c](https://github.com/SuperInstance/flux-fracture-c) |

## License

MIT
