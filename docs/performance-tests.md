# Performance and component checks — v0.6.0

## CPU single-core benchmark

- Runs a fixed prime-candidate calculation loop on one worker for 3–15 seconds.
- Reports candidate/primality throughput, not a standardized benchmark score.
- It is not a thermal stress test or a guarantee of CPU stability.

## CPU multi-core benchmark

- Runs the same bounded calculation workload across up to eight threads.
- Default UI duration is five seconds; native limit is 3–10 seconds.
- Does not exercise every CPU instruction, AVX unit, cache, integrated GPU or thermal limit.
- User confirmation is required and the UI advises saving work first.

## RAM smoke test

- Allocates 32–512 MiB, subject to a quarter of reported available memory.
- Writes and verifies six deterministic patterns: address-derived values, complement, rotated/XOR values, alternating checkerboard words, all-zero and all-one patterns.
- The application initializes the allocated buffer before indexed access and reports a mismatch index when read-back differs.
- Does not test all physical addresses, all modules/slots, ECC, memory timing, firmware memory training or memory outside the application-owned allocation.
- A pass means only that the process-level read-back test matched. It is not a full memory stability certification.
- A full memory diagnostic requires a bootable test utility such as Memtest86+ or an OEM pre-boot memory diagnostic.

## GPU telemetry

- NVIDIA: `nvidia-smi` can provide reported utilization, memory, temperature, graphics clock and power values.
- AMD on supported Linux systems: AMD SMI CLI JSON metrics can provide supported utilization, temperature, power, clock and memory metrics. Field availability depends on GPU family, driver and AMD SMI version.
- Intel on supported Linux systems: `intel_gpu_top -J -s 1000 -n 1` can provide engine busy counters. The UI labels this as engine activity; it is not necessarily a single whole-GPU utilization metric. PMU permissions and driver support may limit access.
- On other OS/vendor combinations, CrossDiag continues to show inventory but leaves unavailable telemetry fields unknown.
- Telemetry is passive sampling. CrossDiag does not apply a GPU load or run a GPU stress suite.

## Guidance

Save work before running benchmarks, close unrelated workloads, connect laptop power where appropriate, and monitor thermal telemetry. Stop testing if the computer becomes unstable, unexpectedly hot, or begins throttling severely. Results depend on power policy, cooling, firmware and background workload.
