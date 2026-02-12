#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/ship_alpha.sh [options]

This script automates the full alpha shipment flow:
1) local validation (fmt/test/clippy)
2) feature commit/push/PR/checks/merge
3) release branch (version bump + changelog cut)
4) release PR/checks/merge
5) GitHub prerelease publish

Options:
  --feature-branch <name>         Branch to use for feature PR.
                                   If omitted, current branch is used.
  --feature-commit <message>      Commit message for feature changes.
                                   Required only when there are local staged changes.
  --feature-pr-title <title>      Feature PR title (defaults to feature commit message).
  --feature-pr-body-file <path>   Feature PR body file.
  --release-pr-body-file <path>   Release PR body file.
  --release-version <version>     Release version, e.g. 0.0.34-alpha.0.
                                   If omitted, patch is auto-incremented.
  --release-date <YYYY-MM-DD>     Release date for CHANGELOG heading (default: today).
  --base-branch <name>            Base branch for PRs (default: main).
  --sleep-seconds <n>             Initial wait before checking PR checks (default: 120).
  --check-interval <n>            gh checks poll interval in seconds (default: 20).
  --jobs <n>                      Cargo -j jobs for test/clippy (default: 1).
  --gh-retries <n>                Retries for transient gh API connectivity errors (default: 8).
  --gh-retry-delay <n>            Delay between gh retries in seconds (default: 30).
  --skip-local-checks             Skip local cargo fmt/test/clippy.
  -h, --help                      Show this help.

Examples:
  scripts/ship_alpha.sh \
    --feature-branch feat/r022-something \
    --feature-commit "feat(render): add R-022 something" \
    --feature-pr-title "feat(render): add R-022 something"

  scripts/ship_alpha.sh \
    --feature-branch feat/r022-something \
    --feature-commit "feat(render): add R-022 something" \
    --release-version 0.0.34-alpha.0
USAGE
}

log() {
  printf '[ship-alpha] %s\n' "$*"
}

die() {
  printf '[ship-alpha] ERROR: %s\n' "$*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "Missing required command: $1"
}

run_gh_with_retries() {
  local attempts="$1"
  local delay="$2"
  shift 2

  local try=1
  local out
  local status
  while true; do
    set +e
    out="$("$@" 2>&1)"
    status=$?
    set -e

    if [[ "$status" -eq 0 ]]; then
      printf '%s\n' "$out"
      return 0
    fi

    if ! printf '%s' "$out" | rg -q "error connecting to api.github.com"; then
      printf '%s\n' "$out" >&2
      return "$status"
    fi

    if [[ "$try" -ge "$attempts" ]]; then
      printf '%s\n' "$out" >&2
      return "$status"
    fi

    log "Transient gh API error. Retry ${try}/${attempts} in ${delay}s"
    sleep "$delay"
    try=$((try + 1))
  done
}

extract_pr_url() {
  printf '%s\n' "$1" | rg -o 'https://github.com/[^[:space:]]+/pull/[0-9]+' | tail -n 1
}

extract_release_url() {
  printf '%s\n' "$1" | rg -o 'https://github.com/[^[:space:]]+/releases/tag/[^[:space:]]+' | tail -n 1
}

