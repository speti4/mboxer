#!/bin/sh
# The commit rules of docs/git-workflow.md (D-09, D-10): no Claude-Session trailer and no Claude Code session
# link in a commit message or a pull request text, and the maintainer commits with the GitHub noreply address.
# Called by .githooks/commit-msg and by the commit-rules job in .github/workflows/ci.yml.
#
#   check-commit.sh MESSAGE_FILE [NAME EMAIL]...
#
# Every NAME EMAIL pair is one identity of the commit (author, committer). Exits 1 with a reason per broken rule.
set -eu

maintainer_name='Szőke Péter'
maintainer_email='20753030+speti4@users.noreply.github.com'

message_file=$1
shift
status=0

# Comment lines and everything below the scissors line of `git commit -v` (the diff) are not the message.
message=$(sed -e '/^# -* >8 -*$/,$d' -e '/^#/d' "$message_file")

if printf '%s\n' "$message" | grep -qi '^claude-session:'; then
  echo "commit rules: no Claude-Session: trailer (D-09); Co-Authored-By is enough" >&2
  status=1
fi
if printf '%s\n' "$message" | grep -q 'claude\.ai/code/session_'; then
  echo "commit rules: no Claude Code session link in a commit or pull request text (D-09)" >&2
  status=1
fi

reported=
while [ $# -ge 2 ]; do
  if [ "$1" = "$maintainer_name" ] && [ "$2" != "$maintainer_email" ] && [ "$2" != "$reported" ]; then
    echo "commit rules: $1 commits as $maintainer_email, not $2 (D-10)" >&2
    reported=$2
    status=1
  fi
  shift 2
done

exit $status
