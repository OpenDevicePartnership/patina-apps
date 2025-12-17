# Core Services Benchmarks

`services_benchmark_test` compares core service performance between the Patina (Rust) implementation and canonical C implementation.

## Usage

To build and use the benchmark tool:

### Build Benchmark Package

```bash
# Build a specific package
cargo make --env PACKAGE=services_benchmark_test build-package
```

## Benchmark Categories

The benchmark suite tests the performance of 30 different UEFI Boot Services across 6 categories.
For more information on boot services, see the [UEFI spec](https://uefi.org/specs/UEFI/2.9_A/07_Services_Boot_Services.html).

### Iterations

The number of iterations per benchmark is derived from operation counts during normal operation of the Patina core.
The exact counts can be found in [memory_safety_strategy.md in Patina](https://opendevicepartnership.github.io/patina/background/memory_safety_strategy.html).
The benchmarks here use similar orders of magnitude rather than exact counts.

### 1. Controller Services

#### `connect_controller` (100 iterations)

**File**: `bench/controller.rs`

Benchmarks the UEFI driver model's controller connection mechanism. This primarily measures device driver performance
in UEFI systems.

### 2. Event Services

#### `bench_check_event_signaled` (10000 iterations)  

**File**: `bench/event.rs`

Benchmarks checking the state of an already-signaled event. This is the fast path of `check_event`.

#### `bench_check_event_unsignaled` (10000 iterations)

**File**: `bench/event.rs`

Benchmarks checking the state of an unsignaled event.
This is the slow path of `check_event` and is important for event polling scenarios.

#### `create_event` (1000 iterations)

**File**: `bench/event.rs`

Benchmarks event creation performance.

#### `close_event` (1000 iterations)

**File**: `bench/event.rs`

Benchmarks event cleanup (close) performance.

#### `signal_event` (100000 iterations)

**File**: `bench/event.rs`

Benchmarks individual event signaling.

#### `signal_event_group` (100 iterations)

**File**: `bench/event.rs`

Benchmarks signaling multiple events as a group.
The time taken by `signal_event` is scales with the number of events in the group,
so this benchmark gradually increases the number of events by 1 per iteration.

### 3. Image Services

#### `start_image, exit` (100 iterations)

**File**: `bench/image.rs`

Benchmarks UEFI image execution performance through a no-op image that exits immediately.
This does not benchmark an individual function as it is difficult to measure `start_image` and `exit` indpendently.
Instead, this benchmark roughly measures the performance of a complete image execution lifecycle.

#### `load_image` (100 iterations)

**File**: `bench/image.rs`

Benchmarks UEFI image loading performance.

### 4. Memory Services

#### `allocate_pages` (1000 iterations)

**File**: `bench/memory.rs`

Benchmarks page-level memory allocation (with size 1 page / 4KB).

#### `allocate_pool` (10000 iterations)  

**File**: `bench/memory.rs`

Benchmarks pool memory allocation (of size 1KB). Models smaller, more frequent memory allocations as compared to `allocate_pages`.

#### `free_pages` (100 iterations)

**File**: `bench/memory.rs`

Benchmarks page deallocation performance.

#### `free_pool` (10000 iterations)

**File**: `bench/memory.rs`

Benchmarks pool memory deallocation.
Like `allocate_pool`, this represents smaller, more frequent memory allocations in the core.

#### `copy_mem` (10 iterations)

**File**: `bench/memory.rs`

Benchmarks memory copying performance. This is not currently used in the Patina DXE core.

#### `set_mem` (10 iterations)

**File**: `bench/memory.rs`

Benchmarks memory initialization performance. This is not currently used in the Patina DXE core.

#### `get_memory_map` (10 iterations)

**File**: `bench/memory.rs`

Benchmarks system memory map retrieval. This is critical for OS loaders and memory managers.

### 5. Miscellaneous Services

#### `calculate_crc32` (100 iterations)

**File**: `bench/misc.rs`

Benchmarks checksum calculation performance (over 128 bytes of data).

#### `install_configuration_table` (10 iterations)

**File**: `bench/misc.rs`

Benchmarks configuration table installation.

### 6. Protocol Services

#### `install_protocol_interface` (100 iterations)

**File**: `bench/protocol.rs`

Benchmarks protocol installation performance.

#### `open_protocol` (10000 iterations)

**File**: `bench/protocol.rs`

Benchmarks protocol access performance. This is the preferred method for retrieving protocol interfaces in modern UEFI (2.0+).

#### `handle_protocol` (10000 iterations)

**File**: `bench/protocol.rs`

Benchmarks protocol access. This is a legacy method but is still included due to needing to support legacy UEFI (1.0).

#### `close_protocol` (100 iterations)

**File**: `bench/protocol.rs`

Benchmarks protocol cleanup performance.

#### `locate_device_path` (100 iterations)

**File**: `bench/protocol.rs`

Benchmarks device path resolution.

#### `open_protocol_information` (100 iterations)

**File**: `bench/protocol.rs`

Benchmarks protocol metadata retrieval.

#### `protocols_per_handle` (100 iterations)

**File**: `bench/protocol.rs`

Tests handle protocol enumeration.

#### `register_protocol_notify` (10 iterations)

**File**: `bench/protocol.rs`

Benchmarks protocol notification registration. This is used infrequently in the Patina DXE core.

#### `reinstall_protocol_interface` (100 iterations)

**File**: `bench/protocol.rs`

Benchmarks protocol update performance. This sometimes triggers `connect/disconnect_controller`
and can be more time-consuming than `install_protocol_interface`.

#### `uninstall_protocol_interface` (10 iterations)

**File**: `bench/protocol.rs`

Benchmarks protocol removal performance. This is used infrequently in the Patina DXE core.

### 7. Task Priority Level (TPL) Services

#### `raise_tpl` (1000000 iterations)

**File**: `bench/tpl.rs`

Tests interrupt disable performance. Raises to TPL_HIGH_LEVEL to test the maximum performance impact of interrupts.

#### `restore_tpl` (1000000 iterations)

**File**: `bench/tpl.rs`

Tests interrupt restore performance. Uses all TPL levels to test the performance impact of restoring to each level.

## Performance Characteristics

The benchmarks measure cycle counts using CPU performance counters, providing:

- **Total Cycles**: Raw CPU cycles consumed
- **Cycles/Operation**: Average cycles per function call  
- **Total Time**: Wall-clock time in milliseconds
- **Statistical Data**: Min, max, and standard deviation
- **Call Count**: Number of iterations for statistical significance

## Output Format

Results are displayed as a markdown table in the UEFI shell (one sample row shown below):

```plain-text
| Name               | Total cycles | Total calls | Cycles/op | Total time (ms) | Min cycles | Max cycles | SD [cycles] |
| ------------------ | ------------ | ----------- | --------- | --------------- | ---------- | ---------- | ----------- |
| connect_controller | 1234567      | 100         | 12345.67  | 45.67           | 10000      | 15000      | 1500        |
```
