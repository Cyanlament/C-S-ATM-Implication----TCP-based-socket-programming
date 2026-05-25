# 作业2：ATM 和银行服务器通信程序开发实验报告

## 一、基本信息

- 姓名：（请填写）
- 学号：（请填写）
- 班级：（请填写）
- 指导教师：胡海波
- GitHub 仓库：（请填写）
- 日期：2026 年 4 月

## 二、实验目的

1. 理解应用层协议的设计方法，掌握基于 TCP 的自定义协议报文格式。
2. 使用 Socket 编程实现 ATM 客户端与银行服务器端之间的请求、响应通信。
3. 掌握服务端状态管理、用户认证、余额查询与支取等业务逻辑。
4. 学会使用文本文件保存用户口令和账户余额，并在业务成功后进行持久化更新。
5. 通过 Rust 与 TypeScript 两种实现，比较不同语言在网络编程与客户端交互界面方面的实现方式。

## 三、协议设计说明

本实验采用基于 TCP 的文本行协议，默认端口为 `2525`。所有请求和响应均使用 UTF-8 编码，并以换行符 `\n` 作为报文结束标记。

### 3.1 客户端 -> 服务器报文

| 报文 | 说明 |
| --- | --- |
| `HELO <userid>` | ATM 插卡后发送卡号/用户号 |
| `PASS <passwd>` | 用户输入口令后发送到服务器进行验证 |
| `BALA` | 查询当前账户余额 |
| `WDRA <amount>` | 请求支取指定金额 |
| `QUIT` | 结束当前会话 |

### 3.2 服务器 -> 客户端报文

| 报文 | 说明 |
| --- | --- |
| `500 AUTH REQUIRE` | 收到合法 `HELO` 后，提示客户端继续发送口令 |
| `525 OK!` | 认证成功或取款成功 |
| `401 ERROR!` | 非法请求、认证失败、余额不足或状态错误 |
| `AMNT:<amount>` | 返回当前余额 |
| `BYE` | 正常结束会话 |

### 3.3 服务端状态机

服务器为每个 TCP 连接维护独立状态：

`STATE_INIT` -> `HELO` -> `STATE_AUTH_REQUIRED` -> `PASS` -> `STATE_LOGGED_IN`

只有在 `STATE_LOGGED_IN` 下允许执行：

- `BALA`
- `WDRA`

考虑到题目中的客户端流程说明写明“认证失败后可重试或退出”，程序允许在任意阶段发送 `QUIT` 正常结束会话，服务器回复 `BYE` 后断开连接。

若在其他状态下执行非法命令，服务器统一返回：

```text
401 ERROR!
```

## 四、数据文件与程序结构

### 4.1 数据文件

每个实现版本目录下都使用两份文本文件：

- `users.txt`
- `balances.txt`

示例内容如下：

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

服务器启动时读取两份文件到内存。用户成功支取金额后，立即更新内存中的余额，并重写整个 `balances.txt` 文件。

### 4.2 Rust 版本结构

| 文件 | 作用 |
| --- | --- |
| `rust-atm/src/lib.rs` | 协议常量、请求解析、文本文件读写、日志追加 |
| `rust-atm/src/bin/server.rs` | 多线程 TCP 服务端、状态机、余额更新与日志 |
| `rust-atm/src/bin/client.rs` | GUI 客户端 |
| `rust-atm/src/bin/test_case.rs` | 自动测试脚本 |
| `rust-atm/users.txt` | 用户卡号与口令 |
| `rust-atm/balances.txt` | 用户余额 |

### 4.3 TypeScript 版本结构

| 文件 | 作用 |
| --- | --- |
| `ts-atm/src/common/protocol.ts` | 协议常量与请求解析 |
| `ts-atm/src/server.ts` | TCP 服务端、状态机、文本文件读写与日志 |
| `ts-atm/src/client/main.ts` | Electron 主进程，负责 TCP 连接转发 |
| `ts-atm/src/client/renderer/renderer.ts` | GUI 页面逻辑 |
| `ts-atm/scripts/test_case.ts` | 自动测试脚本 |
| `ts-atm/users.txt` | 用户卡号与口令 |
| `ts-atm/balances.txt` | 用户余额 |

## 五、运行步骤

### 5.1 Rust 版本

进入目录：

```bash
cd rust-atm
```

启动服务端：

```bash
cargo run --bin server -- 2525
```

启动 GUI 客户端：

```bash
cargo run --bin client -- 127.0.0.1 2525
```

运行自动测试：

