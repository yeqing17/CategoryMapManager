<script setup lang="ts">
import { computed } from "vue";
import { open as openExternal } from "@tauri-apps/api/shell";
import { useMappingStore } from "@/stores/mappingStore";

const store = useMappingStore();

/**
 * 统计当前扫描到的文件与映射数量。
 */
const totalFiles = computed(() => store.files.length);
const totalMappings = computed(() => {
  // 按本地栏目ID去重统计
  const uniqueLocalIds = new Set<string>();
  store.files.forEach((file) => {
    file.mappings.forEach((mapping) => {
      uniqueLocalIds.add(mapping.localId);
    });
  });
  return uniqueLocalIds.size;
});
</script>

<template>
  <section class="picker">
    <div class="header-row">
      <h2>目标目录</h2>
      <div class="stats" v-if="totalFiles > 0">
        <div class="stat-item">
          <span class="stat-label">文件</span>
          <strong class="stat-value">{{ totalFiles }}</strong>
        </div>
        <div class="stat-item">
          <span class="stat-label">映射</span>
          <strong class="stat-value">{{ totalMappings }}</strong>
        </div>
      </div>
    </div>
    <div class="input-row">
      <div class="input-wrapper">
        <input
          id="target-input"
          class="path-input"
          type="text"
          v-model.trim="store.targetDir"
          placeholder="可手动粘贴目录，或使用浏览按钮选择"
        />
        <div class="actions">
          <button type="button" @click="store.pickDirectory" class="browse-btn">
            <span class="icon">📁</span>
            <span>浏览</span>
          </button>
          <button
            type="button"
            class="primary"
            @click="() => store.scanDirectory()"
            :disabled="store.loading"
          >
            <span class="icon">{{ store.loading ? "⏳" : "🔍" }}</span>
            <span>{{ store.loading ? "扫描中..." : "扫描" }}</span>
          </button>
          <button
            type="button"
            class="secondary"
            @click="store.exportMappings"
            :disabled="!store.hasData || store.loading"
          >
            <span class="icon">📤</span>
            <span>导出</span>
          </button>
          <button
            type="button"
            class="secondary"
            @click="store.importMappings"
            :disabled="store.loading"
          >
            <span class="icon">📥</span>
            <span>导入</span>
          </button>
        </div>
      </div>
    </div>
    <p class="hint error" v-if="store.error">{{ store.error }}</p>
  </section>
</template>

<style scoped>
.picker {
  padding: 24px;
  background: linear-gradient(145deg, #ffffff, #f8fafc);
  border-radius: 24px;
  border: 1px solid rgba(15, 23, 42, 0.08);
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.header-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.header-row h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: #0f172a;
}

.stats {
  display: flex;
  gap: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  background: rgba(37, 99, 235, 0.1);
  border-radius: 12px;
}

.stat-label {
  font-size: 12px;
  color: #64748b;
}

.stat-value {
  font-size: 20px;
  font-weight: 600;
  color: #2563eb;
}

.input-row {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.path-input {
  width: 100%;
  padding: 14px 18px;
  border-radius: 16px;
  border: 1px solid rgba(15, 23, 42, 0.1);
  font-size: 15px;
  background: #fff;
}

.actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

button {
  border: none;
  border-radius: 12px;
  padding: 12px 20px;
  cursor: pointer;
  font-weight: 500;
  font-size: 14px;
  transition: all 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  position: relative;
  overflow: hidden;
}

button::before {
  content: "";
  position: absolute;
  top: 50%;
  left: 50%;
  width: 0;
  height: 0;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  transform: translate(-50%, -50%);
  transition: width 0.3s, height 0.3s;
}

button:hover::before {
  width: 300px;
  height: 300px;
}

button:active {
  transform: scale(0.98);
}

/* 浏览按钮 - 更优雅的样式 */
button:not(.primary):not(.secondary) {
  background: linear-gradient(135deg, #f1f5f9, #e2e8f0);
  color: #475569;
  border: 1px solid rgba(15, 23, 42, 0.08);
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.05);
}

button:not(.primary):not(.secondary):hover {
  background: linear-gradient(135deg, #e2e8f0, #cbd5e1);
  color: #334155;
  box-shadow: 0 4px 12px rgba(15, 23, 42, 0.1);
  transform: translateY(-1px);
}

button.primary {
  background: linear-gradient(135deg, #2563eb, #1d4ed8);
  color: #fff;
  box-shadow: 0 4px 14px rgba(37, 99, 235, 0.3);
}

button.primary:hover {
  background: linear-gradient(135deg, #1d4ed8, #1e40af);
  box-shadow: 0 6px 20px rgba(37, 99, 235, 0.4);
  transform: translateY(-2px);
}

button.secondary {
  background: linear-gradient(135deg, #10b981, #059669);
  color: #fff;
  box-shadow: 0 4px 14px rgba(16, 185, 129, 0.3);
}

button.secondary:hover {
  background: linear-gradient(135deg, #059669, #047857);
  box-shadow: 0 6px 20px rgba(16, 185, 129, 0.4);
  transform: translateY(-2px);
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none !important;
}

button:disabled:hover {
  box-shadow: none;
  transform: none !important;
}

button .icon {
  font-size: 16px;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  filter: grayscale(0);
  transition: transform 0.2s;
}

button:hover .icon {
  transform: scale(1.1);
}

button.browse-btn {
  background: linear-gradient(135deg, #f8fafc, #f1f5f9);
  color: #475569;
  border: 1.5px solid rgba(15, 23, 42, 0.1);
}

button.browse-btn:hover {
  background: linear-gradient(135deg, #f1f5f9, #e2e8f0);
  border-color: rgba(37, 99, 235, 0.3);
  color: #2563eb;
}

.link {
  background: transparent;
  color: #2563eb;
  padding: 0;
}

.meta-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}

.hint {
  margin: 0;
  font-size: 13px;
  color: #475467;
}

.hint.error {
  color: #dc2626;
}
</style>

