<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useMappingStore } from "@/stores/mappingStore";
import { ask } from "@tauri-apps/api/dialog";
import type { FileMapping, MappingEntry } from "@/types/mapping";

const props = defineProps<{
  files: FileMapping[];
}>();

const store = useMappingStore();

// 使用独立的响应式 Map 来存储折叠状态
const collapseStates = ref(new Map<string, boolean>());

// 当文件列表变化时，清理不存在的 localId 的折叠状态
watch(() => props.files, () => {
  const existingIds = new Set<string>();
  props.files.forEach((file) => {
    file.mappings.forEach((mapping) => {
      existingIds.add(mapping.localId);
    });
  });
  
  // 清理不存在的 ID 的状态
  const statesToRemove: string[] = [];
  collapseStates.value.forEach((_, localId) => {
    if (!existingIds.has(localId)) {
      statesToRemove.push(localId);
    }
  });
  statesToRemove.forEach((id) => collapseStates.value.delete(id));
}, { deep: true });

/**
 * 计算列表是否为空。
 */
const isEmpty = computed(() => props.files.length === 0);

/**
 * 计算文件映射的统计信息。
 */
const getFileStats = (file: FileMapping) => {
  const duplicateLocal = file.mappings.filter(
    (m) => m.status === "duplicate_local"
  ).length;
  const duplicateGw = file.mappings.filter(
    (m) => m.status === "duplicate_gw"
  ).length;
  return { duplicateLocal, duplicateGw };
};

/**
 * 按本地ID数值大小排序（仅用于显示，不修改原文件）。
 */
const sortedMappings = (mappings: MappingEntry[]) => {
  return [...mappings].sort((a, b) => {
    const numA = parseInt(a.localId, 10) || 0;
    const numB = parseInt(b.localId, 10) || 0;
    return numA - numB;
  });
};

/**
 * 跨文件对比：收集所有文件的映射，按本地ID分组。
 */
interface CrossFileMapping {
  localId: string;
  entries: Array<{
    filePath: string;
    mapping: MappingEntry;
  }>;
  isConsistent: boolean; // 所有文件中的国网ID是否一致
}

const crossFileMappings = computed(() => {
  const map = new Map<string, CrossFileMapping>();
  
  // 收集所有映射
  props.files.forEach((file) => {
    if (file && file.mappings && Array.isArray(file.mappings)) {
      file.mappings.forEach((mapping) => {
        if (mapping && mapping.localId) {
          if (!map.has(mapping.localId)) {
            map.set(mapping.localId, {
              localId: mapping.localId,
              entries: [],
              isConsistent: true
            });
          }
          const item = map.get(mapping.localId)!;
          item.entries.push({
            filePath: file.filePath,
            mapping
          });
        }
      });
    }
  });
  
  // 检查一致性并初始化折叠状态
  map.forEach((item) => {
    if (item.entries.length > 1) {
      const firstGwId = item.entries[0].mapping.gwId;
      item.isConsistent = item.entries.every(
        (e) => e.mapping.gwId === firstGwId
      );
      // 初始化折叠状态：一致的默认折叠，不一致的默认展开
      if (!collapseStates.value.has(item.localId)) {
        collapseStates.value.set(item.localId, item.isConsistent);
      }
    } else {
      // 只有一个文件，默认折叠
      if (!collapseStates.value.has(item.localId)) {
        collapseStates.value.set(item.localId, true);
      }
    }
  });
  
  // 转换为数组并按本地ID数值排序
  return Array.from(map.values()).sort((a, b) => {
    const numA = parseInt(a.localId, 10) || 0;
    const numB = parseInt(b.localId, 10) || 0;
    return numA - numB;
  });
});

/**
 * 获取折叠状态。
 */
const isCollapsed = (localId: string): boolean => {
  return collapseStates.value.get(localId) ?? true;
};

/**
 * 切换折叠状态。
 */
const toggleCollapse = (localId: string) => {
  const current = collapseStates.value.get(localId) ?? true;
  collapseStates.value.set(localId, !current);
};

/**
 * 删除映射项。
 */
