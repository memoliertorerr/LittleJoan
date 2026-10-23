#!/usr/bin/env bash
# CI (Linux): `apt-get install` that moves on from a mirror that crawls.
#
#   bash scripts/ci/apt-install.sh <package | ./file.deb>...
#
# GitHub's Ubuntu runners take packages from an Azure mirror first (a mirror list in the sources:
# that mirror, then the Ubuntu archive). It has been seen delivering ~60 KB/s for 45 minutes —
# too slow to count as failing, so apt never moved on and the job hit its time limit (main's run
# 39875402501, smoke (linux-22.04), 2026-10-23, both attempts). So here:
# - apt retries and drops stalled connections (Acquire::Retries, Acquire::http(s)::Timeout);
# - a plain Ubuntu mirror named in the sources gets a mirror list of its own (it, then the
#   archive), as the runner's own sources already have;
# - `apt-get update` and the download run under a deadline each; past it apt is stopped, the
#   Ubuntu archive (ports.ubuntu.com on arm64) goes first in every mirror list, and the step goes
#   on from what was already fetched. What gets installed is the same either way; only where it
#   comes from changes.
#
# Away from CI (a test): LJ_APT_ETC (default /etc/apt), LJ_APT_SUDO (default sudo; empty for none),
# LJ_APT_GET (default apt-get), LJ_APT_DEADLINE (seconds per try, default 480). Plain bash 3.2
# features only, so it can be tried on a Mac against a folder of fake sources.
set -euo pipefail

etc="${LJ_APT_ETC:-/etc/apt}"
sudo_cmd="${LJ_APT_SUDO-sudo}"
apt_get="${LJ_APT_GET:-apt-get}"
deadline="${LJ_APT_DEADLINE:-480}"

as_root() {
  if [ -n "$sudo_cmd" ]; then "$sudo_cmd" "$@"; else "$@"; fi
}

case "$(dpkg --print-architecture 2>/dev/null || uname -m)" in
  amd64 | x86_64 | i386) official="http://archive.ubuntu.com/ubuntu/" ;;
  *) official="http://ports.ubuntu.com/ubuntu-ports/" ;;
esac

as_root mkdir -p "$etc/apt.conf.d"
printf '%s\n' 'Acquire::Retries "3";' 'Acquire::http::Timeout "30";' 'Acquire::https::Timeout "30";' \
  | as_root tee "$etc/apt.conf.d/99-little-joan-ci" > /dev/null

# The sources files, one per line.
source_files() {
  local file
  for file in "$etc/sources.list" "$etc"/sources.list.d/*.list "$etc"/sources.list.d/*.sources; do
    if [ -f "$file" ]; then printf '%s\n' "$file"; fi
  done
}

# Plain Ubuntu archive mirrors in the sources (not the archive itself, not a mirror list yet), each
# into a mirror list of its own: it first, the archive second.
index=0
while IFS= read -r uri; do
  if [ "${uri%/}/" = "$official" ]; then continue; fi
  index=$((index + 1))
  list="$etc/little-joan-mirrors-$index.txt"
  printf '%s\tpriority:1\n%s\tpriority:2\n' "${uri%/}/" "$official" | as_root tee "$list" > /dev/null
  while IFS= read -r file; do
    as_root env LJ_URI="$uri" LJ_LIST="$list" perl -pi -e \
      's{(^|\s)\Q$ENV{LJ_URI}\E(?=\s|$)}{$1mirror+file:$ENV{LJ_LIST}}g' "$file"
  done < <(source_files)
  echo "apt: $uri now goes through $list (it, then $official)"
done < <(source_files | while IFS= read -r file; do
  perl -ne 'print "$1\n" while m{(?:^|\s)(https?://\S*ubuntu\S*/ubuntu(?:-ports)?/?)(?=\s|$)}g' "$file"
done | sort -u)

# Every mirror list the sources use now.
lists=()
while IFS= read -r list; do
  lists+=("$list")
  echo "apt: mirror list $list:"
  sed 's/^/  /' "$list" || true
done < <(source_files | while IFS= read -r file; do
  perl -ne 'print "$1\n" while m{mirror\+file:(\S+)}g' "$file"
done | sort -u)

preferred=0
# The archive first in every mirror list (once): it gets priority 1, the others theirs + 10.
prefer_archive() {
  local list text
  if [ "$preferred" = 1 ]; then return 0; fi
  preferred=1
  for list in ${lists[@]+"${lists[@]}"}; do
    if [ ! -f "$list" ]; then continue; fi
    text="$(printf '%s\tpriority:1\n' "$official"; perl -pe 's/priority:(\d+)/"priority:" . ($1 + 10)/ge' "$list")"
    printf '%s\n' "$text" | as_root tee "$list" > /dev/null
    echo "apt: $list now starts with $official"
  done
}

# Runs `apt-get <args>` under the deadline; when it doesn't finish (or fails), the archive goes
# first and it runs once more, as long as it takes (the job's own limit still holds).
step() {
  local what="$1" started=$SECONDS
  shift
  if as_root timeout --kill-after=30 "$deadline" "$apt_get" "$@"; then
    echo "apt: $what took $((SECONDS - started)) s"
    return 0
  fi
  echo "::warning::apt: $what didn't finish within ${deadline} s (or failed) after $((SECONDS - started)) s; the Ubuntu archive goes first now"
  prefer_archive
  started=$SECONDS
  if [ "$what" != update ]; then as_root "$apt_get" update; fi
  as_root "$apt_get" "$@"
  echo "apt: $what took $((SECONDS - started)) s more"
}

step update update
step download install -y --download-only "$@"
# Everything is in apt's cache now, so this fetches nothing. (Not `--no-download`: with it apt
# 2.4 and 2.7 refuse a local .deb — "Internal Error, Pathname to install is not absolute", CI.)
as_root "$apt_get" install -y "$@"
