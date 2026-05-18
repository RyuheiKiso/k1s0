# tier2 第二業界 stub service

## 概要

製造業 (`pack/manufacturing`) と業界中立性を検証するための第二業界 stub。
サービス業をモデルとした最小実装。公開 API 中立性 (API neutrality) の CI 検証に使用する。

## 目的

1. 製造業固有語を含まない API 設計の実証
2. `pack/manufacturing` との API diff 検証
3. 業界横断層の実装が業界非依存であることの CI 確認

## 構造

stub_service/ は manufacturing/ と同型の構造を持ち、API neutrality を検証する。
