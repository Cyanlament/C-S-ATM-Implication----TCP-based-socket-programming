# TypeScript ATM 作业版（GUI 美化版）

这版用 Electron 做客户端，界面更好看，演示也更丝滑。

## 快速运行

先进入目录：

cd ts-atm

1. 安装依赖

npm install

2. 启动服务端

npm run start:server

3. 新开一个终端，启动 GUI 客户端

npm run start:client

4. 可选：跑自动测试脚本

npm run test:case

## 文件说明

- 数据文件：data/accounts.json
- 异常日志：logs/exception.log
- 取款日志：logs/withdraw.log
- 运行日志：logs/server.log

## 打包给同学

在仓库根目录执行：

powershell -ExecutionPolicy Bypass -File scripts/package-ts.ps1

会生成：

packages/ts-atm-portable-win64.zip

解压后双击 start-server.bat 和 start-client.bat 就能直接用。
