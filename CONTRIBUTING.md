Contributing
======================

Pull requests (PRs)
-------------------

- Create draft pull requests and change the status to ready when the PR is ready.
- Use a [feature branch][git-feature-branch] instead of the master branch.

### Commit messages

Follow the [conventional commits guidelines][conventional_commits] to make the git logs
more valuable. The general structure is:
```
[optional type([optional scope]): ]<short-description>

[optional body]

[optional footer(s)]
```
- The *description* shouldn't start with a capital letter or end in a period.
- Use the **imperative voice** for the *description*: "Fix bug" rather than "Fixed bug"
  or "Fixes bug."
- Try to keep the *description* under **50** characters and the following lines under
  **72** characters.
- A blank line must follow the *description*.


[conventional_commits]: https://www.conventionalcommits.org
[git-feature-branch]: https://www.atlassian.com/git/tutorials/comparing-workflows
[pauli_tracker package]: https://github.com/taeruh/pauli_tracker/blob/main/python_lib
