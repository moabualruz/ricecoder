# Server Mode Notes (Local-First)

## Current Decision

- Server mode is disabled by default and gated behind a build feature (`server`).
- Local indexing and search remain the default for all CLI flows.
- Remote calls only occur when `--server` (or `--server` in MCP) is explicitly supplied
  and the binary is built with the server feature.

## Open Questions

- Multi-user usage: how will multiple clients share a server while preserving local
  indexing for low-latency workflows?
- Watch/index behavior: when a server is available, should watch/index operations
  always update the server, or only when explicitly requested?
- Data scope: should server mode index only project-local code, or also allow
  broader datasets (e.g., language docs) with explicit opt-in?
- CI/CD usage: what is the intended flow for pinned checks and reproducible
  server indexes in automation?

## Potential Server Use Cases

- Shared indexing for multiple tools/clients.
- CI/CD validation against a fixed, reproducible index snapshot.
- Centralized telemetry and admin surface for operational visibility.
- Remote search for larger datasets when local storage is constrained.

## Follow-up Ideas

- Add a formal server mode decision log once workflows are defined.
- Document how local and server indices co-exist without conflicting behaviors.
- Define a clear opt-in flag for any server synchronization behavior.