const handleDelete = async (filePath: string, localId: string) => {
  const confirmed = await ask(
    `确定要删除文件 "${filePath}" 中的本地栏目ID "${localId}" 吗？`,
    {
      title: "确认删除",
      type: "warning",
      okLabel: "删除",
      cancelLabel: "取消"
    }
  );
  
  if (confirmed) {
    await store.deleteMapping(filePath, localId);
  }
};

/**
 * 显示模式：separate（分别显示）或 compare（对比显示）。
 */
const displayMode = ref<"separate" | "compare">("separate");

/**
 * 批量选择状态。
 */
interface SelectedMapping {
  filePath: string;
  localId: string;
}

const selectedMappings = ref<Set<string>>(new Set());

/**
 * 生成选择键。
 */
const getSelectionKey = (filePath: string, localId: string): string => {
  return `${filePath}::${localId}`;
};

/**
 * 切换选择状态。
 */
const toggleSelection = (filePath: string, localId: string) => {
  const key = getSelectionKey(filePath, localId);
  // 创建新的 Set 以确保响应式更新
  const newSet = new Set(selectedMappings.value);
  if (newSet.has(key)) {
    newSet.delete(key);
  } else {
    newSet.add(key);
  }
  selectedMappings.value = newSet;
};

/**
 * 处理对比模式下单个本地ID的选择切换。
 */
const handleCompareItemToggle = (item: CrossFileMapping) => {
  // 检查当前是否所有条目都已选中
  const allSelected = item.entries.length > 0 && 
    item.entries.every(entry => isSelected(entry.filePath, item.localId));
  const targetState = !allSelected;
  
  // 创建新的 Set 以确保响应式更新
  const newSet = new Set(selectedMappings.value);
  
  // 遍历所有条目，添加或删除
  item.entries.forEach(entry => {
    const key = getSelectionKey(entry.filePath, item.localId);
    if (targetState) {
      newSet.add(key);
    } else {
      newSet.delete(key);
    }
  });
  
  // 强制更新响应式状态
  selectedMappings.value = new Set(newSet);
};

/**
 * 检查是否已选择。
 */
const isSelected = (filePath: string, localId: string): boolean => {
  return selectedMappings.value.has(getSelectionKey(filePath, localId));
};

/**
 * 全选/取消全选。
 */
const toggleSelectAll = (mode: "separate" | "compare") => {
  if (mode === "separate") {
    const allKeys = new Set<string>();
    props.files.forEach((file) => {
      file.mappings.forEach((mapping) => {
        allKeys.add(getSelectionKey(file.filePath, mapping.localId));
      });
    });
    
    // 直接检查是否已经全选
    const isAllSelected = allKeys.size > 0 && 
      selectedMappings.value.size === allKeys.size &&
      Array.from(allKeys).every(key => selectedMappings.value.has(key));
    
    if (isAllSelected) {
      // 取消全选
      selectedMappings.value = new Set<string>();
    } else {
      // 全选：创建新的 Set 包含所有键
      selectedMappings.value = new Set(allKeys);
    }
  } else {
    const allKeys = new Set<string>();
    crossFileMappings.value.forEach((item) => {
      item.entries.forEach((entry) => {
        allKeys.add(getSelectionKey(entry.filePath, item.localId));
      });
    });
    
    // 直接检查是否已经全选
    const isAllSelected = allKeys.size > 0 && 
      selectedMappings.value.size === allKeys.size &&
      Array.from(allKeys).every(key => selectedMappings.value.has(key));
    
    if (isAllSelected) {
      // 取消全选
      selectedMappings.value = new Set<string>();
    } else {
      // 全选：创建新的 Set 包含所有键
      selectedMappings.value = new Set(allKeys);
    }
  }
};

/**
 * 获取已选择的数量。
 */
const selectedCount = computed(() => selectedMappings.value.size);

/**
 * 计算总映射数量（用于全选 - 分别显示模式）。
 */
const totalMappingsCount = computed(() => {
  let count = 0;
  props.files.forEach((file) => {
    count += file.mappings.length;
  });
  return count;
});

/**
 * 检查是否全选（分别显示模式）。
 */
