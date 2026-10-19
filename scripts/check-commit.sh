#!/bin/sh
# The commit rules of docs/git-workflow.md: the subject starts with a type (task 056), no Claude-Session trailer
# and no Claude Code session link in a commit message or a pull request text, and the maintainer commits with the
# GitHub noreply address (D-010). Called by .githooks/commit-msg and by the commit-rules job in
# .github/workflows/commit-rules.yml, which also runs the self-test.
#
#   check-commit.sh MESSAGE_FILE [NAME EMAIL]...
#   check-commit.sh --self-test
#
# Every NAME EMAIL pair is one identity of the commit (author, committer). Exits 1 with a reason per broken rule.
# For a pull request text the first line is the title, so the title is held to the subject rule.
set -eu

# The types of docs/git-workflow.md, "Titles": `type: text` or `type(scope): text`, lower case.
types='feat|fix|docs|ci|chore|tasks|deps'

# The maintainer's names: the git config one, and the GitHub login a merge on the web writes.
is_maintainer() { [ "$1" = 'Szőke Péter' ] || [ "$1" = 'speti4' ]; }
maintainer_email='20753030+speti4@users.noreply.github.com'

# Good and bad messages through this script; exits 1 when one gets the wrong verdict.
self_test() {
  dir=$(mktemp -d)
  trap 'rm -rf "$dir"' EXIT
  cases=0
  failures=0
  # check pass|fail MESSAGE [NAME EMAIL]...
  check() {
    expected=$1
    printf '%s\n' "$2" > "$dir/message.txt"
    shift 2
    cases=$((cases + 1))
    if sh "$0" "$dir/message.txt" "$@" 2> "$dir/stderr.txt"; then actual=pass; else actual=fail; fi
    if [ "$actual" != "$expected" ]; then
      failures=$((failures + 1))
      printf 'self-test: expected %s, got %s: %s\n' "$expected" "$actual" "$(head -n 1 "$dir/message.txt")" >&2
      sed 's/^/  /' "$dir/stderr.txt" >&2
    fi
  }
  trailer='Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>'

  # The subject rule.
  for subject in 'feat: open a second mbox' 'fix: keep the label tree on a multibyte boundary' \
    'docs: describe the search syntax' 'ci: re-run commit-rules on an edited pull request' \
    'chore: polish the code comments' 'tasks: close 056' 'deps: bump serde from 1.0.1 to 1.0.2' \
    'fix(core): skip a broken header' 'feat: ship the usage guide as HTML (task 048) (#201)' \
    "Merge branch 'main' into task/056-commit-hook-and-ci-checks" 'Merge pull request #12 from someone/a-branch' \
    'Revert "feat: open a second mbox"'; do
    check pass "$subject

$trailer"
  done
  for subject in 'open a second mbox' 'Feat: open a second mbox' 'FIX: a bug' 'feature: open a second mbox' \
    'feat:open a second mbox' 'feat : open a second mbox' 'feat():  open a second mbox' 'feat(): open a second mbox' \
    'feat:' 'feat: ' 'docs, tasks: two types' 'fixup! feat: open a second mbox' 'Revert feat: open a second mbox' \
    'Merged the branch' 'WIP'; do
    check fail "$subject

$trailer"
  done
  check pass "

feat: a subject after blank lines"
  check pass "# a comment line git strips
feat: a subject below a comment"
  check fail ''

  # The D-010 rules: trailer, link, author address.
  check fail "feat: a subject

Claude-Session: https://example.invalid/session"
  check fail "feat: a subject

See https://claude.ai/code/session_0123456789abcdef"
  check pass "feat: a subject" 'Szőke Péter' "$maintainer_email" 'Szőke Péter' "$maintainer_email"
  check fail "feat: a subject" 'Szőke Péter' 'someone@example.com' 'Szőke Péter' "$maintainer_email"
  check pass "feat: a subject" 'A Contributor' 'contributor@example.com' 'GitHub' 'noreply@github.com'

  if [ "$failures" -ne 0 ]; then
    echo "self-test: $failures of $cases cases failed" >&2
    exit 1
  fi
  echo "self-test: $cases cases passed"
  exit 0
}

if [ "${1-}" = --self-test ]; then
  self_test
fi

message_file=$1
shift
status=0

# Comment lines and everything below the scissors line of `git commit -v` (the diff) are not the message.
message=$(sed -e '/^# -* >8 -*$/,$d' -e '/^#/d' "$message_file")

# The subject is the first line that is not blank, as git's cleanup keeps it. A merge or a revert subject passes
# as git writes it.
subject=$(printf '%s\n' "$message" | sed -n '/[^[:space:]]/{p;q;}')
case $subject in
  'Merge '* | 'Revert "'*) ;;
  *)
    if ! printf '%s\n' "$subject" | grep -Eq "^($types)(\([^()[:space:]]+\))?: [^[:space:]]"; then
      echo "commit rules: the subject starts with a type and a colon, \`feat: …\` or \`feat(scope): …\`, one of" \
        "$(echo "$types" | sed 's/|/ /g') (docs/git-workflow.md): ${subject:-(empty)}" >&2
      status=1
    fi
    ;;
esac

if printf '%s\n' "$message" | grep -qi '^claude-session:'; then
  echo "commit rules: no Claude-Session: trailer (D-010 point 3); Co-Authored-By is enough" >&2
  status=1
fi
if printf '%s\n' "$message" | grep -q 'claude\.ai/code/session_'; then
  echo "commit rules: no Claude Code session link in a commit or pull request text (D-010 point 3)" >&2
  status=1
fi

reported=
while [ $# -ge 2 ]; do
  if is_maintainer "$1" && [ "$2" != "$maintainer_email" ] && [ "$2" != "$reported" ]; then
    echo "commit rules: $1 commits as $maintainer_email, not $2 (D-010)" >&2
    reported=$2
    status=1
  fi
  shift 2
done

exit $status
