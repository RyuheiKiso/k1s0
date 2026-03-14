// ComponentRegistry はビルディングブロックコンポーネントの登録・管理を行うクラス。
// Map を用いてコンポーネントを名前で管理し、一括初期化・クローズ・ステータス取得をサポートする。

import type { Component, ComponentStatus } from './component.js';

export class ComponentRegistry {
  // 登録されたコンポーネントを名前をキーとして管理するマップ
  private readonly components = new Map<string, Component>();

  // コンポーネントをレジストリに登録する。同名が既に存在する場合はエラーをスローする。
  register(component: Component): void {
    if (this.components.has(component.name)) {
      throw new Error(`コンポーネント '${component.name}' は既に登録されています`);
    }
    this.components.set(component.name, component);
  }

  // 名前でコンポーネントを取得する。存在しない場合は undefined を返す。
  get(name: string): Component | undefined {
    return this.components.get(name);
  }

  // 登録済みの全コンポーネントを順次初期化する。
  // いずれかの初期化に失敗した場合、その時点でエラーをスローする。
  async initAll(): Promise<void> {
    for (const [name, component] of this.components) {
      try {
        await component.init();
      } catch (err) {
        throw new Error(`コンポーネント '${name}' の初期化に失敗しました: ${err}`);
      }
    }
  }

  // 登録済みの全コンポーネントを順次クローズする。
  // いずれかのクローズに失敗した場合、その時点でエラーをスローする。
  async closeAll(): Promise<void> {
    for (const [name, component] of this.components) {
      try {
        await component.close();
      } catch (err) {
        throw new Error(`コンポーネント '${name}' のクローズに失敗しました: ${err}`);
      }
    }
  }

  // 登録済みの全コンポーネントのステータスをオブジェクトとして返す。
  async statusAll(): Promise<Record<string, ComponentStatus>> {
    const statuses: Record<string, ComponentStatus> = {};
    for (const [name, component] of this.components) {
      statuses[name] = await component.status();
    }
    return statuses;
  }
}
