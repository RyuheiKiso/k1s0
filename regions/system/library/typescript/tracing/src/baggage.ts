export class Baggage {
  private entries = new Map<string, string>();

  set(key: string, value: string): void {
    this.entries.set(key, value);
  }

  get(key: string): string | undefined {
    return this.entries.get(key);
  }

  toHeader(): string {
    const parts: string[] = [];
    for (const [k, v] of this.entries) {
      parts.push(`${k}=${v}`);
    }
    return parts.join(',');
  }

  static fromHeader(s: string): Baggage {
    const baggage = new Baggage();
    if (!s) return baggage;
    for (const part of s.split(',')) {
      const eqIndex = part.indexOf('=');
      if (eqIndex > 0) {
        const key = part.substring(0, eqIndex).trim();
        const value = part.substring(eqIndex + 1).trim();
        baggage.entries.set(key, value);
      }
    }
    return baggage;
  }
}
