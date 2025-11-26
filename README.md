# 栏目映射管理工具

一个基于 Tauri + Vue 3 的桌面应用程序，用于批量管理和维护 `theme*.json` 文件中的本地/国网栏目ID映射关系。

## ✨ 功能特性

- 🔍 **自动扫描识别**：自动扫描目录下所有 `theme*.json` 文件，识别 `sExtOptions` 段落中的 `portal_frag_*` 映射关系
- 📊 **映射统计**：按本地栏目ID去重统计，清晰展示文件数量和映射数量
- ⚠️ **重复检测**：自动检测本地栏目ID和国网栏目ID的重复情况，并给出明确提示
- 📝 **批量新增**：支持批量添加映射关系，一键同步到所有 theme 文件
- 💾 **自动备份**：写入前自动创建备份，确保数据安全可回滚
- 📤 **导出功能**：导出去重后的映射项为完整的 sExtOptions JSON 格式
- 📥 **导入功能**：支持导入完整的 sExtOptions JSON 文件，自动替换现有映射
- 🎨 **现代化界面**：采用 Apple/Google 风格的扁平化设计，界面简洁美观

## 🛠️ 技术栈

- **前端框架**：Vue 3 + TypeScript + Vite
- **状态管理**：Pinia
- **桌面框架**：Tauri 1.5
- **后端语言**：Rust
- **构建工具**：Vite + Cargo

## 📦 安装与构建

### 快速开始

```bash
# 克隆仓库
git clone <repository-url>
cd addColumn/app

# 安装依赖
npm install

# 开发模式
npm run tauri:dev
```

### 环境要求

- Node.js 18+ 
- Rust 1.70+
- 系统依赖（Windows 需要 WebView2）

### 开发环境

```bash
# 克隆项目
git clone <repository-url>
cd addColumn

# 进入应用目录
cd app

# 安装依赖
npm install

# 开发模式运行
npm run tauri:dev
```

### 生产构建

#### 方式一：使用 Tauri CLI（推荐）

```bash
# 构建桌面应用
npm run tauri:build
```

#### 方式二：手动构建（如果 CLI 构建遇到问题）

```bash
# 1. 构建前端
cd app
npm run build

# 2. 构建 Rust 后端
cd src-tauri
cargo build --release

# 3. 可执行文件位于
# app/src-tauri/target/release/add-column-manager.exe
```

**注意**：
- Windows 构建需要安装 [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)
- 首次构建可能需要较长时间（需要下载 Rust 依赖）
- 构建产物位于 `app/src-tauri/target/release/` 目录
- Windows 会生成 `.exe` 安装程序，位于 `app/src-tauri/target/release/bundle/msi/` 目录
- 如果遇到 `custom-protocol` feature 错误，可以尝试手动构建方式

## 🚀 使用指南

### 1. 选择目标目录

点击「浏览」按钮选择包含 `theme*.json` 文件的目录，或直接在输入框粘贴路径。

### 2. 扫描文件

点击「扫描」按钮，程序会自动识别所有 `theme*.json` 文件中的 `sExtOptions` 段落，提取 `portal_frag_*` 映射关系。

### 3. 查看结果

扫描后可在下方查看每个文件的映射详情，包括：
- 本地栏目ID
- 国网栏目ID
- 状态（正常/本地ID重复/国网ID重复）

### 4. 批量新增

在批量新增区域填写本地栏目ID和国网栏目ID，点击「写入所有文件」即可同步到所有 theme 文件。

### 5. 导出/导入

- **导出**：点击「导出」按钮，将去重后的映射项导出为完整的 sExtOptions JSON 格式
- **导入**：点击「导入」按钮，选择 JSON 文件导入，程序会：
  - 自动检测重复项并提示
  - 显示确认对话框
  - 自动创建备份
  - 替换现有映射

### 6. 注意事项

- **本地栏目ID**：必须唯一，重复时会标记为错误
- **国网栏目ID**：可以重复，但会给出警告提示
- **自动备份**：每次写入前会自动创建备份，备份路径显示在操作结果中

## 📁 项目结构

```
addColumn/
├── app/                    # 前端应用目录
│   ├── src/
│   │   ├── components/     # Vue 组件
│   │   ├── stores/         # Pinia 状态管理
│   │   ├── types/          # TypeScript 类型定义
│   │   └── assets/         # 静态资源
│   ├── src-tauri/          # Tauri 后端
│   │   ├── src/
│   │   │   └── main.rs     # Rust 主程序
│   │   ├── Cargo.toml      # Rust 依赖配置
│   │   └── tauri.conf.json # Tauri 配置文件
│   └── package.json        # Node.js 依赖配置
├── Example/                # 示例文件目录
│   └── theme*.json         # 示例 theme 文件
└── README.md               # 项目说明文档
```

## 🔧 配置说明

### Tauri 权限配置

应用需要以下权限（已在 `tauri.conf.json` 中配置）：

- `dialog.open`：打开文件对话框
- `dialog.save`：保存文件对话框
- `dialog.ask`：确认对话框
- `fs.all`：文件系统操作
- `shell.open`：打开外部文件夹

## 📝 开发说明

### 核心功能实现

1. **映射解析**：直接从文本解析 `portal_frag_*` 条目，支持检测重复的 key（JSON5 格式）
2. **去重检测**：统计本地ID和国网ID的出现次数，自动标记重复状态
3. **文件操作**：保留注释和格式，精确插入新映射项
4. **备份机制**：按时间戳创建备份目录，确保数据安全

### 代码规范

- 使用 JSDoc 注释
- TypeScript 严格模式
- Rust 代码遵循官方规范

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

MIT License

## 👤 作者

项目维护者

---

## 📦 发布与分发

### GitHub Releases

项目配置了 GitHub Actions 自动构建工作流，当推送 tag（格式：`v*`）时会自动构建并上传构建产物。

### 手动构建

如果自动构建遇到问题，可以手动构建：

```bash
cd app
npm install
npm run build
cd src-tauri
cargo build --release
```

构建产物：
- Windows: `app/src-tauri/target/release/add-column-manager.exe`
- 安装包: `app/src-tauri/target/release/bundle/msi/AddColumnManager_0.1.0_x64_en-US.msi`

## 🔗 相关链接

- [Tauri 文档](https://tauri.app/v1/guides/)
- [Vue 3 文档](https://vuejs.org/)
- [Rust 文档](https://doc.rust-lang.org/)

---

**注意**：本工具专门用于处理包含 JSON5 格式（支持注释）的 `theme*.json` 文件，请确保文件格式正确。

