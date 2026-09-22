# Security policy

Report vulnerabilities privately to the maintainers rather than opening a
public issue. This repository is an early runtime scaffold, not a hardened
multi-tenant sandbox.

WASM mods run through Wasmtime without WASI. Filesystem, network, shell,
environment, process, and secret access are therefore absent; simulation host
calls are deny-by-default capabilities. Fuel and linear-memory limits reduce
resource exhaustion risk. Traps and denials become structured errors.

World packages are uncompressed deterministic tar archives. Inspection rejects
malformed archives, but package signatures and full archive-bomb policy are not
implemented. Do not install untrusted packages into privileged paths. Local
world traversal is constrained by directory enumeration and package entries are
never extracted by the current code. Replay integrity is protected by BLAKE3
artifact and event-chain hashes, not by an author signature; an attacker able to
replace a replay can also recompute unsigned hashes. Deterministic denial of
service remains possible within configured CPU/fuel and memory bounds.

Never expose new host functions to mods without a capability check, bounded
inputs, deterministic behavior, and tests for denial.
