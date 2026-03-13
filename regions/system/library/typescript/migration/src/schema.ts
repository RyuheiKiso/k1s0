export interface Column {
  name: string;
  dataType: string;
  nullable: boolean;
  default?: string;
}

export interface Index {
  name: string;
  table: string;
  columns: string[];
  unique: boolean;
}

export type Constraint =
  | { type: 'primaryKey'; columns: string[] }
  | { type: 'foreignKey'; columns: string[]; refTable: string; refColumns: string[] }
  | { type: 'unique'; columns: string[] }
  | { type: 'check'; expression: string };

export interface Table {
  name: string;
  columns: Column[];
  indexes: Index[];
  constraints: Constraint[];
}

export interface Schema {
  tables: Table[];
}
