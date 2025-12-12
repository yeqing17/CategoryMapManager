# 栏目映射管理工具

一个基于 Tauri + Vue 3 的桌面应用程序，用于批量管理和维护 `theme*.json` 文件中的本地/国网栏目 ID 映射关系。

## ✨ 功能特性

- 🔍 **自动扫描识别**：自动扫描目录下所有 `theme*.json` 文件，识别 `sExtOptions` 段落中的 `portal_frag_*` 映射关系
- 📊 **映射统计**：按本地栏目 ID 去重统计，清晰展示文件数量和映射数量
- ⚠️ **重复检测**：自动检测本地栏目 ID 和国网栏目 ID 的重复情况，并给出明确提示
- 📝 **批量新增**：支持批量添加映射关系，一键同步到所有 theme 文件
- 🗑️ **删除功能**：支持单个和批量删除映射关系，操作前自动备份
- 🔄 **对比视图**：支持跨文件对比映射关系，自动折叠相同项，高亮显示差异
- 📋 **排序功能**：按本地栏目 ID 数值大小自动排序，方便查看
- 💾 **自动备份**：所有文件修改操作前自动创建备份，确保数据安全可回滚
- 📝 **操作日志**：每次批量操作后自动生成详细的操作日志，记录操作内容、备份路径和版本变化
- 🔢 **版本号管理**：支持自动递增 JSON 文件中的版本号，可通过界面开关控制（默认开启）
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
cd CategoryMapManager/app

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
cd CategoryMapManager

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
# app/src-tauri/target/release/CategoryMapManager.exe
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

**版本号管理**：在目标目录区域可以看到「自动递增版本号」配置选项，默认开启。当启用时，每次修改 theme\*.json 文件都会自动将文件中的 `version` 字段值加 1。

### 2. 扫描文件

点击「扫描」按钮，程序会自动识别所有 `theme*.json` 文件中的 `sExtOptions` 段落，提取 `portal_frag_*` 映射关系。

### 3. 查看结果

扫描后可在下方查看每个文件的映射详情，包括：

- 本地栏目 ID
- 国网栏目 ID
- 状态（正常/本地 ID 重复/国网 ID 重复）

### 4. 批量新增

在批量新增区域填写本地栏目 ID 和国网栏目 ID，点击「写入所有文件」即可同步到所有 theme 文件。

### 5. 删除映射

- **单个删除**：在映射表格中点击对应行的删除按钮，可删除单个文件的映射关系
- **批量删除**：
  - 在「分别显示」模式下，可勾选多个映射项进行批量删除
  - 在「对比显示」模式下，可勾选某个本地栏目 ID 的所有文件映射进行批量删除
- 所有删除操作前都会自动创建备份

### 6. 对比视图

点击「对比显示」按钮，可以：

- 查看所有文件中相同本地栏目 ID 的映射关系
- 自动折叠所有文件中映射关系一致的项
- 高亮显示映射关系不一致的项，方便快速定位差异

### 7. 导出/导入

- **导出**：点击「导出」按钮，将去重后的映射项导出为完整的 sExtOptions JSON 格式
- **导入**：点击「导入」按钮，选择 JSON 文件导入，程序会：
  - 自动检测重复项并提示
  - 显示确认对话框
  - 自动创建备份
  - 替换现有映射

### 8. 操作日志

每次批量操作（新增、删除、导入）后，程序会在处理文件的目录下自动生成操作日志文件（`operation_YYYYMMDD-HHMMSS.log`），记录：

- 操作类型和时间
- 备份路径
- 版本变化信息（显示每个文件的版本号变化：旧版本 → 新版本）
- 新增/删除的映射详情（本地栏目 ID 和国网栏目 ID）
- 成功处理的文件列表
- 跳过的文件及原因

### 9. 注意事项

- **本地栏目 ID**：必须唯一，重复时会标记为错误
- **国网栏目 ID**：可以重复，但会给出警告提示
- **自动备份**：所有文件修改操作前都会自动创建备份，备份路径显示在操作结果中
- **版本号管理**：启用「自动递增版本号」后，每次修改文件都会自动将 version 字段值加 1
- **操作日志**：每次批量操作后都会生成详细的操作日志，包含版本变化记录，方便追溯和审计

## 📁 项目结构

```text
CategoryMapManager/
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
2. **去重检测**：统计本地 ID 和国网 ID 的出现次数，自动标记重复状态
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

yeiqng

## 项目维护者

## 📦 发布与分发

### GitHub Releases

项目配置了 GitHub Actions 自动构建工作流，当推送 tag（格式：`v*`）时会自动构建并上传构建产物。

**推荐使用安装包**：

- **MSI 安装包**（推荐）：`.msi` 文件，Windows 标准安装程序，更安全可靠
- **NSIS 安装包**：`.exe` 安装程序，另一种安装方式
- **便携版 EXE**：直接运行的 `.exe` 文件，无需安装

### ⚠️ Windows SmartScreen 警告处理

由于应用未进行代码签名，Windows SmartScreen 可能会显示安全警告。这是正常现象，请按以下步骤处理：

#### 方法一：使用安装包（推荐）

**优先使用 MSI 或 NSIS 安装包**，安装包通常更容易被 Windows 信任，较少出现 SmartScreen 警告。

#### 方法二：解除便携版 EXE 锁定

如果使用便携版 `.exe` 文件遇到 SmartScreen 警告：

1. **下载文件后**，右键点击 `.exe` 文件
2. 选择 **"属性"**
3. 在属性窗口底部，如果看到 **"此文件来自其他计算机，可能被阻止以帮助保护该计算机"**
4. 勾选 **"解除锁定"** 复选框
5. 点击 **"确定"**
6. 现在可以正常双击运行程序了

#### 方法三：通过 Windows Defender 添加例外

1. 打开 **Windows 安全中心**
2. 进入 **"病毒和威胁防护"**
3. 点击 **"病毒和威胁防护设置"** → **"管理设置"**
4. 在 **"排除项"** 中添加程序所在文件夹

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

- Windows: `app/src-tauri/target/release/CategoryMapManager.exe`
- 安装包: `app/src-tauri/target/release/bundle/msi/CategoryMapManager_*.msi`

## 🔗 相关链接

- [Tauri 文档](https://tauri.app/v1/guides/)
- [Vue 3 文档](https://vuejs.org/)
- [Rust 文档](https://doc.rust-lang.org/)

---

**注意**：本工具专门用于处理包含 JSON5 格式（支持注释）的 `theme*.json` 文件，请确保文件格式正确。