compute_next_alpha_version() {
  local current
  current="$(sed -n -E 's/^version = "([^"]+)"/\1/p' Cargo.toml | head -n 1)"
  [[ -n "$current" ]] || die "Could not read current version from Cargo.toml"

  if [[ "$current" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)-alpha\.([0-9]+)$ ]]; then
    local major="${BASH_REMATCH[1]}"
    local minor="${BASH_REMATCH[2]}"
    local patch="${BASH_REMATCH[3]}"
    printf '%s.%s.%s-alpha.0\n' "$major" "$minor" "$((patch + 1))"
    return 0
  fi

  die "Current version '$current' is not in expected alpha format X.Y.Z-alpha.N"
}

move_unreleased_added_to_release() {
  local changelog_path="$1"
  local release_version="$2"
  local release_date="$3"
  local notes_out="$4"

  VERSION="$release_version" RELEASE_DATE="$release_date" NOTES_OUT="$notes_out" CHANGELOG_PATH="$changelog_path" perl <<'PERL'
use strict;
use warnings;

my $path = $ENV{CHANGELOG_PATH};
my $version = $ENV{VERSION};
my $date = $ENV{RELEASE_DATE};
my $notes_out = $ENV{NOTES_OUT};

open my $in, '<', $path or die "Could not open $path for read: $!\n";
local $/;
my $text = <$in>;
close $in;

$text =~ /## \[Unreleased\]\n\n### Added\n((?:- .*\n)+)\n/s
  or die "Could not parse Unreleased/Added section in CHANGELOG.md\n";
my $bullets = $1;

$bullets !~ /\A- No changes yet\.\n\z/s
  or die "Unreleased section has no releasable entries\n";

my $replacement =
  "## [Unreleased]\n\n### Added\n- No changes yet.\n\n" .
  "## [$version] - $date\n\n### Added\n" .
  $bullets .
  "\n";

$text =~ s/## \[Unreleased\]\n\n### Added\n(?:- .*\n)+\n/$replacement/s
  or die "Failed to update CHANGELOG.md\n";

open my $out, '>', $path or die "Could not open $path for write: $!\n";
print {$out} $text;
close $out;

open my $notes, '>', $notes_out or die "Could not open $notes_out for write: $!\n";
print {$notes} "## Added\n";
print {$notes} $bullets;
close $notes;
PERL
}

SLEEP_SECONDS=120
CHECK_INTERVAL=20
JOBS=1
GH_RETRIES=8
GH_RETRY_DELAY=30
BASE_BRANCH="main"
FEATURE_BRANCH=""
FEATURE_COMMIT_MSG=""
FEATURE_PR_TITLE=""
FEATURE_PR_BODY_FILE=""
RELEASE_PR_BODY_FILE=""
RELEASE_VERSION=""
RELEASE_DATE="$(date +%Y-%m-%d)"
SKIP_LOCAL_CHECKS=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --feature-branch)
      FEATURE_BRANCH="$2"
      shift 2
      ;;
    --feature-commit)
      FEATURE_COMMIT_MSG="$2"
      shift 2
      ;;
    --feature-pr-title)
      FEATURE_PR_TITLE="$2"
      shift 2
      ;;
    --feature-pr-body-file)
      FEATURE_PR_BODY_FILE="$2"
      shift 2
      ;;
    --release-pr-body-file)
      RELEASE_PR_BODY_FILE="$2"
      shift 2
      ;;
    --release-version)
      RELEASE_VERSION="$2"
      shift 2
      ;;
    --release-date)
      RELEASE_DATE="$2"
      shift 2
      ;;
    --base-branch)
      BASE_BRANCH="$2"
      shift 2
      ;;
    --sleep-seconds)
      SLEEP_SECONDS="$2"
      shift 2
      ;;
    --check-interval)
      CHECK_INTERVAL="$2"
      shift 2
      ;;
    --jobs)
      JOBS="$2"
      shift 2
      ;;
    --gh-retries)
      GH_RETRIES="$2"
      shift 2
      ;;
    --gh-retry-delay)
      GH_RETRY_DELAY="$2"
      shift 2
      ;;
    --skip-local-checks)
      SKIP_LOCAL_CHECKS=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "Unknown argument: $1"
      ;;
  esac
done

require_cmd git
require_cmd gh
require_cmd cargo
require_cmd rg
require_cmd perl
require_cmd sed

git rev-parse --is-inside-work-tree >/dev/null 2>&1 || die "Run this from inside the repository"
gh auth status >/dev/null 2>&1 || die "gh is not authenticated. Run: gh auth login"