const isAllSelectedSeparate = computed(() => {
  if (totalMappingsCount.value === 0) return false;
  const allKeys = new Set<string>();
  props.files.forEach((file) => {
    file.mappings.forEach((mapping) => {
      allKeys.add(getSelectionKey(file.filePath, mapping.localId));
    });
  });
  return allKeys.size > 0 && 
    selectedMappings.value.size === allKeys.size &&
    Array.from(allKeys).every(key => selectedMappings.value.has(key));
});

/**
 * 检查是否全选（对比显示模式）。
 */
const isAllSelectedCompare = computed(() => {
  const allKeys = new Set<string>();
  crossFileMappings.value.forEach((item) => {
    item.entries.forEach((entry) => {
      allKeys.add(getSelectionKey(entry.filePath, item.localId));
    });
  });
  if (allKeys.size === 0) return false;
  return selectedMappings.value.size === allKeys.size &&
    Array.from(allKeys).every(key => selectedMappings.value.has(key));
});

/**
 * 计算总映射数量（用于全选 - 对比显示模式）。
 */
const totalCompareMappingsCount = computed(() => {
  let count = 0;
  crossFileMappings.value.forEach((item) => {
    count += item.entries.length;
  });
  return count;
});

/**
 * 批量删除。
 */
const handleBatchDelete = async () => {
  if (selectedMappings.value.size === 0) {
    return;
  }

  const requests: Array<{ filePath: string; localId: string }> = [];
  selectedMappings.value.forEach((key) => {
    const [filePath, localId] = key.split("::");
    requests.push({ filePath, localId });
  });

  const confirmed = await ask(
    `确定要删除选中的 ${requests.length} 条映射吗？\n\n此操作不可撤销。`,
    {
      title: "确认批量删除",
      type: "warning",
      okLabel: "删除",
      cancelLabel: "取消"
    }
  );

  if (confirmed) {
    try {
      await store.batchDeleteMappings(requests);
      // 清空选择（创建新的 Set 以确保响应式更新）
      selectedMappings.value = new Set();
    } catch (err) {
      // 错误已在 store 中处理
    }
  }
};

const handleClearSelection = () => {
  selectedMappings.value = new Set();
};
</script>