```bash
cargo run --bin test_case -- 127.0.0.1 2525 100001 1234 100
```

### 5.2 TypeScript 版本

进入目录：

```bash
cd ts-atm
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

运行自动测试：

```bash
npm run test:case -- 127.0.0.1 2525 100001 1234 100
```

同机测试使用 `127.0.0.1`。跨机测试时，客户端 Host 改为服务端电脑的 WLAN IPv4 地址，例如 `172.19.153.48`。

## 六、测试结果

### 6.1 正常流程

```text
>> HELO 100001
<< 500 AUTH REQUIRE
>> PASS 1234
<< 525 OK!
>> BALA
<< AMNT:5000.00
>> WDRA 100
<< 525 OK!
>> BALA
<< AMNT:4900.00
>> QUIT
<< BYE
```

### 6.2 密码错误

```text
>> HELO 100001
<< 500 AUTH REQUIRE
>> PASS wrong_password
<< 401 ERROR!
>> QUIT
<< BYE
```

### 6.3 余额不足

```text
>> HELO 100001
<< 500 AUTH REQUIRE
>> PASS 1234
<< 525 OK!
>> WDRA 9999999
<< 401 ERROR!
>> QUIT
<< BYE
```

### 6.4 多客户端并发

两个客户端同时连接服务端并分别执行认证、查询余额和取款操作时，服务端能够为每个连接维护独立状态，不会因为某个客户端的认证或取款操作影响另一个客户端的会话控制流程。

## 七、日志与持久化结果

服务端会在 `logs/` 目录下记录：

- `server.log`
- `exception.log`
- `withdraw.log`

典型记录如下：

```text
[timestamp] 127.0.0.1:50001 password failed for 100001
[timestamp] 127.0.0.1:50002 insufficient funds user=100001 request=9999999.00 balance=4900.00
[timestamp] 127.0.0.1:50003 user=100001 withdraw=100.00 before=5000.00 after=4900.00
```

取款成功后，`balances.txt` 中对应用户余额会从：

```text
100001 5000.00
```

更新为：

```text
100001 4900.00
```

## 八、运行截图要求

提交报告前补上以下真实截图：

1. 认证成功界面
2. 查询余额界面
3. 取款成功界面
4. 余额不足界面
5. 退出界面
6. `exception.log` 与 `withdraw.log` 截图
7. `balances.txt` 更新前后对比截图

## 九、遇到的问题及解决方法

1. **协议旧版本与新作业要求不一致**  
   最初实现中使用的是旧版请求 `BYE` 和响应 `500 AUTH REQUIRED!`。根据本次作业要求，需要改为 `QUIT` 和 `500 AUTH REQUIRE`。因此对 Rust 和 TypeScript 两个版本的协议常量、请求解析、测试脚本、GUI 文案和文档进行了统一修正。

2. **数据文件格式与作业要求不一致**  
   旧版本使用 `accounts.json` 保存密码和余额，而本次作业要求使用 `users.txt` 和 `balances.txt` 两个文本文件。因此重构了服务端的数据加载与保存逻辑，把密码和余额拆分保存，并在取款成功后重写整个 `balances.txt` 文件。

3. **非法命令状态处理需要更严格**  
   为符合题目中“除登录成功状态外非法操作统一返回 401 ERROR!”的要求，重新整理了服务端状态机，确保 `BALA`、`WDRA` 只有在登录成功后才能执行；同时考虑到题目示例中明确提到认证失败后可退出，因此保留了 `QUIT` 在任意阶段都可正常结束会话的兼容处理。

4. **打包脚本与便携运行包需要同步更新**  
   原打包脚本复制的是 `accounts.json`。修改协议和数据文件后，需要同步调整 `package-rust.ps1`、`package-ts.ps1` 和便携包中的 `bat` 启动脚本，改为携带 `users.txt` 与 `balances.txt`。

## 十、心得体会

这次实现里，最需要注意的是 TCP 只提供字节流，应用层必须自己约定报文边界、命令顺序和错误返回。服务端状态机写清楚以后，认证前后能执行哪些命令就比较明确，测试脚本也更容易覆盖异常分支。

另一个收获是文件持久化和并发访问不能分开看。余额存在文本文件里并不复杂，但多个客户端同时取款时，内存余额和写回文件必须放在同一套同步逻辑里处理，否则日志看起来成功，数据文件却可能被后一次写入覆盖。

两个版本都按同一份协议实现，运行步骤和测试用例保持一致。Rust 版本更适合观察服务端状态和线程处理，TypeScript/Electron 版本更适合做 GUI 演示。
