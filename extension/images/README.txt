注意：
请将你的大头照以 PNG 格式放入 images/ 文件夹，并按数字编号命名：
  1.png  → 人物1
  2.png  → 人物2
  ...以此类推

建议：
- 使用透明背景的 PNG 图片
- 用 https://www.photoroom.com/tools/background-remover 去除背景
- 用 https://compresspng.com/ 压缩图片
- 替换 icon.png 为你的扩展图标（96x96 PNG）

如果某些图片包含文字不希望被翻转，编辑 images/flip_blacklist.json：
{
    "useAlternativeImages": false,
    "blacklistedImages": ["1", "3"]
}
