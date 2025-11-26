<script setup lang="ts">
import { computed } from "vue";
import type { FileMapping } from "@/types/mapping";

const props = defineProps<{
  files: FileMapping[];
}>();

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
</script>

<template>
  <section class="table-wrapper">
    <div v-if="isEmpty" class="empty">尚未扫描到任何 theme*.json 文件。</div>
    <div v-else class="file-grid">
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
              <th>本地栏目 ID</th>
              <th>国网栏目 ID</th>
              <th>状态</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="mapping in file.mappings"
              :key="mapping.localId"
              :class="{
                'row-error': mapping.status === 'duplicate_local',
                'row-warning': mapping.status === 'duplicate_gw'
              }"
            >
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
            </tr>
          </tbody>
        </table>
      </article>
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
</style>

