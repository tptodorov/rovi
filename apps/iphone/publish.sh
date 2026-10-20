#!/usr/bin/env bash
# Publishes a clean build of this app to the gh-pages branch, served at
# https://tptodorov.github.io/rovi/iphone/. Rollback: see README.
set -euo pipefail
cd "$(dirname "$0")"
[ -z "$(git status --porcelain -- .)" ] || { echo "Commit apps/iphone before publishing" >&2; exit 1; }
npm run build
revision=$(cat dist/iphone/version.txt)
site=$(mktemp -d)
trap 'git worktree remove --force "$site"' EXIT
if git fetch -q origin gh-pages 2>/dev/null; then
  git worktree add -q --detach "$site" FETCH_HEAD
else
  git worktree add -q --detach "$site"
  git -C "$site" checkout -q --orphan gh-pages
  git -C "$site" rm -rqf .
fi
rm -rf "$site/iphone"
cp -r dist/iphone "$site/iphone"
touch "$site/.nojekyll"
git -C "$site" add -A
git -C "$site" commit -qm "pages: iphone controller $revision" || { echo "Already published $revision"; exit 0; }
git -C "$site" push -q origin HEAD:gh-pages
echo "Published $revision to gh-pages"
