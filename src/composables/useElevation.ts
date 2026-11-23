/**
 * 受限模式 → 管理员实例（ADR-0010）。
 *
 * 只负责发起重启：成功后当前进程会由 Rust 侧的 finish_elevation 退出；
 * UAC 取消时保持当前实例与受限模式，不把用户留在半关闭状态。
 */
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessage } from "naive-ui";
import { useConnectionsStore } from "../stores/connections";
import { useI18n } from "../i18n";
import { encodeElevationState } from "../lib/launch-state";
import { errorCode } from "../lib/kill-error";

const elevating = ref(false);

export function useElevation() {
  const store = useConnectionsStore();
  const { t } = useI18n();
  const message = useMessage();

  async function elevate(): Promise<void> {
    if (elevating.value || store.isElevated) return;
    elevating.value = true;
    try {
      const state = encodeElevationState({
        view: store.view,
        search: store.search,
        protocol: store.protocol,
        status: store.status,
        family: store.family,
        listeningOnly: store.listeningOnly,
      });
      const launched = await invoke<boolean>("relaunch_elevated", { state });
      if (!launched) {
        // 后端发现当前实例已经是管理员：只纠正 UI，不退出进程。
        store.isElevated = true;
        elevating.value = false;
        return;
      }
    } catch (e) {
      const code = errorCode(e);
      if (code === "ELEVATION_CANCELLED") {
        message.info(t("elevate.cancelled"));
      } else {
        message.error(t("elevate.failed"));
      }
      elevating.value = false;
      return;
    }
    // 提权进程已创建：隐藏托盘并退出当前实例。
    // 退出过程可能掐断 IPC，不能把它的失败当成"提权失败"误报。
    void invoke("finish_elevation").catch(() => {
      elevating.value = false;
    });
  }

  return { elevate, elevating };
}