if [[ -n "$FEATURE_PR_BODY_FILE" && ! -f "$FEATURE_PR_BODY_FILE" ]]; then
  die "Feature PR body file not found: $FEATURE_PR_BODY_FILE"
fi
if [[ -n "$RELEASE_PR_BODY_FILE" && ! -f "$RELEASE_PR_BODY_FILE" ]]; then
  die "Release PR body file not found: $RELEASE_PR_BODY_FILE"
fi

cleanup_files=()
cleanup() {
  local f
  for f in "${cleanup_files[@]}"; do
    [[ -n "$f" ]] && rm -f "$f"
  done
}
trap cleanup EXIT

current_branch="$(git branch --show-current)"
if [[ -n "$FEATURE_BRANCH" && "$FEATURE_BRANCH" != "$current_branch" ]]; then
  if git show-ref --verify --quiet "refs/heads/$FEATURE_BRANCH"; then
    git checkout "$FEATURE_BRANCH"
  else
    git checkout -b "$FEATURE_BRANCH"
  fi
fi

current_branch="$(git branch --show-current)"
if [[ "$current_branch" == "$BASE_BRANCH" ]]; then
  die "Current branch is '$BASE_BRANCH'. Use --feature-branch to create/use a feature branch."
fi
FEATURE_BRANCH="$current_branch"

if [[ "$SKIP_LOCAL_CHECKS" -eq 0 ]]; then
  log "Running local checks"
  cargo fmt --all
  cargo test --all-features -j "$JOBS"
  cargo clippy --all-targets --all-features -j "$JOBS" -- -D warnings
else
  log "Skipping local checks (--skip-local-checks)"
fi

log "Committing feature changes (if any)"
git add -A
if git diff --cached --quiet; then
  log "No staged changes found; skipping feature commit"
else
  [[ -n "$FEATURE_COMMIT_MSG" ]] || die "--feature-commit is required when there are local changes to commit"
  git commit -m "$FEATURE_COMMIT_MSG"
fi

if [[ -z "$FEATURE_PR_TITLE" ]]; then
  if [[ -n "$FEATURE_COMMIT_MSG" ]]; then
    FEATURE_PR_TITLE="$FEATURE_COMMIT_MSG"
  else
    FEATURE_PR_TITLE="chore: ship ${FEATURE_BRANCH}"
  fi
fi

log "Pushing feature branch"
git push -u origin "$FEATURE_BRANCH"

if [[ -z "$FEATURE_PR_BODY_FILE" ]]; then
  tmp_feature_body="$(mktemp -t ship-alpha-feature-body.XXXXXX.md)"
  cleanup_files+=("$tmp_feature_body")
  cat > "$tmp_feature_body" <<BODY
