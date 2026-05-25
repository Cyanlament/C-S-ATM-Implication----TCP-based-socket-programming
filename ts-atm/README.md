# TypeScript/Electron ATM

TypeScript 版本包含：

| 路径 | 作用 |
| --- | --- |
| `src/server.ts` | TCP 银行服务器 |
| `src/client/` | Electron 图形客户端 |
| `scripts/test_case.ts` | 自动跑认证、查余额、取款和退出流程 |

默认监听端口是 `2525`。

## 数据文件

服务端启动时读取项目目录下的：

| 文件 | 内容 |
| --- | --- |
| `users.txt` | 卡号和 PIN |
| `balances.txt` | 卡号和余额 |

取款成功后会更新 `balances.txt`。

## 运行

安装依赖：

```bash
npm install
```

启动服务端：

```bash
npm run start:server -- 2525
```

启动 GUI 客户端：

```bash
npm run start:client -- 127.0.0.1 2525
```

运行测试程序：

```bash
npm run test:case -- 127.0.0.1 2525 100001 1234 100
```

同机测试用 `127.0.0.1`。跨电脑测试时，客户端 Host 填服务端电脑的 WLAN IPv4 地址。

## 打包

在仓库根目录运行：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/package-ts.ps1
```

输出文件：

```text
packages/ts-atm-portable-win64.zip
```

`node_modules/`、`dist/` 和 `packages/` 都是本地生成内容，不提交到 GitHub。