<template>
  <section class="table-wrapper">
    <div v-if="isEmpty" class="empty">尚未扫描到任何 theme*.json 文件。</div>
    <div v-else>
      <!-- 显示模式切换和批量操作 -->
      <div class="toolbar">
        <div class="mode-switcher">
          <button
            @click="displayMode = 'separate'"
            :class="{ active: displayMode === 'separate' }"
            class="mode-btn"
          >
            📋 分别显示
          </button>
          <button
            @click="displayMode = 'compare'"
            :class="{ active: displayMode === 'compare' }"
            class="mode-btn"
          >
            🔍 对比显示
          </button>
        </div>
        <div class="batch-actions" v-if="selectedCount > 0">
          <span class="selected-count">已选择 {{ selectedCount }} 项</span>
          <button @click="handleBatchDelete" class="batch-delete-btn">
            🗑️ 批量删除
          </button>
          <button @click="handleClearSelection" class="clear-selection-btn">
            清除选择
          </button>
        </div>
      </div>

      <!-- 分别显示模式 -->
      <div v-if="displayMode === 'separate'" class="file-grid">
        <article v-for="file in files" :key="file.filePath" class="file-card">
          <header>
            <h3>{{ file.filePath }}</h3>
            <div class="stats">
              <span>映射数量：{{ file.mappings.length }}</span>
              <span
                v-if="getFileStats(file).duplicateLocal > 0"
                class="stat-error"
              >
                ⚠️ 本地ID重复：{{ getFileStats(file).duplicateLocal }}
              </span>
              <span
                v-if="getFileStats(file).duplicateGw > 0"
                class="stat-warning"
              >
                ⚠️ 国网ID重复：{{ getFileStats(file).duplicateGw }}
              </span>
            </div>
          </header>
          <table>
            <thead>
              <tr>
                <th style="width: 50px; min-width: 50px; text-align: center; padding: 8px 4px;">
                  <input
                    type="checkbox"
                    @click.stop
                    @change="(e) => {
                      e.stopPropagation();
                      toggleSelectAll('separate');
                    }"
                    :checked="isAllSelectedSeparate"
                    :indeterminate="selectedCount > 0 && !isAllSelectedSeparate && selectedCount < totalMappingsCount"
                    class="checkbox"
                    title="全选/取消全选"
                  />
                </th>
                <th>本地栏目 ID</th>
                <th>国网栏目 ID</th>
                <th>状态</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="mapping in sortedMappings(file.mappings)"
                :key="mapping.localId"
                :class="{
                  'row-error': mapping.status === 'duplicate_local',
                  'row-warning': mapping.status === 'duplicate_gw',
                  'row-selected': isSelected(file.filePath, mapping.localId)
                }"
              >
                <td style="text-align: center; width: 50px; min-width: 50px; padding: 8px 4px;">
                  <input
                    type="checkbox"
                    :checked="isSelected(file.filePath, mapping.localId)"
                    @change="toggleSelection(file.filePath, mapping.localId)"
                    class="checkbox"
                  />
                </td>
                <td>{{ mapping.localId }}</td>
                <td>{{ mapping.gwId ?? "未解析" }}</td>
                <td>
                  <span
                    v-if="mapping.status === 'duplicate_local'"
                    class="badge error"
                    title="本地栏目ID重复，必须唯一"
                  >
                    ⚠️ 本地ID重复
                  </span>
                  <span
                    v-else-if="mapping.status === 'duplicate_gw'"
                    class="badge warning"
                    title="国网栏目ID重复，请注意"
                  >
                    ⚠️ 国网ID重复
                  </span>
                  <span v-else-if="mapping.sameId" class="badge success">
                    ✓ 正常（ID一致）
                  </span>
                  <span v-else class="badge normal">
                    ✓ 正常
                  </span>
                </td>
                <td>
                  <button
                    @click="handleDelete(file.filePath, mapping.localId)"
                    class="delete-btn"
                    title="删除此映射"
                  >
                    🗑️
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </article>
      </div>

      <!-- 对比显示模式 -->
      <div v-else class="compare-view">
        <div class="compare-header">
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
            <h3>跨文件对比</h3>
            <div style="display: flex; align-items: center; gap: 12px;">
              <input
                type="checkbox"
                @click.stop
                @change="(e) => {
                  e.stopPropagation();
                  toggleSelectAll('compare');
                }"
                :checked="isAllSelectedCompare"
                :indeterminate="selectedCount > 0 && !isAllSelectedCompare && selectedCount < totalCompareMappingsCount"
                class="checkbox"
                title="全选/取消全选"
              />
              <span style="font-size: 12px; color: #64748b;">全选</span>
            </div>
          </div>
          <p class="hint">
            <span class="hint-item">
              <span class="hint-dot consistent"></span>
              一致（已折叠）
            </span>
            <span class="hint-item">
              <span class="hint-dot inconsistent"></span>
              不一致（已展开）
            </span>
            <span class="hint-item" style="margin-left: 16px;">
              💡 点击复选框可选择该本地栏目ID在所有文件中的映射
            </span>
          </p>
        </div>
        <div class="compare-list">
          <div
            v-for="item in crossFileMappings"
            :key="item.localId"
            :class="{
              'compare-item': true,
              'consistent': item.isConsistent,
              'inconsistent': !item.isConsistent,
              'collapsed': isCollapsed(item.localId) && item.isConsistent
            }"
          >
            <div
              class="compare-header-row"
              :class="{ 'clickable': item.entries.length > 1 }"
            >
              <div class="local-id">
                <input
                  type="checkbox"
                  :checked="item.entries.length > 0 && item.entries.every(e => isSelected(e.filePath, item.localId))"
                  @click.stop
                  @change="handleCompareItemToggle(item)"
                  class="checkbox checkbox-main"
                  :title="`选择本地栏目ID ${item.localId} 在所有 ${item.entries.length} 个文件中的映射`"
                />
                <span 
                  class="id-value"
                  @click="item.entries.length > 1 && toggleCollapse(item.localId)"
                  :style="{ cursor: item.entries.length > 1 ? 'pointer' : 'default' }"
                >
                  {{ item.localId }}
                </span>
                <span
                  v-if="item.entries.length > 1"
                  class="collapse-icon"
                  @click.stop="toggleCollapse(item.localId)"
                >
                  {{ isCollapsed(item.localId) ? "▶" : "▼" }}
                </span>
              </div>
              <div class="gw-id-summary">
                <span v-if="item.isConsistent && item.entries.length > 0">
                  {{ item.entries[0].mapping.gwId ?? "未解析" }}
                  <span class="file-count">({{ item.entries.length }}个文件)</span>
                </span>
                <span v-else class="inconsistent-label">
                  ⚠️ 不一致
                </span>
              </div>
            </div>
            <div
              v-if="!isCollapsed(item.localId) || !item.isConsistent"
              class="compare-details"
            >
              <div
                v-for="entry in item.entries"
                :key="entry.filePath"
                :class="{
                  'detail-row': true,
                  'row-selected': isSelected(entry.filePath, item.localId)
                }"
              >
                <input
                  type="checkbox"
                  :checked="isSelected(entry.filePath, item.localId)"
                  @change="() => toggleSelection(entry.filePath, item.localId)"
                  class="checkbox"
                  style="margin-right: 8px;"
                />
                <span class="file-name">{{ entry.filePath }}</span>
                <span class="gw-id">{{ entry.mapping.gwId ?? "未解析" }}</span>
                <button
                  @click="handleDelete(entry.filePath, item.localId)"
                  class="delete-btn-small"
                  title="删除此映射"
                >
                  🗑️
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.table-wrapper {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.empty {
  padding: 48px;
  border-radius: 16px;
  text-align: center;
  background: #fff;
  color: #475467;
  border: 1px dashed rgba(15, 23, 42, 0.12);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding: 12px;
  background: #f8fafc;
  border-radius: 12px;
  flex-wrap: wrap;
  gap: 12px;
}

.mode-switcher {
  display: flex;
  gap: 12px;
}

.batch-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.selected-count {
  font-size: 14px;
  color: #475569;
  font-weight: 500;
}

.batch-delete-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 8px;
  background: linear-gradient(135deg, #dc2626, #b91c1c);
  color: #fff;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
}

.batch-delete-btn:hover {
  background: linear-gradient(135deg, #b91c1c, #991b1b);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(220, 38, 38, 0.3);
}

.clear-selection-btn {
  padding: 8px 16px;
  border: 1px solid rgba(15, 23, 42, 0.1);
  border-radius: 8px;
  background: #fff;
  color: #64748b;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.clear-selection-btn:hover {
  background: #f1f5f9;
  border-color: rgba(37, 99, 235, 0.3);
  color: #475569;
}

.mode-btn {
  padding: 8px 16px;
  border: 1px solid rgba(15, 23, 42, 0.1);
  border-radius: 8px;
  background: #fff;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
}

.mode-btn:hover {
  background: #f1f5f9;
  border-color: rgba(37, 99, 235, 0.3);
}

.mode-btn.active {
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
  color: #fff;
  border-color: #2563eb;
}

.file-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(420px, 1fr));
  gap: 20px;
}

