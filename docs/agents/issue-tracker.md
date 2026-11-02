# Issue tracker: GitHub

Issues and specs for this repository live in GitHub Issues: `patricksmill/sml2-rs`. Use the `gh` CLI for issue operations.

## Conventions

- Create an issue with `gh issue create --title "..." --body "..."`.
- Read an issue with `gh issue view <number> --json number,title,body,labels,comments`.
- List issues with `gh issue list --state open --json number,title,body,labels,comments` and add appropriate state or label filters.
- Comment with `gh issue comment <number> --body "..."`.
- Apply or remove labels with `gh issue edit <number> --add-label "..."` or `--remove-label "..."`.
- Close with `gh issue close <number>`; post any explanation first.

Infer the repository from the local GitHub remote. File issues only into this repository.

## Pull requests as a triage surface

**PRs as a request surface: no.** External PRs are not part of routine issue triage. Explicitly named PRs may still be triaged.
