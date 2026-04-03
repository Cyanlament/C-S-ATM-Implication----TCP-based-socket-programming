# Rust ATM 作业版

这版是 Rust 实现，包含：

- server：服务端
- client：GUI 客户端
- test_case：自动测试脚本

## 快速运行

先进入目录：

cd rust-atm

1. 启动服务端

cargo run --bin server

2. 新开一个终端，启动 GUI 客户端

cargo run --bin client

3. 可选：跑自动测试

cargo run --bin test_case

## 文件说明

- 数据文件：data/accounts.json
- 异常日志：logs/exception.log
- 取款日志：logs/withdraw.log
- 运行日志：logs/server.log

## 打包给同学

在仓库根目录执行：

powershell -ExecutionPolicy Bypass -File scripts/package-rust.ps1

会生成：

packages/rust-atm-portable-win64.zip

解压后双击 start-server.bat 和 start-client.bat 就能用。
