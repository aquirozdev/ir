# First experiment protocol

Status: design only. No tasks have been scored and no model has been selected or frozen.

## Hypothesis and confounds

A typed declarative representation may reduce the inference needed for business application construction. A deterministic runtime also supplies substantial prewritten functionality. Measure both effects; do not attribute all gains to representation or model intelligence.

## Baselines

| Arm | Model | Output and execution |
| --- | --- | --- |
| A | Same small checkpoint | Conventional source with standard libraries |
| B | Same small checkpoint | SIR on the fixed runtime |
| C | Same small checkpoint | Source using an equivalent high-level helper library |
| D | Larger open checkpoint | Source; after the first small-model comparison |
| E | External coding system | Source; separate hardware/API category, later |

Include unconstrained and grammar-constrained SIR ablations and repair/no-repair ablations. Freeze prompts and adapters before holdout access. Give all arms the same specification, permitted public feedback and a documented total solving budget. Report additional matched-token and matched-time comparisons when feasible; do not silently truncate source at a SIR-sized token budget.

## Neutral behavioral contract

All submissions expose the same command/query interface and are tested for externally observable behavior. Evaluators must not require SIR internals, exact AST matches or runtime-generated tests. The runtime cannot be the only oracle: derive adversarial expectations independently from specifications. Use mutation testing to check that evaluators reject deliberately wrong implementations.

Select tasks before seeing model outcomes. Humans must demonstrate representation coverage, and excluded requirements must be reported. Separate representable tasks from an externally sampled application set to reveal scope selection bias.

## Data separation

Begin with hand-authored public development tasks. Create holdouts with disjoint semantic compositions and independent paraphrases, not just renamed entities. Freeze runtime, prompts, model and task-generation seeds before evaluation. If test feedback guides development, retire that test split and create a fresh one.

Training verification tests and final evaluation tests are distinct. Rejection sampling and RL may use training verifiers; they must never use final holdout tests or their rewards. Synthetic labels are not automatically perfect: specification ambiguity, generator defects and interpreter defects need auditing.

Keep private tests outside the public repository and outside the submission process's readable filesystem. `no network` does not prevent reading nearby private tests. Use process/container isolation and a narrow protocol; malicious source baselines require sandboxing. Do not expose holdout tests through public Actions logs or artifacts. Evaluators should be controlled by an independent runner for serious submissions.

## Measurements

Record all-or-nothing task pass rate, test pass rate, timeout/crash rates, paired task outcomes, confidence intervals, total wall and CPU seconds, peak memory including child processes, model bytes, input/output tokens, repairs and budget. Pre-register a statistical comparison appropriate to paired binary outcomes. Energy is optional and reported only with a defined measurement method. No provider energy estimates from API latency.

Record model SHA256, license, tokenizer/template, quantization, inference engine commit/build, runtime commit, dataset version, OS/container digest, CPU model, CPU quota/affinity, RAM limit, context size, seed and decoding settings. Setup/download time and solve time must be reported separately. A hosted runner label is not identical hardware across runs; measure variability and use enforced quotas.

## Gates

1. **Coverage:** humans encode the selected development task set; failure means revise scope or IR.
2. **Runtime:** independent tests cover transactions, policies and invariants; failures block model benchmarking.
3. **Pilot:** same-model A/B/C comparison, no fine-tuning, fixed budget. Investigate whether any gain exceeds uncertainty and survives the helper-library baseline.
4. **Confirmatory:** pre-register a fresh holdout and minimum meaningful effect before accessing it. Do not set the threshold after seeing the pilot result.
5. **Scale:** train only after the pilot supports the approach; evaluate tiny models and independent holdouts.

A 70% task pass rate and 2× relative improvement are provisional aspirations, not acceptance evidence. Frontier comparisons are meaningful only for the explicitly evaluated domain. Report total system/runtime size as well as neural model size. Source-equivalent expansion is descriptive, not a correctness metric, and needs a fixed independent reference implementation.
