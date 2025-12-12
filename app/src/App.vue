<script setup lang="ts">
import { ref } from "vue";
import DirectoryPicker from "@/components/DirectoryPicker.vue";
import BulkInsertForm from "@/components/BulkInsertForm.vue";
import MappingTable from "@/components/MappingTable.vue";
import { useMappingStore } from "@/stores/mappingStore";

const store = useMappingStore();

const showGuide = ref(false);
const guideSteps = [
  {
    title: "选择目标目录",
    desc: "点击「浏览」选择包含 theme*.json 的目录，或直接在输入框粘贴路径。",
  },
  {
    title: "版本号管理",
    desc: "在目标目录区域可以看到「自动递增版本号」配置选项（默认开启）。启用后，每次修改 theme*.json 文件都会自动将文件中的 version 字段值加 1。",
  },
  {
    title: "扫描文件",
    desc: "点击「扫描」按钮，程序会自动识别所有 theme*.json 文件中的 sExtOptions 段落，提取 portal_frag_* 映射关系。",
  },
  {
    title: "查看结果",
    desc: "扫描后可在下方查看每个文件的映射详情，包括本地栏目ID、国网栏目ID和状态（正常/重复）。结果按本地栏目ID数值大小自动排序。",
  },
  {
    title: "对比视图",
    desc: "点击「对比显示」按钮，可以查看所有文件的映射关系对比。相同映射会自动折叠，不同映射会高亮显示，方便快速定位差异。",
  },
  {
    title: "批量新增",
    desc: "在批量新增区域填写本地栏目ID和国网栏目ID，点击「写入所有文件」即可同步到所有 theme 文件。操作前会自动创建备份。",
  },
  {
    title: "删除映射",
    desc: "支持单个和批量删除。在映射表格中勾选需要删除的项，点击「批量删除」按钮。所有删除操作前都会自动创建备份。",
  },
  {
    title: "导出/导入",
    desc: "「导出」可将去重后的映射导出为 JSON 文件；「导入」可批量替换现有映射。导入前会自动创建备份并显示确认对话框。",
  },
  {
    title: "操作日志",
    desc: "每次批量操作后，程序会在处理文件的目录下自动生成操作日志文件，记录操作详情、备份路径、版本变化（旧版本 → 新版本）和处理的映射信息。",
  },
  {
    title: "自动备份",
    desc: "所有文件修改操作（新增、删除、导入）前都会自动创建备份，备份路径显示在操作结果中，可通过「打开」按钮查看。",
  },
  {
    title: "注意事项",
    desc: "本地栏目ID必须唯一，重复时会标记为错误；国网栏目ID可以重复但会给出警告提示。版本号管理功能可随时通过界面开关控制。",
  },
];
</script>

<template>
  <main class="dashboard">
    <section class="hero">
      <div class="hero-text">
        <h1>栏目映射管理工具</h1>
        <p class="subtitle">
          扫描 theme*.json 的 sExtOptions，识别并批量维护本地/国网栏目ID映射
        </p>
      </div>
      <button class="help-btn" type="button" @click="showGuide = !showGuide">
        使用指南
      </button>
    </section>

    <section class="control-grid">
      <DirectoryPicker />
      <BulkInsertForm />
    </section>

    <transition name="fade">
      <div v-if="showGuide" class="guide-overlay" @click="showGuide = false">
        <div class="guide-panel" @click.stop>
          <header>
            <h3>使用指南</h3>
            <button type="button" @click="showGuide = false">×</button>
          </header>
          <ol>
            <li v-for="(step, idx) in guideSteps" :key="idx">
              <strong>{{ step.title }}</strong>
              {{ step.desc }}
            </li>
          </ol>
        </div>
      </div>
    </transition>

    <MappingTable :files="store.files" />
  </main>
</template>

<style scoped>
.dashboard {
  padding: 40px;
  max-width: 1280px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.hero {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 24px;
  padding: 32px 36px;
  border-radius: 32px;
  background: linear-gradient(135deg, #101828, #1f4fe0);
  color: #fff;
}

.hero-text h1 {
  margin: 0 0 8px;
  font-size: 28px;
  font-weight: 600;
}

.hero-text .subtitle {
  margin: 0;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.8);
  line-height: 1.5;
}

.help-btn {
  border: none;
  border-radius: 999px;
  padding: 10px 20px;
  background: rgba(255, 255, 255, 0.18);
  color: #fff;
  cursor: pointer;
  font-weight: 500;
  white-space: nowrap;
  transition: background 0.2s;
}

.help-btn:hover {
  background: rgba(255, 255, 255, 0.25);
}

.control-grid {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.guide-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.5);
  backdrop-filter: blur(4px);
  display: flex;
  justify-content: flex-end;
  align-items: flex-start;
  padding: 40px;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.guide-panel {
  width: min(560px, 90vw);
  max-height: 90vh;
  overflow-y: auto;
  background: linear-gradient(145deg, #ffffff, #f8fafc);
  border-radius: 28px;
  padding: 0;
  box-shadow: 0 32px 64px rgba(15, 23, 42, 0.25),
    0 0 0 1px rgba(15, 23, 42, 0.05);
  display: flex;
  flex-direction: column;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.guide-panel header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 28px;
  border-bottom: 1px solid rgba(15, 23, 42, 0.08);
  background: linear-gradient(135deg, #1f4fe0, #2563eb);
  color: #fff;
  border-radius: 28px 28px 0 0;
}

.guide-panel header h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.guide-panel button {
  border: none;
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
  font-size: 24px;
  width: 32px;
  height: 32px;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
  line-height: 1;
  padding: 0;
}

.guide-panel button:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: scale(1.1);
}

.guide-panel ol {
  margin: 0;
  padding: 24px 28px;
  list-style: none;
  counter-reset: step-counter;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.guide-panel ol li {
  counter-increment: step-counter;
  position: relative;
  padding-left: 48px;
  color: #334155;
  line-height: 1.7;
  font-size: 14px;
}

.guide-panel ol li::before {
  content: counter(step-counter);
  position: absolute;
  left: 0;
  top: 0;
  width: 32px;
  height: 32px;
  background: linear-gradient(135deg, #2563eb, #1f4fe0);
  color: #fff;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.3);
}

.guide-panel ol li strong {
  color: #0f172a;
  font-weight: 600;
  display: block;
  margin-bottom: 4px;
}

@media (max-width: 1024px) {
  .hero {
    flex-direction: column;
    align-items: flex-start;
  }

  .hero-text .subtitle {
    font-size: 13px;
  }

  .guide-overlay {
    padding: 24px;
  }
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
