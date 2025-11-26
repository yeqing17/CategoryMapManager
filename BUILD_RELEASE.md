# 构建和发布说明

## 📦 构建产物类型

Tauri 在 Windows 上可以生成以下类型的构建产物：

### 1. **MSI 安装程序** (推荐)
- 位置：`app/src-tauri/target/release/bundle/msi/*.msi`
- 特点：Windows 标准安装程序格式
- 用途：适合正式发布，提供标准的安装/卸载体验

### 2. **NSIS 安装程序**
- 位置：`app/src-tauri/target/release/bundle/nsis/*.exe`
- 特点：另一种 Windows 安装程序格式
- 用途：可自定义安装界面

### 3. **独立可执行文件**
- 位置：`app/src-tauri/target/release/AddColumnManager.exe`
- 特点：便携版，无需安装
- 用途：适合快速测试或便携使用

## 🔧 配置说明

### Tauri 配置 (`app/src-tauri/tauri.conf.json`)

```json
"tauri": {
  "bundle": {
    "active": true,           // 启用 bundle 生成
    "targets": "all",         // 生成所有类型的安装程序
    "identifier": "com.addcolumn.manager",  // 应用标识符
    "icon": [...],            // 图标文件列表
    "windows": {
      "certificateThumbprint": null,  // 代码签名证书（可选）
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

**关键配置项：**
- `active: true` - 必须启用才能生成安装程序
- `targets: "all"` - 生成所有支持的安装程序类型
- `identifier` - 应用的唯一标识符

### GitHub Actions 工作流 (`.github/workflows/build.yml`)

工作流包含以下关键步骤：

1. **构建应用** - 使用 `npm run tauri:build`
2. **列出构建产物** - 调试步骤，显示生成的文件
3. **上传 Artifacts** - 临时存储构建产物（30天）
4. **创建 Release** - 自动创建 GitHub Release 并上传文件

**Release 创建条件：**
- 仅在推送 tag（格式：`v*`）时触发
- 自动从 tag 名称提取版本号
- 自动上传所有找到的构建产物

## 🚀 使用方法

### 本地构建

```bash
cd app
npm run tauri:build
```

构建产物会生成在：
- `app/src-tauri/target/release/` - 可执行文件
- `app/src-tauri/target/release/bundle/` - 安装程序

### 自动发布流程

1. **创建并推送 tag：**
   ```bash
   git tag v0.1.2
   git push origin v0.1.2
   ```

2. **GitHub Actions 自动执行：**
   - 构建应用
   - 生成所有安装程序
   - 创建 GitHub Release
   - 上传构建产物到 Release

3. **在 GitHub 查看：**
   - 访问：`https://github.com/yeqing17/CategoryMapManager/releases`
   - 查看最新 Release
   - 下载构建产物

## 📝 常见问题

### Q: 为什么没有生成 MSI 文件？

**A:** 可能的原因：
1. `bundle.active` 未设置为 `true`
2. Windows 上缺少 WiX Toolset（Tauri 会自动下载，但可能需要时间）
3. 构建过程中出现错误（查看构建日志）

### Q: 如何只生成特定类型的安装程序？

**A:** 修改 `tauri.conf.json` 中的 `targets`：
```json
"targets": "msi"  // 只生成 MSI
// 或
"targets": ["msi", "nsis"]  // 生成 MSI 和 NSIS
```

### Q: Release 中没有文件？

**A:** 检查：
1. 构建是否成功完成
2. 文件路径是否正确
3. 查看 "Prepare release files" 步骤的输出

### Q: 如何添加代码签名？

**A:** 在 `tauri.conf.json` 中配置：
```json
"windows": {
  "certificateThumbprint": "你的证书指纹",
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com"
}
```

## 🔍 调试技巧

1. **查看构建日志：**
   - GitHub Actions 中的 "List build outputs" 步骤
   - 显示所有生成的文件和大小

2. **本地测试：**
   ```bash
   # 查看生成的文件
   ls app/src-tauri/target/release/bundle/
   ```

3. **检查配置：**
   - 确保 `bundle.active: true`
   - 确保图标文件存在
   - 检查 `identifier` 格式是否正确

## 📚 参考资源

- [Tauri Bundle 文档](https://tauri.app/v1/guides/building/)
- [GitHub Actions Release 文档](https://docs.github.com/en/actions)
- [WiX Toolset 文档](https://wixtoolset.org/)