## Summary
- automated feature shipment by \`scripts/ship_alpha.sh\`

## Validation
- cargo fmt --all
- cargo test --all-features -j $JOBS
- cargo clippy --all-targets --all-features -j $JOBS -- -D warnings
BODY
  FEATURE_PR_BODY_FILE="$tmp_feature_body"
fi

log "Creating feature PR"
feature_pr_out="$(run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
  gh pr create --base "$BASE_BRANCH" --head "$FEATURE_BRANCH" --title "$FEATURE_PR_TITLE" --body-file "$FEATURE_PR_BODY_FILE")"
feature_pr_url="$(extract_pr_url "$feature_pr_out")"
if [[ -z "$feature_pr_url" ]]; then
  feature_pr_url="$(run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
    gh pr view --head "$FEATURE_BRANCH" --json url --jq '.url')"
fi
[[ -n "$feature_pr_url" ]] || die "Could not resolve feature PR URL"
feature_pr_number="${feature_pr_url##*/}"
log "Feature PR: $feature_pr_url"

log "Waiting ${SLEEP_SECONDS}s before checking feature PR checks"
sleep "$SLEEP_SECONDS"

log "Watching feature PR checks"
run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
  gh pr checks "$feature_pr_number" --watch --interval "$CHECK_INTERVAL" >/dev/null

log "Merging feature PR"
run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
  gh pr merge "$feature_pr_number" --squash --delete-branch >/dev/null

log "Syncing $BASE_BRANCH"
git checkout "$BASE_BRANCH"
git pull --ff-only origin "$BASE_BRANCH"

if [[ -z "$RELEASE_VERSION" ]]; then
  RELEASE_VERSION="$(compute_next_alpha_version)"
fi

if [[ ! "$RELEASE_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+-alpha\.[0-9]+$ ]]; then
  die "Invalid --release-version '$RELEASE_VERSION' (expected X.Y.Z-alpha.N)"
fi

release_branch="chore/release-v${RELEASE_VERSION}"
release_tag="v${RELEASE_VERSION}"

log "Preparing release branch $release_branch"
git checkout -b "$release_branch"
sed -i -E "0,/^version = \".*\"/s//version = \"$RELEASE_VERSION\"/" Cargo.toml

release_notes_file="$(mktemp -t ship-alpha-release-notes.XXXXXX.md)"
cleanup_files+=("$release_notes_file")
move_unreleased_added_to_release "CHANGELOG.md" "$RELEASE_VERSION" "$RELEASE_DATE" "$release_notes_file"

git add Cargo.toml CHANGELOG.md
git commit -m "chore(release): prepare v${RELEASE_VERSION}"

log "Pushing release branch"
git push -u origin "$release_branch"

if [[ -z "$RELEASE_PR_BODY_FILE" ]]; then
  tmp_release_body="$(mktemp -t ship-alpha-release-body.XXXXXX.md)"
  cleanup_files+=("$tmp_release_body")
  cat > "$tmp_release_body" <<BODY
## Summary
- bump crate version to \`${RELEASE_VERSION}\`
- cut changelog entries from \`Unreleased\` into \`${RELEASE_VERSION}\`

## Notes
- prepared automatically by \`scripts/ship_alpha.sh\`
BODY
  RELEASE_PR_BODY_FILE="$tmp_release_body"
fi

log "Creating release PR"
release_pr_out="$(run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
  gh pr create --base "$BASE_BRANCH" --head "$release_branch" \
  --title "chore(release): prepare v${RELEASE_VERSION}" --body-file "$RELEASE_PR_BODY_FILE")"
release_pr_url="$(extract_pr_url "$release_pr_out")"
if [[ -z "$release_pr_url" ]]; then
  release_pr_url="$(run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
    gh pr view --head "$release_branch" --json url --jq '.url')"
fi
[[ -n "$release_pr_url" ]] || die "Could not resolve release PR URL"
release_pr_number="${release_pr_url##*/}"
log "Release PR: $release_pr_url"

log "Waiting ${SLEEP_SECONDS}s before checking release PR checks"
sleep "$SLEEP_SECONDS"

log "Watching release PR checks"
run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
  gh pr checks "$release_pr_number" --watch --interval "$CHECK_INTERVAL" >/dev/null

log "Merging release PR"
run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
  gh pr merge "$release_pr_number" --squash --delete-branch >/dev/null

log "Syncing $BASE_BRANCH after release merge"
git checkout "$BASE_BRANCH"
git pull --ff-only origin "$BASE_BRANCH"

log "Publishing prerelease $release_tag"
release_create_out="$(run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
  gh release create "$release_tag" --prerelease --title "$release_tag" --notes-file "$release_notes_file")"
release_url="$(extract_release_url "$release_create_out")"
if [[ -z "$release_url" ]]; then
  release_url="$(run_gh_with_retries "$GH_RETRIES" "$GH_RETRY_DELAY" \
    gh release view "$release_tag" --json url --jq '.url')"
fi

log "Done"
printf 'Feature PR: %s\n' "$feature_pr_url"
printf 'Release PR: %s\n' "$release_pr_url"
printf 'Release: %s\n' "$release_url"
