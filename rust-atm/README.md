# Rust ATM

Rust 版本包含三个程序：

| 程序 | 作用 |
| --- | --- |
| `server` | TCP 银行服务器 |
| `client` | egui 图形客户端 |
| `test_case` | 自动跑认证、查余额、取款和退出流程 |

默认监听端口是 `2525`。

## 数据文件

服务端启动时读取当前目录下的：

| 文件 | 内容 |
| --- | --- |
| `users.txt` | 卡号和 PIN |
| `balances.txt` | 卡号和余额 |

取款成功后会更新 `balances.txt`。

## 运行

启动服务端：

```bash
cargo run --bin server -- 2525
```

启动 GUI 客户端：

```bash
cargo run --bin client -- 127.0.0.1 2525
```

运行测试程序：

```bash
cargo run --bin test_case -- 127.0.0.1 2525 100001 1234 100
```

同机测试用 `127.0.0.1`。跨电脑测试时，客户端 Host 填服务端电脑的 WLAN IPv4 地址。

## 打包

在仓库根目录运行：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/package-rust.ps1
```

输出文件：

```text
packages/rust-atm-portable-win64.zip
```
