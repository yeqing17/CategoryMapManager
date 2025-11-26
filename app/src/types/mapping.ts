/**
 * 单条栏目映射数据。
 */
export interface MappingEntry {
  localId: string;
  gwId?: string;
  rawValue: string;
  sameId: boolean;
  /** 状态：normal-正常, duplicate_local-本地ID重复, duplicate_gw-国网ID重复 */
  status: string;
}

/**
 * 单个 theme 文件的映射结果。
 */
export interface FileMapping {
  filePath: string;
  mappings: MappingEntry[];
}

/**
 * Tauri 侧返回的扫描响应。
 */
export interface ScanResponse {
  files: FileMapping[];
}

/**
 * 批量新增映射的输入。
 */
export interface MappingInput {
  localId: string;
  gwId: string;
}

/**
 * 跳过的文件信息。
 */
export interface SkippedFile {
  filePath: string;
  reason: string;
  duplicateIds: string[];
}

/**
 * 后端返回的批量新增结果。
 */
export interface BulkInsertResult {
  updatedFiles: string[];
  skippedFiles: SkippedFile[];
  backupDir?: string;
}

