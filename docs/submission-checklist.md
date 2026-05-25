# 作业2 提交检查清单

## 代码

- [ ] `rust-atm/` 已按新协议更新
- [ ] `ts-atm/` 已按新协议更新
- [ ] 默认端口为 `2525`
- [ ] 协议命令为 `HELO / PASS / BALA / WDRA / QUIT`
- [ ] 服务端响应为 `500 AUTH REQUIRE / 525 OK! / 401 ERROR! / AMNT:<amount> / BYE`
- [ ] 服务端使用 `users.txt` 和 `balances.txt`

## 功能

- [ ] `HELO` 后返回 `500 AUTH REQUIRE`
- [ ] `PASS` 正确返回 `525 OK!`
- [ ] `BALA` 返回 `AMNT:<amount>`
- [ ] `WDRA` 成功时更新 `balances.txt`
- [ ] 余额不足时返回 `401 ERROR!`
- [ ] `QUIT` 后返回 `BYE`
- [ ] 支持多个客户端并发连接

## 日志

- [ ] `logs/exception.log` 可正常生成
- [ ] `logs/withdraw.log` 可正常生成

## 文档

- [ ] `docs/protocol.md` 已同步为 RFC-20242024 版本
- [ ] 实验报告已完成
- [ ] README 已更新为新协议和新数据文件说明

## 打包

- [ ] `scripts/package-rust.ps1` 已复制 `users.txt` / `balances.txt`
- [ ] `scripts/package-ts.ps1` 已复制 `users.txt` / `balances.txt`
- [ ] bat 文件支持启动 server / client / test_case

## GitHub

- [ ] 代码已 push
- [ ] 仓库说明完整
- [ ] `node_modules/`、`dist/`、`target/`、`packages/`、`*.log` 未进入提交
- [ ] 若需要压缩包，作为 GitHub Release 附件上传
