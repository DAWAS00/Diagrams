# Project Rust skills

Installed locally for Codex on 2026-11-05 using the Skills CLI with `--agent codex --copy -y`. No global skill installation is required for this repository.

| Skill | Source | Use |
|---|---|---|
| [rust-best-practices](.agents/skills/rust-best-practices/SKILL.md) | [Apollo GraphQL](https://skills.sh/apollographql/skills/rust-best-practices) | Ownership, typed errors, API design, linting and review |
| [rust-testing](.agents/skills/rust-testing/SKILL.md) | [ECC](https://skills.sh/affaan-m/ecc/rust-testing) | Behavior tests, integration tests, property testing and benchmarks |

The sources and skill content were reviewed before installation. At discovery, the CLI reported 22.3K installs for Apollo and 13.2K for ECC; directory pages reported 22.2K and 6.9K respectively. GitHub API repository star counts were 125 and 297,169. Popularity is context rather than a guarantee of correctness.

## Project application

Use these references with [our engineering rules](plan-rust-diagram-editor/RUST-ENGINEERING.md). Repository requirements and verified compiler behavior take precedence over illustrative skill snippets. In particular, do not add Tokio, mocks, coverage gates or testing dependencies merely because a skill demonstrates them. Keep the safe-Rust and no-production-panic policy, and test meaningful behavior at the task's boundaries.

Only these two skill directories are allowed through `.gitignore`. Existing REA skills and machine-specific `.codex` configuration remain local. `skills-lock.json` records the CLI source paths and content hashes. This does not change the Cargo dependencies or completion status of application tasks.

## Install or refresh

Run from the repository root:

```sh
npx skills add apollographql/skills --skill rust-best-practices --agent codex --copy -y
npx skills add affaan-m/ecc --skill rust-testing --agent codex --copy -y
npx skills list --agent codex
```

Review skill and lockfile diffs after updates. Upstream license notices are preserved in each skill directory.
