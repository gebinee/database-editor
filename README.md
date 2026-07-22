# gebinee 单词数据库编辑器

基于 Tauri 2 + Vue 3 的桌面端单词数据库管理工具，支持单词与注音结果的增删改查、Excel 批量导入导出、自定义字体和主题切换。

## 功能特性

- **单词管理**：添加、查询、编辑、删除单词及其注音结果
- **分页浏览**：支持分页列表、模糊搜索、正序/倒序排序
- **Excel 导入**：从 Excel 文件批量导入，自动校验重复、非法字符等问题，支持预览和行内编辑（独立窗口）
- **Excel 导出**：将问题项导出为 Excel 便于修正
- **外观定制**：支持浅色/深色/跟随系统主题，单词字体、注音字体、UI 字体（西文 + 中文）独立配置
- **自定义字体**：支持上传 ttf/otf/woff/woff2 字体文件
- **自动更新**：内置更新检查，支持 GitHub Release 分发

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + Element Plus + Pinia |
| 构建工具 | Vite 8 |
| 数据库 | SQLite（rusqlite, bundled） |
| Excel 处理 | calamine（读）+ rust_xlsxwriter（写） |
| 组件库 | [@gebinee/components](https://github.com/gebinee/components) |

## 开发环境要求

- **Node.js** >= 18
- **Rust** 最新稳定版（通过 [rustup](https://rustup.rs/) 安装）
- **Windows**：需要 Microsoft Visual Studio C++ Build Tools
- **macOS**：需要 Xcode Command Line Tools
- **Linux**：需要 `libwebkit2gtk`、`libgtk-3` 等系统库

## 快速开始

```bash
# 克隆项目
git clone https://github.com/gebinee/database-editor.git
cd database-editor

# 安装前端依赖
npm install

# 启动开发服务器（桌面窗口 + 热更新）
npm run tauri dev
```

## 项目结构

```
database-editor/
├── src/                        # 前端源码
│   ├── main.js                 # 主窗口入口
│   ├── import-main.js          # 导入窗口入口
│   ├── App.vue                 # 根组件（布局、初始化）
│   ├── api/                    # Tauri 命令封装
│   │   ├── db.js               # 数据库操作 API
│   │   └── settings.js         # 设置相关 API
│   ├── assets/                 # 静态资源（内置字体等）
│   ├── components/             # Vue 组件
│   │   ├── AddPanel.vue        # 添加单词面板
│   │   ├── QueryPanel.vue      # 查询单词面板
│   │   ├── WordTable.vue       # 单词列表表格
│   │   ├── EditDialog.vue      # 编辑对话框
│   │   └── SettingsDialog.vue  # 设置对话框（封装组件库）
│   ├── stores/                 # Pinia 状态管理
│   │   ├── settings.js         # 设置状态
│   │   └── query.js            # 查询状态
│   ├── utils/                  # 工具函数
│   │   ├── error.js            # 错误信息提取
│   │   └── validation.js       # 输入校验
│   └── views/
│       ├── MainView.vue        # 主视图（左右布局）
│       └── ImportView.vue      # Excel 导入视图（独立窗口）
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs             # 程序入口
│   │   ├── lib.rs              # Tauri 应用初始化、命令注册、窗口事件
│   │   ├── commands.rs         # Tauri 命令实现（前后端桥梁）
│   │   ├── db.rs               # SQLite 数据库操作
│   │   ├── models.rs           # 数据模型定义
│   │   ├── error.rs            # 统一错误类型
│   │   ├── excel.rs            # Excel 读写
│   │   └── settings.rs         # 设置持久化
│   ├── capabilities/           # Tauri 权限配置（main + import 窗口）
│   ├── icons/                  # 应用图标
│   ├── tauri.conf.json         # Tauri 配置（含 import 窗口预配置）
│   └── Cargo.toml              # Rust 依赖
├── index.html                  # 主窗口 HTML 入口
├── import.html                 # 导入窗口 HTML 入口
├── vite.config.js              # Vite 配置（多页面构建）
└── package.json                # 前端依赖
```

## 构建打包

```bash
# 生产构建（生成安装包）
npm run tauri build

# 产物位于 src-tauri/target/release/bundle/
```

## 架构说明

### 前后端交互

前端通过 `@tauri-apps/api` 的 `invoke()` 调用 Rust 端注册的 Tauri 命令，命令名使用 snake_case 自动映射。所有命令返回统一格式的错误类型 `AppError`（`{ kind, message }`），前端通过 `errorMessage()` 函数转为中文提示。

### 多窗口架构

应用包含两个窗口：

- **主窗口**（`main`）：单词管理界面，启动时显示
- **导入窗口**（`import`）：Excel 导入预览界面，预创建但隐藏（`visible: false`），点击"从 Excel 导入"时显示

窗口间通信机制：
- **文件路径传递**：Rust 端通过 `AppState.pending_import_path` 全局状态暂存路径，导入窗口通过 `take_pending_import_path` 命令取出
- **设置同步**：主窗口保存设置后通过 `app.emit("settings:changed")` 全局事件通知导入窗口重新应用主题/字体
- **窗口生命周期**：导入窗口拦截关闭请求改为隐藏（避免动态重建白屏），主窗口关闭时强制销毁导入窗口

### 数据库设计

使用 SQLite，单表结构：

```sql
CREATE TABLE kv_store (
    key   TEXT PRIMARY KEY,   -- 单词
    value TEXT NOT NULL       -- 注音结果
);
```

- 使用 `journal_mode=DELETE`，不使用 WAL 模式
- 应用退出时显式关闭连接，避免残留 journal 文件
- 卸载时通过 NSIS 钩子保留数据库文件

### 输入校验

- **单词词法校验**：后端 `validate_key()` 校验单词必须符合 `^[a-zA-Z\-]+$`，在 `add_entry`、`update_entry_key`、`import_entries` 三处调用
- **Excel 导入去重**：三层去重（后端读取、前端预览、后端插入），相同 key+value 的行视为一行

### 应用状态管理

Rust 端通过 `AppState` 管理全局状态（数据库连接、设置、错误信息、待导入路径），使用 `Mutex` 保护并发访问。前端通过 Pinia store 管理 UI 状态（查询条件、分页等）。

## 许可证

[MIT](LICENSE)