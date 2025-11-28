import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { invoke } from "@tauri-apps/api/tauri";
import { open, save, ask } from "@tauri-apps/api/dialog";
import { readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import type {
  BulkInsertResult,
  FileMapping,
  MappingInput
} from "@/types/mapping";

export interface ScanResult {
  files: FileMapping[];
}

export interface BackupResult {
  backupDir: string;
}

export interface BulkInsertPayload {
  entries: MappingInput[];
}

/**
 * 管理栏目映射数据的 Pinia Store。
 */
export const useMappingStore = defineStore("mapping-store", () => {
  const targetDir = ref("");
  const files = ref<FileMapping[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const lastBackupDir = ref<string | null>(null);
  const lastInsertReport = ref<BulkInsertResult | null>(null);

  const hasData = computed(() => files.value.length > 0);

  /**
   * 打开目录选择器并更新待处理目录。
   */
  const pickDirectory = async () => {
    const selected = await open({
      directory: true,
      multiple: false
    });
    if (typeof selected === "string") {
      targetDir.value = selected;
    }
  };

  /**
   * 调用后端扫描 theme*.json 文件。
   */
  const scanDirectory = async (manualPath?: string) => {
    const pathToUse = manualPath ?? targetDir.value;
    if (!pathToUse) {
      error.value = "请先选择包含 theme*.json 的目标目录。";
      return;
    }
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<ScanResult>("scan_theme_files", {
        targetDir: pathToUse
      });
      files.value = result.files;
      targetDir.value = pathToUse;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    } finally {
      loading.value = false;
    }
  };

  /**
   * 调用后端备份当前目录中的 theme*.json 文件。
   */
  /**
   * 批量写入新的栏目映射。
   */
  const bulkInsert = async (entries: MappingInput[]) => {
    if (!targetDir.value) {
      error.value = "尚未选择目录，无法写入映射。";
      return;
    }
    if (!entries.length) {
      error.value = "请至少输入一条映射。";
      return;
    }
    try {
      const result = await invoke<BulkInsertResult>("bulk_insert_mappings", {
        targetDir: targetDir.value,
        entries
      });
      lastInsertReport.value = result;
      // 更新备份路径（如果有备份）
      if (result.backupDir) {
        lastBackupDir.value = result.backupDir;
      }
      await scanDirectory(targetDir.value);
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    }
  };

  /**
   * 导出去重后的映射项（sExtOptions格式）。
   */
  const exportMappings = async () => {
    if (!hasData.value) {
      error.value = "没有可导出的数据，请先扫描文件。";
      return;
    }
    try {
      // 按本地ID去重，保留最新的映射
      const uniqueMappings = new Map<string, { localId: string; gwId?: string; rawValue: string }>();
      files.value.forEach((file) => {
        file.mappings.forEach((mapping) => {
          if (!uniqueMappings.has(mapping.localId)) {
            uniqueMappings.set(mapping.localId, {
              localId: mapping.localId,
              gwId: mapping.gwId,
              rawValue: mapping.rawValue
            });
          }
        });
      });

      // 构建sExtOptions格式的JSON
      const sExtOptions: Record<string, string> = {};
      uniqueMappings.forEach((mapping) => {
        sExtOptions[`portal_frag_${mapping.localId}`] = mapping.rawValue;
      });

      const exportData = {
        sExtOptions
      };

      const filePath = await save({
        filters: [
          {
            name: "JSON",
            extensions: ["json"]
          }
        ],
        defaultPath: "mappings_export.json"
      });

      if (filePath) {
        await writeTextFile(filePath, JSON.stringify(exportData, null, 2));
        error.value = null;
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    }
  };

  /**
   * 导入sExtOptions格式的JSON文件。
   */
  const importMappings = async () => {
    if (!targetDir.value) {
      error.value = "尚未选择目录，无法导入映射。";
      return;
    }
    try {
      const filePath = await open({
        filters: [
          {
            name: "JSON",
            extensions: ["json"]
          }
        ],
        multiple: false
      });

      if (typeof filePath === "string") {
        const content = await readTextFile(filePath);
        const data = JSON.parse(content);

        if (!data.sExtOptions || typeof data.sExtOptions !== "object") {
          error.value = "导入文件格式错误：缺少 sExtOptions 字段。";
          return;
        }

        // 提取portal_frag_*映射
        const entries: MappingInput[] = [];
        const rawMappings: Record<string, string> = {};
        for (const [key, value] of Object.entries(data.sExtOptions)) {
          if (key.startsWith("portal_frag_") && typeof value === "string") {
            const localId = key.replace("portal_frag_", "");
            // 从value中提取国网ID
            const gwIdMatch = value.match(/es_tabId=([^&|"]+)/);
            const gwId = gwIdMatch ? gwIdMatch[1] : "";
            if (localId && gwId) {
              entries.push({ localId, gwId });
              rawMappings[localId] = value;
            }
          }
        }

        if (entries.length === 0) {
          error.value = "导入文件中没有找到有效的映射项。";
          return;
        }

        // 检查现有映射中的重复项
        const existingLocalIds = new Set<string>();
        files.value.forEach((file) => {
          file.mappings.forEach((mapping) => {
            existingLocalIds.add(mapping.localId);
          });
        });

        const duplicateIds = entries.filter((entry) =>
          existingLocalIds.has(entry.localId)
        );

        // 显示确认对话框
        let confirmMessage = `准备导入 ${entries.length} 条映射。\n\n`;
        if (duplicateIds.length > 0) {
          confirmMessage += `⚠️ 发现 ${duplicateIds.length} 条重复的本地栏目ID，将替换现有映射：\n${duplicateIds.slice(0, 10).map((e) => e.localId).join("、")}${duplicateIds.length > 10 ? ` 等${duplicateIds.length}项` : ""}\n\n`;
        }
        confirmMessage += "导入前将自动创建备份。\n\n是否确认继续？";

        const confirmed = await ask(confirmMessage, {
          title: "确认导入映射",
          type: "warning",
          okLabel: "确认导入",
          cancelLabel: "取消"
        });

        if (!confirmed) {
          return;
        }

        // 先备份
        loading.value = true;
        error.value = null;
        try {
          const backup = await invoke<BackupResult>("backup_theme_files", {
            targetDir: targetDir.value
          });
          lastBackupDir.value = backup.backupDir;

          // 调用导入命令（替换模式）
          const result = await invoke<BulkInsertResult>("import_mappings", {
            targetDir: targetDir.value,
            mappings: rawMappings
          });

          lastInsertReport.value = result;
          await scanDirectory(targetDir.value);
        } catch (err) {
          error.value = err instanceof Error ? err.message : String(err);
        } finally {
          loading.value = false;
        }
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    }
  };

  /**
   * 删除单个文件中的指定映射项。
   */
  const deleteMapping = async (filePath: string, localId: string) => {
    if (!targetDir.value) {
      error.value = "尚未选择目录，无法删除映射。";
      return;
    }
    try {
      const backupDir = await invoke<string | null>("delete_mapping", {
        filePath,
        localId
      });
      // 更新备份路径（如果有备份）
      if (backupDir) {
        lastBackupDir.value = backupDir;
      }
      // 重新扫描以更新显示
      await scanDirectory(targetDir.value);
      error.value = null;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
    }
  };

  /**
   * 批量删除映射项。
   */
  const batchDeleteMappings = async (requests: Array<{ filePath: string; localId: string }>) => {
    if (!targetDir.value) {
      error.value = "尚未选择目录，无法删除映射。";
      return;
    }
    if (requests.length === 0) {
      error.value = "请至少选择一条映射进行删除。";
      return;
    }
    try {
      const result = await invoke<BulkInsertResult>("batch_delete_mappings", {
        requests
      });
      // 更新备份路径和报告（如果有备份）
      if (result.backupDir) {
        lastBackupDir.value = result.backupDir;
      }
      lastInsertReport.value = result;
      // 重新扫描以更新显示
      await scanDirectory(targetDir.value);
      error.value = null;
      return result;
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err);
      throw err;
    }
  };

  return {
    targetDir,
    files,
    loading,
    error,
    hasData,
    lastBackupDir,
    lastInsertReport,
    pickDirectory,
    scanDirectory,
    bulkInsert,
    exportMappings,
    importMappings,
    deleteMapping,
    batchDeleteMappings
  };
});

