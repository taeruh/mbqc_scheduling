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

Updating python_lib with upstream
---------------------------------

Because of [#1444] we have to extend the original [pauli_tracker package] instead of
using the underlining Rust crate as a dependency. We do not use `git submodules` for
that because we want to track the extension in this repository here (kinda unnecessary
to create an additional GitHub repo for that); this can be done with [subtree merging]:\
Initial step:
```
git remote add upstream/pauli_tracker https://github.com/taeruh/pauli_tracker.git
git fetch upstream/pauli_tracker --no-tags
mkdir pauli_tracker
git read-tree --prefix=pauli_tracker/ -u upstream/pauli_tracker/main
```
Now the whole `pauli_tracker` repo is in the `pauli_tracker` directory, however, we are
only interested in the `python_lib` subdirectory remove all files and directories in
`pauli_tracker`, except of `python_lib`
```
rm -rf <...> (cf. makefile in pauli_tracker)
```
Commit:
```
git add --all; git commit -m "Add pauli_tracker"
```
We pull in updates from upstream via
```
git fetch upstream/pauli_tracker --no-tags
git merge -Xsubtree=pauli_tracker upstream/pauli_tracker/main --no-commit
# fix conflicts and remove all the unneeded stuff again (cf. makefile in pauli_tracker)
git commit
```
Note that first merge requires `--allow-unrelated-histories`.\
Doing it this way keeps the git history clean and we track everything in one repo. What
is a little bit annoying is that instead of having a directory `python_lib` at the top
level, we have `pauli_tracker/python_lib`. A way around this would be to just to a
normal merge, however, this would lead to many merge conflicts, since there would be
many unrelated files with the same name. This could be fixed by first checking out a
separate `pauli_tracker` branches, merging `upstream/pauli_tracker/main` into this
branch, remove the unneeded stuff in the `pauli_tracker` branch, merge `pauli_tracker`
into `main`. However, this would lead to a not-so-clean git history (maybe one could
clean it up with some squashing and rebasing, but I do not know how to easily do that).
Therefore, we stick with the [subtree merging] strategy, since this also describes best
what we are actually doing, i.e., pulling in a repo as submodule but tracking its
contents, and just create a soft-link to `pauli_tracker/python_lib` at the top level.

[conventional_commits]: https://www.conventionalcommits.org
[git-feature-branch]: https://www.atlassian.com/git/tutorials/comparing-workflows
[pauli_tracker package]: https://github.com/taeruh/pauli_tracker/blob/main/python_lib
[subtree merging]: https://git-scm.com/book/en/v2/Git-Tools-Advanced-Merging
[#1444]: https://github.com/PyO3/pyo3/issues/1444
