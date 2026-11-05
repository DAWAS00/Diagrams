---
tags: [skill-creator]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Write, Edit, Bash]
---

shell.rs in the app is way too big. start splitting it: move the status bar out into its own module. dont change how anything behaves