.file-card {
  background: #fefefe;
  border-radius: 24px;
  padding: 20px;
  border: 1px solid rgba(15, 23, 42, 0.08);
}

.file-card header {
  margin-bottom: 12px;
}

.file-card header h3 {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
  word-break: break-all;
}

.stats {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  margin-top: 4px;
  font-size: 12px;
  color: #64748b;
}

.stat-error {
  color: #dc2626;
  font-weight: 600;
}

.stat-warning {
  color: #d97706;
  font-weight: 600;
}

table {
  width: 100%;
  border-collapse: collapse;
}

th,
td {
  text-align: left;
  padding: 8px;
  border-bottom: 1px solid rgba(15, 23, 42, 0.06);
  font-size: 13px;
}

th:first-child,
td:first-child {
  width: 50px !important;
  min-width: 50px !important;
  text-align: center !important;
  padding: 8px 4px !important;
  position: relative;
  z-index: 1;
}

th {
  font-weight: 600;
  color: #475569;
  background: #f8fafc;
}

.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 999px;
  background: #e5edff;
  color: #1f4fe0;
  font-size: 12px;
}

.badge.success {
  background: #d1fae5;
  color: #047857;
}

.badge.normal {
  background: #e5edff;
  color: #1f4fe0;
}

.badge.warning {
  background: #fef3c7;
  color: #92400e;
}

