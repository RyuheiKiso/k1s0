export function normalizePath(path: string): string {
  return path.replace(/\\/g, '/');
}

export function toDisplayPath(workspaceRoot: string, path: string): string {
  const normalizedRoot = normalizePath(workspaceRoot).replace(/\/$/, '');
  const normalizedPath = normalizePath(path);

  if (!normalizedRoot) {
    return normalizedPath;
  }

  if (normalizedPath.startsWith(`${normalizedRoot}/`)) {
    return normalizedPath.slice(normalizedRoot.length + 1);
  }

  return normalizedPath;
}
