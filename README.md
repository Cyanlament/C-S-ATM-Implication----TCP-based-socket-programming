# 网络作业 2：ATM 银行服务器

这是计算机网络作业 2 的代码。仓库里保留了两套实现：

- `rust-atm/`：Rust 服务端、egui 客户端、测试程序
- `ts-atm/`：TypeScript 服务端、Electron 客户端、测试程序

两套程序使用同一套 TCP 文本协议，默认端口都是 `2525`。

## 目录

| 路径 | 内容 |
| --- | --- |
| `rust-atm/` | Rust 版本 |
| `ts-atm/` | TypeScript/Electron 版本 |
| `docs/protocol.md` | 协议说明 |
| `docs/lab2-atm-experiment-report.md` | 实验报告 |
| `scripts/` | Windows 打包脚本 |

## 数据文件

两个版本各自使用自己目录下的两份文本文件：

| 文件 | 格式 |
| --- | --- |
| `users.txt` | `卡号 PIN` |
| `balances.txt` | `卡号 余额` |

示例：

```text
100001 1234
100002 1111
100003 0721
```

```text
100001 5000.00
100002 1200.50
100003 6000.00
```

服务端启动时读取这两份文件。取款成功后，程序会把新余额写回 `balances.txt`。

## 协议

客户端按行发送：

| 命令 | 说明 |
| --- | --- |
| `HELO <userid>` | 发送卡号 |
| `PASS <passwd>` | 发送 PIN |
| `BALA` | 查询余额 |
| `WDRA <amount>` | 取款 |
| `QUIT` | 退出 |

服务端响应：

| 响应 | 说明 |
| --- | --- |
| `500 AUTH REQUIRE` | 等待口令 |
| `525 OK!` | 认证成功或取款成功 |
| `401 ERROR!` | 请求失败 |
| `AMNT:<amount>` | 当前余额 |
| `BYE` | 会话结束 |

更完整的状态机见 [docs/protocol.md](docs/protocol.md)。

## 运行 Rust 版本

```bash
cd rust-atm
cargo run --bin server -- 2525
```

```bash
cd rust-atm
cargo run --bin client -- 127.0.0.1 2525
```

```bash
cd rust-atm
cargo run --bin test_case -- 127.0.0.1 2525 100001 1234 100
```

## 运行 TypeScript/Electron 版本

```bash
cd ts-atm
npm install
npm run start:server -- 2525
```

```bash
cd ts-atm
npm run start:client -- 127.0.0.1 2525
```

```bash
cd ts-atm
npm run test:case -- 127.0.0.1 2525 100001 1234 100
```

同机测试用 `127.0.0.1`。跨电脑测试时，把客户端 Host 改成服务端电脑的 WLAN IPv4 地址，并确认两台电脑在同一个局域网内。

## 打包

在仓库根目录运行：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/package-all.ps1
```

输出会放到 `packages/`：

- `rust-atm-portable-win64.zip`
- `ts-atm-portable-win64.zip`

`packages/` 是本地打包产物，不提交到 GitHub。需要发压缩包时，可以把 zip 放到 GitHub Release。

## Push 前保留这些

提交到 GitHub 时保留源码、文档、脚本和数据文件：

- `rust-atm/users.txt`
- `rust-atm/balances.txt`
- `ts-atm/users.txt`
- `ts-atm/balances.txt`

不要提交这些本地生成内容：

- `node_modules/`
- `dist/`
- `target/`
- `packages/`
- `logs/`
- `*.log`
