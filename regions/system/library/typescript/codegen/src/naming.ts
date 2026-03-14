/**
 * 文字列をスネークケースに変換する。
 * 例: "fooBar" → "foo_bar", "FooBar" → "foo_bar", "foo-bar" → "foo_bar"
 */
export function toSnakeCase(s: string): string {
  return s
    .replace(/([a-z0-9])([A-Z])/g, '$1_$2')
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1_$2')
    .replace(/[-\s]+/g, '_')
    .toLowerCase();
}

/**
 * 文字列をパスカルケースに変換する。
 * 例: "foo_bar" → "FooBar", "foo-bar" → "FooBar"
 */
export function toPascalCase(s: string): string {
  return s
    .replace(/[-_\s]+(.)?/g, (_, c: string | undefined) =>
      c ? c.toUpperCase() : '',
    )
    .replace(/^(.)/, (_, c: string) => c.toUpperCase());
}

/**
 * 文字列をケバブケースに変換する。
 * 例: "fooBar" → "foo-bar", "FooBar" → "foo-bar", "foo_bar" → "foo-bar"
 */
export function toKebabCase(s: string): string {
  return s
    .replace(/([a-z0-9])([A-Z])/g, '$1-$2')
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1-$2')
    .replace(/[_\s]+/g, '-')
    .toLowerCase();
}

/**
 * 文字列をキャメルケースに変換する。
 * 例: "foo_bar" → "fooBar", "foo-bar" → "fooBar", "FooBar" → "fooBar"
 */
export function toCamelCase(s: string): string {
  const pascal = toPascalCase(s);
  return pascal.charAt(0).toLowerCase() + pascal.slice(1);
}
