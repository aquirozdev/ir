# Research and integration checklist

These are starting points for investigation, not an audited prior-art survey or endorsed performance claims. Do not copy model sizes, runner limits or latency figures into project claims without verifying the exact current artifact and environment.

| Topic | Primary reference | Before integration |
| --- | --- | --- |
| JSON contracts | [JSON Schema](https://json-schema.org/draft/2020-12) | Test parser/schema alignment and unsupported constructs |
| CPU inference | [llama.cpp](https://github.com/ggml-org/llama.cpp) | Pin engine commit, architecture support and decoding backend |
| Authorization | [Cedar documentation](https://docs.cedarpolicy.com/) | Verify schema, policy validation, default-deny behavior and transaction mapping |
| Transactions | [SQLite isolation](https://www.sqlite.org/isolation.html) | Test locking, concurrent commands and rollback |
| CI constraints | [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners) | Verify account-specific limits; enforce and record quotas |
| Candidate weights | [Qwen organization](https://huggingface.co/Qwen) | Select exact checkpoint, license, GGUF provenance and hash |

Prior-art leads from brainstorming include AIRL, AINL and Spec. Their existence, feature claims, dates and experimental comparisons still require a primary-source audit before citing them as established research results. The intended contribution is an empirical result about representation, runtime assistance and model scale; novelty is not established by naming a new IR.
