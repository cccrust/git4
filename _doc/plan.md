# git4 - 規劃文件

`git4` 是一個用 Rust 實作的輕量級版本控制系統，目的是能夠模擬基本的 git 行為，包含初始化、儲存物件、暫存區管理、提交以及歷史紀錄追蹤。

## 系統架構

`git4` 包含以下核心概念：
1. **Repository (儲存庫)** - `.git4` 資料夾，用來儲存所有歷史紀錄與物件。
2. **Objects (物件系統)** 
    - **Blob**: 紀錄檔案內容。
    - **Tree**: 紀錄目錄結構以及關聯的 Blob / Tree。
    - **Commit**: 紀錄某個時間點的 Tree、提交訊息、作者以及父提交。
3. **Index (暫存區)** - 準備被提交的檔案快取。
4. **Refs/HEAD**: 紀錄分支與當前所在的指標。

## 開發計畫 (分階段)

### 階段一：專案初始化與儲存庫建立
- `git4 init`：建立 `.git4` 隱藏目錄結構 (包括 `objects`, `refs`, `HEAD`)。
- 使用 `cargo init` 來建立 Rust 專案基礎。

### 階段二：核心底層指令 (Plumbing)
- `git4 hash-object <file>`：將檔案打成 blob 並產生 SHA-1 雜湊值（支援寫入 `-w`）。
- `git4 cat-file <type> <object>`：讀取並輸出物件內容，以驗證 Zlib 解壓縮與格式解析。
- SHA-1 雜湊與 Zlib 壓縮功能的整合。

### 階段三：目錄樹處理
- `git4 write-tree`：將當前工作目錄的狀態生成 Tree 物件並寫入物件儲存庫中。
- `git4 read-tree <tree_oid>`：將 Tree 物件的內容檢視並對應回工作資料夾。

### 階段四：提交機制
- `git4 commit-tree <tree> -p <parent> -m "msg"`：建立 Commit 物件。
- 解析 HEAD 與更新 ref 機制。

### 階段五：高階指令 (Porcelain)
- `git4 add <file>`：模擬寫入暫存區 (更新 index 或直接封裝成 hash-object 的簡化版)。
- `git4 commit -m <msg>`：整合 `add`, `write-tree`, `commit-tree` 以及更新 HEAD。
- `git4 log`：沿著 commit 往下追蹤並印出提交歷史。

## 依賴庫
- `sha1`: 產生 SHA-1 雜湊。
- `flate2`: 提供 Zlib 壓縮與解壓，處理 git 儲存的格式。
- `clap`: 提供命令行介面的引數解析。
- `anyhow`: 處理錯誤與提示。
