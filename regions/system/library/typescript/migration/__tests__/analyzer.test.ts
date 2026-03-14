import { describe, it, expect } from 'vitest';
import { detectBreakingChanges, formatBreakingChange } from '../src/analyzer.js';

describe('detectBreakingChanges', () => {
  it('should detect DROP TABLE', () => {
    const changes = detectBreakingChanges('DROP TABLE users;');
    expect(changes).toHaveLength(1);
    expect(changes[0]).toEqual({ type: 'tableDropped', table: 'users' });
  });

  it('should detect DROP COLUMN', () => {
    const changes = detectBreakingChanges(
      'ALTER TABLE users DROP COLUMN email;',
    );
    expect(changes).toHaveLength(1);
    expect(changes[0]).toEqual({
      type: 'columnDropped',
      table: 'users',
      column: 'email',
    });
  });

  it('should detect SET NOT NULL', () => {
    const changes = detectBreakingChanges(
      'ALTER TABLE users ALTER COLUMN email SET NOT NULL;',
    );
    expect(changes).toHaveLength(1);
    expect(changes[0]).toEqual({
      type: 'notNullAdded',
      table: 'users',
      column: 'email',
    });
  });

  it('should detect type change', () => {
    const changes = detectBreakingChanges(
      'ALTER TABLE users ALTER COLUMN age SET DATA TYPE BIGINT;',
    );
    expect(changes).toHaveLength(1);
    expect(changes[0]).toEqual({
      type: 'columnTypeChanged',
      table: 'users',
      column: 'age',
      from: 'unknown',
      to: 'BIGINT',
    });
  });

  it('should detect RENAME COLUMN', () => {
    const changes = detectBreakingChanges(
      'ALTER TABLE users RENAME COLUMN old_name TO new_name;',
    );
    expect(changes).toHaveLength(1);
    expect(changes[0]).toEqual({
      type: 'columnRenamed',
      table: 'users',
      from: 'old_name',
      to: 'new_name',
    });
  });

  it('should return no breaking changes for ADD COLUMN', () => {
    const changes = detectBreakingChanges(
      'ALTER TABLE users ADD COLUMN email TEXT;',
    );
    expect(changes).toHaveLength(0);
  });

  it('should return empty array for invalid SQL', () => {
    const changes = detectBreakingChanges('NOT VALID SQL AT ALL !!!');
    expect(changes).toHaveLength(0);
  });
});

describe('formatBreakingChange', () => {
  it('should format columnDropped', () => {
    expect(
      formatBreakingChange({
        type: 'columnDropped',
        table: 'users',
        column: 'email',
      }),
    ).toBe('Column users.email dropped');
  });

  it('should format columnTypeChanged', () => {
    expect(
      formatBreakingChange({
        type: 'columnTypeChanged',
        table: 'users',
        column: 'age',
        from: 'INT',
        to: 'BIGINT',
      }),
    ).toBe('Column users.age type changed from INT to BIGINT');
  });

  it('should format tableDropped', () => {
    expect(
      formatBreakingChange({ type: 'tableDropped', table: 'users' }),
    ).toBe('Table users dropped');
  });

  it('should format notNullAdded', () => {
    expect(
      formatBreakingChange({
        type: 'notNullAdded',
        table: 'users',
        column: 'email',
      }),
    ).toBe('NOT NULL added to users.email');
  });

  it('should format columnRenamed', () => {
    expect(
      formatBreakingChange({
        type: 'columnRenamed',
        table: 'users',
        from: 'old_name',
        to: 'new_name',
      }),
    ).toBe('Column users.old_name renamed to new_name');
  });
});
