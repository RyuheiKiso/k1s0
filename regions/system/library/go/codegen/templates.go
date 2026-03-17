package codegen

// Go サービスの main.go テンプレート
var goMainTemplate = `package main

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"

	"{{.Domain}}-{{.ServiceName}}/internal/app"
)

// main はサービスのエントリーポイント。
// シグナルを受け取るまでサービスを実行し続ける。
func main() {
	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	if err := app.Run(ctx); err != nil {
		log.Fatalf("failed to run: %v", err)
	}
}
`

// Go サービスの go.mod テンプレート
var goModTemplate = `module github.com/k1s0-platform/{{.Tier}}-server-go-{{.ServiceName}}

go 1.24.0

toolchain go1.24.1
`

// Go サービスの app.go テンプレート
var goAppTemplate = `package app

import (
	"context"
	"log"
)

// Run はアプリケーションのメインループを実行する。
// ctx がキャンセルされると終了する。
func Run(ctx context.Context) error {
	log.Println("service started")
	<-ctx.Done()
	log.Println("service stopping")
	return nil
}
`

// Rust サービスの main.rs テンプレート
var rustMainTemplate = `//! {{.ServiceName}} サービスのエントリーポイント。

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("service started");
    tokio::signal::ctrl_c().await?;
    tracing::info!("service stopping");
    Ok(())
}
`

// Rust サービスの Cargo.toml テンプレート
var rustCargoTemplate = `[package]
name = "{{.ServiceName}}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
anyhow = "1"
`
