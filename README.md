# pybinwalk

Python bindings for [Binwalk v3](https://github.com/ReFirmLabs/binwalk), built with PyO3 and maturin.

Provides direct access to binwalk's signature scanning and extraction from Python. Also exposes a C API for use from other languages.

## Requirements

- Python >= 3.8
- Rust toolchain (for building from source)

## Installation

```
pip install pybinwalk
```

To build from source:

```
pip install maturin
maturin build --release
pip install target/wheels/pybinwalk-*.whl
```

## Usage

### Scan bytes in memory

```python
import pybinwalk

bw = pybinwalk.Binwalk()
data = open("firmware.bin", "rb").read()

for result in bw.scan_bytes(data):
    print(f"0x{result.offset:08X}  {result.name}  {result.description}")
```

### Scan a file from disk

```python
bw = pybinwalk.Binwalk()

for result in bw.scan_path("/path/to/firmware.bin"):
    print(f"0x{result.offset:08X}  {result.name}  {result.description}")
```

### Configure with filters

```python
bw = pybinwalk.Binwalk.configure(
    include=["gzip", "squashfs"],
    exclude=["jpeg"],
    full_search=False,
)

results = bw.scan_path("firmware.bin")
```

### Extract

```python
bw = pybinwalk.Binwalk.configure(
    target_file_name="firmware.bin",
    output_directory="./extracted",
)

analysis = bw.analyze_path(bw.base_target_file, do_extraction=True)

for sig in analysis.file_map:
    print(f"0x{sig.offset:08X}  {sig.name}")

for sig_id, extraction in analysis.extractions.items():
    print(f"{sig_id}: success={extraction.success}, dir={extraction.output_directory}")
```

### Version

```python
print(pybinwalk.version())
```

## API reference

### `Binwalk`

| Method | Description |
|---|---|
| `Binwalk()` | Create an instance with default settings. |
| `Binwalk.configure(...)` | Create an instance with custom options. |
| `scan_bytes(data: bytes)` | Scan in-memory data. Returns `list[SignatureResult]`. |
| `scan_path(file_path: str)` | Scan a file on disk. Returns `list[SignatureResult]`. |
| `extract_bytes(data, file_path, file_map)` | Extract from in-memory data using a prior scan result. Returns `dict[str, ExtractionResult]`. |
| `analyze_path(target_file, do_extraction)` | Scan and optionally extract a file. Returns `AnalysisResults`. |

Properties: `signature_count`, `base_target_file`, `base_output_directory`.

### `Binwalk.configure()` parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `target_file_name` | `str or None` | `None` | Path to target file (required for extraction). |
| `output_directory` | `str or None` | `None` | Directory for extracted files. |
| `include` | `list[str] or None` | `None` | Only scan for these signature types. |
| `exclude` | `list[str] or None` | `None` | Skip these signature types. |
| `full_search` | `bool` | `False` | Search all offsets instead of optimized locations. |

### `SignatureResult`

Properties: `offset`, `id`, `size`, `name`, `confidence`, `description`, `always_display`, `extraction_declined`, `preferred_extractor`.

### `ExtractionResult`

Properties: `size`, `success`, `extractor`, `do_not_recurse`, `output_directory`.

### `AnalysisResults`

Properties: `file_path`, `file_map` (list of `SignatureResult`), `extractions` (dict mapping signature id to `ExtractionResult`).

## C API

When built without the `python` feature (`cargo build --release --no-default-features`), the library exposes a C-compatible API:

| Function | Description |
|---|---|
| `bw_scan(data, len)` | Scan a buffer. Returns JSON string. |
| `bw_scan_with_options(data, len, options_json)` | Scan with filter options. Returns JSON string. |
| `bw_version()` | Get version info. Returns JSON string. |
| `bw_list_signatures()` | List available signatures. Returns JSON string. |
| `bw_free(ptr)` | Free a string returned by any of the above. |

All C API functions return a `*mut c_char` pointer to a JSON string. The caller must free it with `bw_free()`.

## License

MIT
