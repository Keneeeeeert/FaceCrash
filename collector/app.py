import tkinter as tk
from tkinter import ttk
from PIL import Image, ImageTk
import cv2
import threading
import time
import os

from rembg import remove
import numpy as np

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
IMAGES_DIR = os.path.join(SCRIPT_DIR, "images")
os.makedirs(IMAGES_DIR, exist_ok=True)


def next_filename():
    files = [f for f in os.listdir(IMAGES_DIR) if f.endswith(".png") and f[:-4].isdigit()]
    max_num = 2
    for f in files:
        n = int(f[:-4])
        if n > max_num:
            max_num = n
    return f"{max_num + 1:02d}.png"


class App:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title("透明背景 PNG 采集器")
        self.root.configure(bg="#1a1a2e")
        self.root.resizable(False, False)

        self.cap = cv2.VideoCapture(0)
        self.cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
        self.cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)

        self.capturing = False
        self.running = True
        self.frame = None
        self.last_result = None

        self._build_ui()
        self._camera_loop()
        self.root.protocol("WM_DELETE_WINDOW", self._on_close)
        self.root.mainloop()

    def _build_ui(self):
        title = tk.Label(self.root, text="透明背景 PNG 采集器",
                         font=("Microsoft YaHei UI", 16, "bold"),
                         fg="#e94560", bg="#1a1a2e")
        title.pack(pady=(16, 10))

        self.vp_frame = tk.Frame(self.root, width=640, height=480, bg="#000",
                                 highlightbackground="#333", highlightthickness=2)
        self.vp_frame.pack(padx=20)
        self.vp_frame.pack_propagate(False)

        self.vp_label = tk.Label(self.vp_frame, bg="#000")
        self.vp_label.place(relwidth=1, relheight=1)

        self.overlay = tk.Frame(self.vp_frame, bg="#000000d9")
        self.countdown_label = tk.Label(self.overlay, text="",
                                        font=("Microsoft YaHei UI", 80, "bold"),
                                        fg="#fff", bg="#000000d9")
        self.processing_label = tk.Label(self.overlay, text="",
                                         font=("Microsoft YaHei UI", 20, "bold"),
                                         fg="#e94560", bg="#000000d9")
        self.progress = ttk.Progressbar(self.overlay, mode="determinate", length=320)
        self.eta_label = tk.Label(self.overlay, text="",
                                  font=("Microsoft YaHei UI", 10),
                                  fg="#999", bg="#000000d9")

        self.btn = tk.Button(self.root, text="拍摄", font=("Microsoft YaHei UI", 13, "bold"),
                             bg="#e94560", fg="#fff", activebackground="#c73650",
                             activeforeground="#fff", relief="flat", padx=40, pady=10,
                             command=self._on_capture, state="normal")
        self.btn.pack(pady=16)

        self.status = tk.Label(self.root, text="",
                               font=("Microsoft YaHei UI", 9),
                               fg="#888", bg="#1a1a2e")
        self.status.pack(pady=(0, 10))

    def _camera_loop(self):
        if not self.running:
            return
        ret, frame = self.cap.read()
        if ret:
            frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            frame = cv2.resize(frame, (640, 480))
            self.frame = frame
            img = Image.fromarray(frame)
            imgtk = ImageTk.PhotoImage(img)
            self.vp_label.configure(image=imgtk)
            self.vp_label.image = imgtk
        self.root.after(33, self._camera_loop)

    def _show_overlay(self):
        self.overlay.place(relwidth=1, relheight=1)

    def _hide_overlay(self):
        self.overlay.place_forget()
        self.countdown_label.place_forget()
        self.processing_label.place_forget()
        self.progress.place_forget()
        self.eta_label.place_forget()

    def _on_capture(self):
        if self.capturing or self.frame is None:
            return
        self.capturing = True
        self.btn.configure(state="disabled")
        self.status.configure(text="")
        threading.Thread(target=self._capture_flow, daemon=True).start()

    def _capture_flow(self):
        for i in range(3, 0, -1):
            self.root.after(0, self._show_overlay)
            self.root.after(0, lambda n=i: self.countdown_label.configure(text=str(n)))
            self.root.after(0, lambda: self.countdown_label.place(relx=0.5, rely=0.5, anchor="center"))
            time.sleep(1)

        self.root.after(0, self._hide_overlay)

        frame_bgr = cv2.cvtColor(self.frame, cv2.COLOR_RGB2BGR)
        self.root.after(0, self._show_processing_ui)

        try:
            result_bgra = remove(frame_bgr)
            result_rgba = cv2.cvtColor(result_bgra, cv2.COLOR_BGRA2RGBA)
            pil_img = Image.fromarray(result_rgba)

            fname = next_filename()
            fpath = os.path.join(IMAGES_DIR, fname)
            pil_img.save(fpath, "PNG")
            self.last_result = pil_img

            self.root.after(0, lambda: self._on_done(fname, pil_img))
        except Exception as e:
            self.root.after(0, lambda: self._on_error(str(e)))

    def _show_processing_ui(self):
        self._show_overlay()
        self.processing_label.configure(text="AI 抠图中...")
        self.processing_label.place(relx=0.5, rely=0.38, anchor="center")
        self.progress["value"] = 0
        self.progress.place(relx=0.5, rely=0.52, anchor="center")
        self.eta_label.configure(text="处理中，请稍候...")
        self.eta_label.place(relx=0.5, rely=0.60, anchor="center")
        self.progress.configure(mode="indeterminate")
        self.progress.start(15)

    def _on_done(self, fname, pil_img):
        self.progress.stop()
        self._hide_overlay()
        self.btn.configure(state="normal")
        self.capturing = False
        self.status.configure(text=f"已保存: {fname}")

    def _on_error(self, msg):
        self.progress.stop()
        self._hide_overlay()
        self.btn.configure(state="normal")
        self.capturing = False
        self.status.configure(text=f"失败: {msg}")

    def _on_close(self):
        self.running = False
        self.cap.release()
        self.root.destroy()


if __name__ == "__main__":
    App()