.badge.error {
  background: #fee2e2;
  color: #991b1b;
}

.row-error {
  background: rgba(239, 68, 68, 0.05);
}

.row-warning {
  background: rgba(245, 158, 11, 0.05);
}

.row-selected {
  background: rgba(37, 99, 235, 0.1) !important;
}

.checkbox {
  cursor: pointer !important;
  width: 20px !important;
  height: 20px !important;
  accent-color: #2563eb !important;
  display: inline-block !important;
  margin: 0 auto !important;
  flex-shrink: 0;
  opacity: 1 !important;
  visibility: visible !important;
  position: relative !important;
  z-index: 10;
  border: 2px solid #cbd5e1 !important;
  border-radius: 4px !important;
  background-color: #fff !important;
  -webkit-appearance: checkbox !important;
  -moz-appearance: checkbox !important;
  appearance: checkbox !important;
}

.checkbox:checked {
  background-color: #2563eb !important;
  border-color: #2563eb !important;
}

.checkbox-main {
  width: 20px;
  height: 20px;
  margin-right: 12px;
  flex-shrink: 0;
  display: block;
}

.delete-btn,
.delete-btn-small {
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
  font-size: 14px;
}

.delete-btn:hover,
.delete-btn-small:hover {
  background: rgba(239, 68, 68, 0.1);
  transform: scale(1.1);
}

.delete-btn-small {
  font-size: 12px;
  padding: 2px 6px;
}

/* 对比显示样式 */
.compare-view {
  background: #fff;
  border-radius: 16px;
  padding: 20px;
  border: 1px solid rgba(15, 23, 42, 0.08);
}

.compare-header {
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(15, 23, 42, 0.08);
}

.compare-header h3 {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: #0f172a;
}

.hint {
  display: flex;
  gap: 16px;
  margin: 0;
  font-size: 12px;
  color: #64748b;
}

.hint-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.hint-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  display: inline-block;
}

.hint-dot.consistent {
  background: #10b981;
}

.hint-dot.inconsistent {
  background: #f59e0b;
}

.compare-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.compare-item {
  border: 1px solid rgba(15, 23, 42, 0.08);
  border-radius: 8px;
  padding: 12px;
  transition: all 0.2s;
}

.compare-item.consistent {
  background: rgba(16, 185, 129, 0.05);
  border-color: rgba(16, 185, 129, 0.2);
}

.compare-item.inconsistent {
  background: rgba(245, 158, 11, 0.1);
  border-color: rgba(245, 158, 11, 0.3);
}

.compare-header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.compare-header-row.clickable {
  cursor: pointer;
}

.compare-header-row.clickable:hover {
  opacity: 0.8;
}

.local-id {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: #0f172a;
  min-width: 100px;
  flex: 1;
}

.local-id .id-value {
  user-select: none;
}

.id-value {
  font-size: 14px;
}

.collapse-icon {
  font-size: 10px;
  color: #64748b;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  transition: background 0.2s;
}

.collapse-icon:hover {
  background: rgba(15, 23, 42, 0.1);
}

.gw-id-summary {
  flex: 1;
  text-align: right;
  font-size: 13px;
  color: #475569;
}

.file-count {
  font-size: 11px;
  color: #94a3b8;
  margin-left: 4px;
}

.inconsistent-label {
  color: #d97706;
  font-weight: 600;
}

.compare-details {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid rgba(15, 23, 42, 0.08);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  background: rgba(15, 23, 42, 0.02);
  border-radius: 6px;
}

.file-name {
  flex: 1;
  font-size: 12px;
  color: #64748b;
  word-break: break-all;
}

.gw-id {
  min-width: 120px;
  text-align: right;
  font-size: 13px;
  color: #475569;
  font-weight: 500;
}

.compare-item.collapsed .compare-details {
  display: none;
}
</style>
