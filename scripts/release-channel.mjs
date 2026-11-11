/**
 * GitHub's releases/latest endpoint resolves only the latest published stable release.
 * Release candidates are pinned to their exact tag so they don't accidentally consume
 * a different release channel's manifest.
 */
export function updaterEndpointForRelease(repository, version, tag) {
  if (/-rc\./.test(version)) {
    return `https://github.com/${repository}/releases/download/${encodeURIComponent(tag)}/latest.json`;
  }
  return `https://github.com/${repository}/releases/latest/download/latest.json`;
}
