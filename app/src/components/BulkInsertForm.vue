<script setup lang="ts">
import { ref } from "vue";
import { open as openExternal } from "@tauri-apps/api/shell";
import { useMappingStore } from "@/stores/mappingStore";
import type { MappingInput } from "@/types/mapping";

const store = useMappingStore();
const entries = ref<MappingInput[]>([{ localId: "", gwId: "" }]);

/**
 * 打开备份文件夹。
 */
const openBackupFolder = async () => {
  if (!store.lastBackupDir) return;
  await openExternal(store.lastBackupDir);
};

/**
 * 提交批量新增请求。
 */
const handleSubmit = async () => {
  const payload = entries.value
    .filter((item: MappingInput) => item.localId && item.gwId)
    .map((item: MappingInput) => ({ ...item }));
  await store.bulkInsert(payload);
};

/**
 * 增加一条映射输入。
 */
const appendRow = () => {
  entries.value.push({ localId: "", gwId: "" });
};

/**
 * 删除指定行。
 */
const removeRow = (index: number) => {
  if (entries.value.length === 1) {
    entries.value[0] = { localId: "", gwId: "" };
    return;
  }
  entries.value.splice(index, 1);
};
</script>

<template>
  <section class="bulk-card">
    <header>
      <h2>批量新增映射</h2>
      <button type="button" @click="appendRow">增加行</button>
    </header>
    <form @submit.prevent="handleSubmit">
      <div class="grid">
        <div class="row" v-for="(entry, idx) in entries" :key="idx">
          <input v-model="entry.localId" placeholder="本地栏目 ID" />
          <input v-model="entry.gwId" placeholder="国网栏目 ID" />
          <button type="button" class="ghost" @click="removeRow(idx)">
            删除
          </button>
        </div>
      </div>
      <div class="footer">
        <button type="submit" class="primary">写入所有文件</button>
        <div v-if="store.lastInsertReport" class="report">
          <div class="report-summary">
            <span class="success">成功：{{ store.lastInsertReport.updatedFiles.length }}</span>
            <span v-if="store.lastInsertReport.skippedFiles.length > 0" class="warning">
              跳过：{{ store.lastInsertReport.skippedFiles.length }}
            </span>
          </div>
          <div v-if="store.lastBackupDir" class="backup-info">
            <span>备份：{{ store.lastBackupDir }}</span>
            <button type="button" class="link-btn" @click="openBackupFolder">打开</button>
          </div>
          <div v-if="store.lastInsertReport.skippedFiles.length > 0" class="skipped-details">
            <div
              v-for="skipped in store.lastInsertReport.skippedFiles"
              :key="skipped.filePath"
              class="skipped-item"
            >
              <strong>{{ skipped.filePath }}</strong>
              <span class="reason">{{ skipped.reason }}</span>
            </div>
          </div>
        </div>
      </div>
    </form>
  </section>
</template>

<style scoped>
.bulk-card {
  margin-top: 20px;
  padding: 24px;
  background: linear-gradient(120deg, #0f172a, #1d4ed8);
  color: #fff;
  border-radius: 30px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

button {
  border: none;
  border-radius: 999px;
  padding: 10px 18px;
  cursor: pointer;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.18);
  color: #fff;
}

button.primary {
  background: #fff;
  color: #0f172a;
}

button.ghost {
  background: transparent;
  color: #fca5a5;
}

.grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.row {
  display: grid;
  grid-template-columns: 1fr 1fr auto;
  gap: 8px;
  background: rgba(15, 23, 42, 0.3);
  padding: 12px;
  border-radius: 16px;
}

input {
  padding: 10px 12px;
  border-radius: 8px;
  border: none;
  background: rgba(255, 255, 255, 0.9);
  color: #0f172a;
}

.footer {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.report {
  display: flex;
  flex-direction: column;
  gap: 10px;
  font-size: 13px;
}

.report-summary {
  display: flex;
  gap: 16px;
}

.report-summary .success {
  color: rgba(255, 255, 255, 0.95);
}

.report-summary .warning {
  color: #fbbf24;
}

.backup-info {
  display: flex;
  align-items: center;
  gap: 8px;
  color: rgba(255, 255, 255, 0.85);
  font-size: 12px;
}

.backup-info span {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.link-btn {
  padding: 4px 12px;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.15);
}

.skipped-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.skipped-details {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.15);
  border-radius: 8px;
  border-left: 3px solid #ef4444;
}

.skipped-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 12px;
}

.skipped-item strong {
  color: rgba(255, 255, 255, 0.95);
  font-weight: 600;
}

.skipped-item .reason {
  color: rgba(255, 255, 255, 0.8);
  margin-left: 8px;
}
</style>

