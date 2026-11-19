/**
 * 关于页的二维码图走公开图片仓库，不进安装包（ADR-0004）。
 *
 * 模板只写 data-remote-img="文件名"，镜像前缀在这里拼；GitHub raw 不通就换 Gitee。
 * 两处都不通时隐藏图片、露出文字兜底——不要让一个破图标占着位置假装加载成功。
 */
const QR_MIRRORS = [
  "https://raw.githubusercontent.com/ShiXiongZhiDao/BrandAssets/main/images/about/",
  "https://gitee.com/ShiXiongZhiDao/BrandAssets/raw/main/images/about/",
];

function loadRemoteImage(img: HTMLImageElement): void {
  if (img.dataset.loaded === "1") return;
  const attempt = Number(img.dataset.try ?? "0");
  if (attempt >= QR_MIRRORS.length) {
    img.hidden = true;
    img.closest(".remote-img-slot")?.classList.add("failed");
    return;
  }
  img.dataset.try = String(attempt + 1);
  img.onerror = () => loadRemoteImage(img);
  img.onload = () => {
    img.dataset.loaded = "1";
  };
  img.src = QR_MIRRORS[attempt] + (img.dataset.remoteImg ?? "");
}

/** 弹窗每次打开后调用一次；已加载的图片不会重复请求。 */
export function wireRemoteImages(root: HTMLElement | null): void {
  if (!root) return;
  for (const img of root.querySelectorAll<HTMLImageElement>("img[data-remote-img]")) {
    loadRemoteImage(img);
  }
}
