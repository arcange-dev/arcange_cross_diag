# GPU telemetry providers — v0.6.0

CrossDiag separates GPU inventory from live telemetry. A GPU can be inventoried even when no compatible telemetry provider is installed or permissions prevent sampling.

## NVIDIA

- Provider: `nvidia-smi`.
- Supported fields depend on GPU/driver: utilization, memory utilization, total/used memory, GPU temperature, graphics clock and power draw.
- Windows and Linux support is attempted when `nvidia-smi` is discoverable; Windows also checks the standard NVIDIA NVSMI installation path.

## AMD (Linux)

- Provider: AMD SMI CLI (`amd-smi`).
- CrossDiag invokes the read-only metrics command `amd-smi metric --temperature --usage --power --clock --mem-usage --json`.
- The parser only maps recognized fields. An absent, unsupported or `N/A` metric remains `null`; it is not displayed as zero.
- Install AMD SMI using the official ROCm/AMD instructions for the exact supported platform and GPU. CrossDiag does not bundle AMD SMI.
- Official documentation: https://rocm.docs.amd.com/projects/amdsmi/en/develop/how-to/amdsmi-cli-tool.html

## Intel (Linux)

- Provider: `intel_gpu_top` from the `intel-gpu-tools` package.
- CrossDiag invokes one JSON sample with `intel_gpu_top -J -s 1000 -n 1`.
- The parser reports the maximum numeric engine `busy` counter in the sample as engine activity. This is not a benchmark, a device-wide average or a physical health score.
- Permission to read GPU performance monitoring counters may be restricted by the kernel/system configuration.
- Intel's `intel_gpu_top` tool does not cover every Intel GPU generation/driver combination; use the tool's own compatibility notes where a device is unsupported.
- Manual: https://manpages.debian.org/testing/intel-gpu-tools/intel_gpu_top.1.en.html

## Interpretation

- Telemetry is a point-in-time observation. It does not test VRAM integrity, shader correctness, PCIe signal quality, power delivery or thermal stability under load.
- GPU stress tests are not part of the passive telemetry scan. A separate, user-confirmed test design and hardware-specific safe limits would be required before introducing load testing.
- Do not compare raw values across different vendors as though they were standardized benchmark scores.
