import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

/**
 * 检查更新（每个 endpoint 最多等待 20 秒，避免网络不通时长时间卡顿）
 * @returns {Promise<object|null>} 返回 Update 对象，无更新时返回 null
 */
export const checkForUpdate = () => check({ timeout: 20000 });

/**
 * 下载并安装更新，通过 onProgress 回调报告进度
 * @param {object} update - checkForUpdate 返回的 Update 对象
 * @param {(percent:number, downloaded:number, total:number)=>void} onProgress
 */
export async function downloadAndInstallUpdate(update, onProgress) {
  let downloaded = 0;
  let contentLength = 0;
  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        contentLength = event.data.contentLength || 0;
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        if (onProgress && contentLength > 0) {
          onProgress(
            Math.min(100, Math.round((downloaded / contentLength) * 100)),
            downloaded,
            contentLength
          );
        }
        break;
      case "Finished":
        if (onProgress) onProgress(100, contentLength, contentLength);
        break;
    }
  });
}

/**
 * 重启应用以应用更新
 */
export const relaunchApp = () => relaunch();
