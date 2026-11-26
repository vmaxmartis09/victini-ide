# ⚡ VICTINI IDE — Autonomous Vibe-Code Station

Victini là một IDE Vibe-Code siêu nhẹ (100% Rust Native `egui`/`wgpu`), thiết kế chuẩn theo phong cách **Qoder Studio** và **T3 Code**. Kết nối trực tiếp với não bộ **Antigravity CLI** (`agy`), tích hợp quản trị Git/Worktree, Remote Emulator (ADB/CDP), và dàn Avatar chuyển động **Notionists** (5 nhân vật từ core_vm).

---

## 🚀 Dành Cho Máy Windows (Đặc biệt máy công ty bị chặn tải)

Repo này được chuẩn bị sẵn để **chạy ngay hoặc code tiếp trên Windows**:

### Cách 1: Chạy ngay không cần cài bất cứ thứ gì
Nhấp đúp chuột vào file:
```cmd
run-windows.bat
```
*(File này sẽ tự động chạy bản dựng sẵn `bin/victini.exe` siêu nhẹ ~5MB, tiêu thụ chỉ ~60MB RAM, không cần quyền Admin).*

---

### Cách 2: Code tiếp & Biên dịch trên Windows
Nếu Ngài muốn chỉnh sửa mã nguồn và biên dịch tiếp:

1. **Nếu máy công ty chưa có Rust:**
   Chạy file cấu hình tự động không cần quyền Admin:
   ```cmd
   tools\setup-portable-rust.bat
   ```
   *(Script tải trực tiếp toolchain portable vào User profile `%USERPROFILE%\.cargo`, vượt qua các rào cản phân quyền Admin của công ty).*

2. **Chạy & Biên dịch:**
   ```cmd
   cargo run --release
   ```

---

## 🛠 Cấu Trúc Dự Án
- `src/` — Toàn bộ mã nguồn Rust 12 Phase hoàn thiện.
- `assets/` — Font chữ Google **Be Vietnam Pro** và toàn bộ Avatar Notionists SVG.
- `bin/` — File nhị phân dựng sẵn `victini.exe` chạy ngay.
- `tools/` — Tiện ích thiết lập môi trường portable cho Windows.
- `run-windows.bat` — Trình khởi chạy 1-click thông minh.

---

## 🔗 Liên Kết Não Bộ Antigravity (AGY)
Victini tự động nhận diện binary AGY tại:
- **Windows:** `%USERPROFILE%\.gemini\bin\agy.exe` hoặc `%USERPROFILE%\.local\bin\agy.exe`
- **macOS:** `~/.local/bin/agy` hoặc `~/.gemini/bin/agy`
