# 网络实验 Lab2 ATM 项目

这是一个学生作业仓库，包含两个版本：

- Rust 版本：rust-atm/
- TypeScript 版本：ts-atm/
- 协议说明：docs/protocol.md

两个版本都满足老师要求：

- TCP Socket 通信
- 默认端口 2525
- 客户端有 GUI，服务端无 GUI
- 服务端读取数据文件
- 服务端记录异常日志和取款日志

## 我该先跑哪个？

- 想要更漂亮 GUI：优先用 TypeScript 版
- 想看 Rust 网络编程：用 Rust 版

## 一键打包（发给同学直接用）

在仓库根目录运行：

powershell -ExecutionPolicy Bypass -File scripts/package-all.ps1

打包后在 packages/ 目录会看到：

- rust-atm-portable-win64.zip
- ts-atm-portable-win64.zip

## 单独看说明

- Rust 使用说明：rust-atm/README.md
- TypeScript 使用说明：ts-atm/README.md
