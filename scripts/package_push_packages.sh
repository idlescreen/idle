#!/usr/bin/env bash
# Called by idle/package.rs after a local package cut. Push packages repo to origin master.
#
# F-009 hardening (audit):
#   - Requires IDLE_RELEASE_TAG env var to match the version argument. No env = no push.
#   - Requires `git tag -s` (GPG signed) to succeed; no unsigned fallback.
#   - Pushes commit + tag atomically via `git push --follow-tags` (no race window).
set -euo pipefail
VERSION="${1:?version}"
TAG_ENV="${IDLE_RELEASE_TAG:-}"
if [ -z "$TAG_ENV" ]; then
  echo "Refusing push: IDLE_RELEASE_TAG env var is not set." >&2
  echo "Set IDLE_RELEASE_TAG=${VERSION} in the environment of the operator that runs package.rs." >&2
  exit 1
fi
if [ "$TAG_ENV" != "$VERSION" ]; then
  echo "Refusing push: IDLE_RELEASE_TAG (${TAG_ENV}) does not match version arg (${VERSION})." >&2
  exit 1
fi
PKG_DIR="$(cd "$(dirname "$0")/../packages" && pwd)"
tag="v${VERSION}"
cd "$PKG_DIR"
# Refuse to tag from any branch other than master — prevents accidental release-branch push.
current_branch="$(git rev-parse --abbrev-ref HEAD)"
if [ "$current_branch" != "master" ]; then
  echo "Refusing push: HEAD is on '${current_branch}', not master." >&2
  exit 1
fi
# gpg signing uses a UID on the secret key. If the repo's user.email doesn't match a
# UID on the operator's GPG key, `git tag -s` fails. Allow override via
# IDLE_RELEASE_SIGNING_EMAIL / IDLE_RELEASE_SIGNING_NAME; defaults match the
# project's GPG key UID.
SIGN_NAME="${IDLE_RELEASE_SIGNING_NAME:-jeryd}"
SIGN_EMAIL="${IDLE_RELEASE_SIGNING_EMAIL:-jerydleuck@gmail.com}"
if [ -n "${IDLE_RELEASE_SIGNING_KEY:-}" ]; then
  SIGN_ARGS=("-c" "user.signingkey=${IDLE_RELEASE_SIGNING_KEY}")
else
  SIGN_ARGS=()
fi
echo "Staging release commit (idempotent if nothing to commit)..."
git add .
if ! git diff --cached --quiet; then
  git commit -m "Release idle v${VERSION}"
else
  echo "  (no staged changes; skipping commit)"
fi
# Tag is created AFTER the commit so it points at the release commit, not the parent.
if ! git rev-parse --verify --quiet "refs/tags/${tag}" >/dev/null; then
  echo "Creating GPG-signed tag ${tag} on $(git rev-parse --short HEAD) as <${SIGN_NAME} <${SIGN_EMAIL}>>..."
  git "${SIGN_ARGS[@]}" -c "user.name=${SIGN_NAME}" -c "user.email=${SIGN_EMAIL}" tag -s -a "${tag}" -m "Release idle ${VERSION}"
fi
echo "Pushing commit + tag atomically to origin master..."
git push --follow-tags origin master
echo "Push complete (commit + tag ${tag})."
