# FaceCrash

用自己的照片闯入 B 站每一个视频封面 —— 拍照、抠图、一键贴满全站。

灵感来源于 YouTube 上的 [MrBeastify](https://github.com/mrbeastify/mrbeastify) 扩展。

## 和 MrBeastify 有什么区别？

MrBeastify 把 MrBeast 的脸贴到 YouTube 封面上。FaceCrash 让你用自己的脸（或者任何人的脸）贴到 B 站封面上——内置摄像头拍照和 AI 抠图工具，不需要任何图片处理技能。

## 功能

- 摄像头实时拍照，AI 自动抠图生成透明 PNG
- 自动检测 B 站首页、搜索页、视频页的封面
- 将你的照片叠加到每一个封面上
- 可配置出现概率、水平翻转概率
- 支持多张照片随机轮换

## 项目结构

```
├── extension/          # 浏览器扩展 (Chrome/Edge MV3)
│   ├── manifest.json
│   ├── utils.js        # 共享工具函数
│   ├── content.js      # 内容脚本：封面检测 + 图像叠加
│   ├── popup.html      # 扩展弹窗界面
│   ├── popup.js        # 弹窗逻辑（设置页）
│   ├── icon.png
│   └── images/         # 人物 PNG 图片
│       ├── 1.png ...   # 按数字编号命名
│       └── flip_blacklist.json
├── collector/          # 拍照抠图工具 (Rust + Python)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs     # GUI 入口 (egui)
│   │   ├── app.rs      # 应用逻辑
│   │   ├── camera.rs   # 摄像头采集
│   │   └── bg_remover.rs  # AI 背景移除 (rembg)
│   ├── app.py          # 抠图处理脚本
│   └── 启动.bat        # 一键启动脚本
├── .gitignore
└── README.md
```

## 快速开始

### 1. 制作你的照片

使用 `collector/` 目录下的工具拍照并自动抠图：

- 确保安装了 Python 3 和 Rust 工具链
- 安装 Python 依赖：`pip install rembg opencv-python`
- 双击 `启动.bat` 或运行 `cargo run --release`
- 用摄像头拍照，工具会自动去除背景，保存为透明 PNG

生成的照片放入 `extension/images/`，按数字命名：
- `1.png` → 照片 1
- `2.png` → 照片 2
- ...以此类推

扩展启动时会自动扫描所有图片。不想自己拍？直接往 `extension/images/` 里丢任何透明底 PNG 也行。

### 2. 安装扩展

1. 打开 `chrome://extensions` 或 `edge://extensions`
2. 启用「开发者模式」
3. 点击「加载解压缩的扩展」，选择 `extension/` 文件夹
4. 访问 B 站任意页面即可看到效果

### 3. 调整设置

点击扩展图标打开设置面板：

| 选项 | 说明 |
|---|---|
| 启用扩展 | 开关扩展 |
| 人物选择 | 选择特定照片或随机轮换 |
| 出现概率 | 封面叠加的概率 (0-100%) |
| 翻转概率 | 水平翻转的概率 (0-100%) |

## 技术栈

- **扩展**: Chrome Extension Manifest V3, 纯 JavaScript
- **采集工具**: Rust (egui + nokhwa), Python (rembg + OpenCV)
- **AI 模型**: U²-Net (salient object detection)

## Contributors

- [Keneeeeeert](https://github.com/Keneeeeeert)
- opencode (AI)

## License

MIT
