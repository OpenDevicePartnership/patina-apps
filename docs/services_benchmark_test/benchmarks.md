# Core Services Benchmarks

`services_benchmark_test` compares core service performance between the Patina (Rust) implementation and canonical C implementation.

## Usage

To build and use the benchmark tool:

### Build Benchmark Package

```bash
# Build a specific package
cargo make build-package --env PACKAGE=services_benchmark_test
```

## Benchmark Categories

The benchmark suite tests 30 different UEFI Boot Services across 6 categories. For more information on boot services,
see the [UEFI spec](https://uefi.org/specs/UEFI/2.9_A/07_Services_Boot_Services.html).

### Iterations

The number of iterations per benchmark is derived from operation counts during normal operation of the Patina core.
The exact counts can be found in [memory_safety_strategy.md in Patina](https://github.com/OpenDevicePartnership/patina/blob/main/docs/src/background/memory_safety_strategy.md?plain=1).
The benchmarks here use similar orders of magnitude rather than exact counts.

### 1. Controller Services

#### `connect_controller` (100 iterations)

**File**: `bench/controller.rs`

Tests the UEFI driver model's controller connection mechanism. This benchmark:

- Creates mock driver binding protocols with `supported`, `start`, and `stop` functions
- Sets up controller and driver handles with test protocols
- Measures the time to connect a driver to a controller using `ConnectController()`
- Tests the core driver binding and device management infrastructure

This is critical for device driver performance in UEFI systems.

### 2. Event Services

#### `bench_check_event_signaled` (10000 iterations)  

**File**: `bench/event.rs`

Tests checking the state of an already-signaled event:

- Creates a `NOTIFY_WAIT` event with a test notification function
- Signals the event before measurement
- Measures `CheckEvent()` performance on a signaled event
- Validates fast-path event state checking

#### `bench_check_event_unsignaled` (10000 iterations)

**File**: `bench/event.rs`

Tests checking the state of an unsignaled event:

- Creates a `NOTIFY_WAIT` event without signaling it
- Measures `CheckEvent()` performance on an unsignaled event  
- Tests the negative/false case of event checking
- Important for event polling scenarios

#### `create_event` (1000 iterations)

**File**: `bench/event.rs`

Measures event creation performance:

- Times `CreateEvent()` with `NOTIFY_WAIT` type
- Uses `NOTIFY` TPL and test notification function
- Tests event object allocation and initialization
- Critical for systems that create many events

#### `close_event` (1000 iterations)

**File**: `bench/event.rs`

Measures event cleanup performance:

- Pre-creates events then times `CloseEvent()`
- Tests event object deallocation and cleanup
- Important for resource management

#### `signal_event` (100000 iterations)

**File**: `bench/event.rs`

Tests individual event signaling:

- Creates events and measures `SignalEvent()` performance
- Tests single event notification mechanism
- High iteration count due to lightweight nature

#### `signal_event_group` (100 iterations)

**File**: `bench/event.rs`

Tests signaling multiple events as a group:

- Creates a group of 5 events to simulate event group functionality
- Measures the time to signal all events in the group sequentially
- Simulates complex event notification scenarios
- Tests bulk event operations

### 3. Image Services

#### `start_image, exit` (100 iterations)

**File**: `bench/image.rs`

Tests UEFI image execution performance:

- Uses a pre-compiled `NoopImage.efi` that exits immediately
- Loads the image then measures `StartImage()` through image exit
- Tests the complete image execution lifecycle
- Critical for boot time performance

#### `load_image` (100 iterations)

**File**: `bench/image.rs`

Measures UEFI image loading performance:

- Times `LoadImage()` using embedded `NoopImage.efi` bytes
- Tests image parsing, validation, and memory setup
- Important for understanding boot loader performance

### 4. Memory Services

#### `allocate_pages` (1000 iterations)

**File**: `bench/memory.rs`

Tests page-level memory allocation:

- Allocates 1 page (4KB) using `AllocatePages()`
- Uses `ACPI_MEMORY_NVS` memory type and `AnyPage` allocation type
- Measures core memory management performance
- Frees pages after measurement for cleanup

#### `allocate_pool` (10000 iterations)  

**File**: `bench/memory.rs`

Tests pool memory allocation:

- Allocates 1KB (PAGE_SIZE/4) using `AllocatePool()`
- Uses `ACPI_MEMORY_NVS` memory type
- Tests smaller, more frequent allocations
- Higher iteration count reflects typical usage patterns

#### `free_pages` (100 iterations)

**File**: `bench/memory.rs`

Measures page deallocation performance:

- Pre-allocates pages then times `FreePages()`
- Tests memory management cleanup performance
- Lower iterations due to allocation overhead

#### `free_pool` (10000 iterations)

**File**: `bench/memory.rs`

Measures pool memory deallocation:

- Pre-allocates pool then times `FreePool()`
- Tests frequent small memory cleanup operations
- Matches allocation patterns with high iteration count

#### `copy_mem` (10 iterations)

**File**: `bench/memory.rs`

Tests memory copying performance:

- Copies large blocks of memory using `CopyMem()`
- Measures bulk data movement efficiency
- Low iterations due to large memory operations

#### `set_mem` (10 iterations)

**File**: `bench/memory.rs`

Tests memory initialization performance:

- Sets large blocks of memory using `SetMem()`
- Measures bulk memory initialization efficiency
- Important for zeroing or pattern-filling operations

#### `get_memory_map` (10 iterations)

**File**: `bench/memory.rs`

Tests system memory map retrieval:

- Calls `GetMemoryMap()` to retrieve current memory layout
- Measures system introspection performance
- Critical for OS loaders and memory managers

### 5. Miscellaneous Services

#### `calculate_crc32` (100 iterations)

**File**: `bench/misc.rs`

Tests checksum calculation performance:

- Calculates CRC32 over 128 bytes of data using `CalculateCrc32()`
- Measures cryptographic/integrity checking performance
- Important for data validation in firmware

#### `install_configuration_table` (10 iterations)

**File**: `bench/misc.rs`

Tests configuration table installation:

- Installs a configuration table entry using test GUID
- Measures system configuration management performance  
- Low iterations due to global system impact

### 6. Protocol Services

#### `install_protocol_interface` (100 iterations)

**File**: `bench/protocol.rs`

Tests protocol installation performance:

- Installs a protocol interface with test GUID and dummy data
- Measures protocol registration in the UEFI database
- Uninstalls after measurement for cleanup
- Critical for driver and service registration

#### `open_protocol` (10000 iterations)

**File**: `bench/protocol.rs`

Tests protocol access performance:

- Opens installed protocols using `OpenProtocol()`
- Measures protocol lookup and access performance
- High iteration count reflects frequent usage
- Tests the core service discovery mechanism

#### `handle_protocol` (10000 iterations)

**File**: `bench/protocol.rs`

Tests legacy protocol access:

- Uses `HandleProtocol()` for direct protocol access
- Measures simplified protocol retrieval
- High iteration count for frequent operations
- Tests backward compatibility interfaces

#### `close_protocol` (100 iterations)

**File**: `bench/protocol.rs`

Tests protocol cleanup performance:

- Closes previously opened protocols using `CloseProtocol()`
- Measures protocol resource management
- Important for proper resource lifecycle management

#### `locate_device_path` (100 iterations)

**File**: `bench/protocol.rs`

Tests device path resolution:

- Locates devices by device path using `LocateDevicePath()`
- Measures device discovery performance
- Critical for device enumeration and access

#### `open_protocol_information` (100 iterations)

**File**: `bench/protocol.rs`

Tests protocol metadata retrieval:

- Gets protocol information using `OpenProtocolInformation()`
- Measures protocol introspection performance
- Important for system debugging and management

#### `protocols_per_handle` (100 iterations)

**File**: `bench/protocol.rs`

Tests handle protocol enumeration:

- Retrieves all protocols on a handle using `ProtocolsPerHandle()`
- Measures handle introspection performance
- Important for device capability discovery

#### `register_protocol_notify` (10 iterations)

**File**: `bench/protocol.rs`

Tests protocol notification registration:

- Registers for protocol installation notifications
- Measures event-driven protocol discovery setup
- Low iterations due to system-wide impact

#### `reinstall_protocol_interface` (100 iterations)

**File**: `bench/protocol.rs`

Tests protocol update performance:

- Reinstalls an existing protocol with new data
- Measures protocol upgrade/modification performance
- Important for dynamic protocol updates

#### `uninstall_protocol_interface` (10 iterations)

**File**: `bench/protocol.rs`

Tests protocol removal performance:

- Uninstalls previously installed protocols
- Measures protocol cleanup and system database updates
- Low iterations due to system impact

### 7. Task Priority Level (TPL) Services

#### `raise_tpl` (1000000 iterations)

**File**: `bench/tpl.rs`

Tests interrupt disable performance:

- Raises TPL to high level (31) using `RaiseTpl()`
- Measures interrupt masking performance
- Very high iteration count due to extremely lightweight operation
- Critical for real-time and interrupt handling performance

#### `restore_tpl` (1000000 iterations)

**File**: `bench/tpl.rs`

Tests interrupt restore performance:

- Restores previous TPL using `RestoreTpl()`
- Rotates through different TPL levels for comprehensive testing
- Measures interrupt unmasking and pending interrupt delivery
- Critical for system responsiveness and interrupt latency

## Performance Characteristics

The benchmarks measure cycle counts using CPU performance counters, providing:

- **Total Cycles**: Raw CPU cycles consumed
- **Cycles/Operation**: Average cycles per function call  
- **Total Time**: Wall-clock time in milliseconds
- **Statistical Data**: Min, max, and standard deviation
- **Call Count**: Number of iterations for statistical significance

## Output Format

Results are displayed as a markdown table in the UEFI shell (one sample row shown below):

```
| Name               | Total cycles | Total calls | Cycles/op | Total time (ms) | Min cycles | Max cycles | SD [cycles] |
| ------------------ | ------------ | ----------- | --------- | --------------- | ---------- | ---------- | ----------- |
| connect_controller | 1234567      | 100         | 12345.67  | 45.67           | 10000      | 15000      | 1500        |
```
