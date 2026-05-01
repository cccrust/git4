# git4 開發規範與計畫指南 (AGENTS.md)

這份文件旨在定義給 AI Agent 以及其他維護者在進行 `git4` 專案開發時的標準操作流程（SOP）與專案整體計畫藍圖。

## 開發操作規定 (SOP)

為了保持本專案所有版本疊代的脈絡清晰，在每一次發布新版本 (例如 v0.3、v0.4...) 時，請嚴格遵守以下開發規定：

1. **先寫出規劃文件**:
   - 在實際撰寫程式碼之前，永遠先在 `_doc/v0.X.md` (例如 `_doc/v0.3.md`) 中寫下該版預計要解決的問題與實作目標。
2. **進行程式碼實作與測試**:
   - 根據規劃編寫 Rust 程式碼並修改核心模組。
   - 同步修改並擴增自動測試腳本 `test.sh`，確保新功能的機制涵蓋正確。
   - 利用 `cargo build` 與執行 `./test.sh` 來保證功能順利無回歸錯誤。
3. **完成編碼後補充開發紀錄**:
   - 程式撰寫與測試順利完成後，回頭去擴充原本該版本的 `_doc/v0.X.md` 文件。
   - 補上「實作細節」、「除錯紀錄」與「下一步的展望」，讓這份文件轉變成該版本的完整開發紀錄憑證。
4. **推進專案版本號**:
   - 完成一個特定版號的功能週期工作後，必須前往 `Cargo.toml` 裡面，將 `version` 的版號進行遞增更新 (例如由 `0.2.0` 更新至 `0.3.0`)。

## 專案開發總體計畫

`git4` 是一個基於 Rust 所建構的輕量版本控制系統，旨在以最簡潔且無依賴外部底層工具的方式，自行建立並還原 git 的核心工作原理。

### 已完成階段
- **v0.1**: 專案基礎建設。確立資料儲存結構 (`.git4/objects`, `refs`, `HEAD`)，以 `flate2` 封裝 Zlib 並利用 `sha1` 實作雜湊。具備產生 Blob、Tree 與 Commit 的指令，以及最初階的 `add`, `commit`, `log` 打底流程。
- **v0.2**: 分支系統。成功加入了 `branch` 指令以產生多線開發管理能力，並實體還原了 `checkout` 功能，使 `git4` 有能力將 Tree 物件從資料庫中重構到工作目錄上。
- **v0.3 - 狀態管理 (`status`)**: 實作三向比對引擎，能精準列出 `to be committed`, `not staged`, `untracked` 之差異狀態。
- **v0.4 - 差異與合併 (`diff` / `merge`)**: 整合 `similar` 套件做到文字級逐行比對 (`diff`)，並實作了 Fast-forward 的快轉合併防護機制。
- **v0.5 - 本地跨倉儲遠端操作**: 以跨實體目錄模擬 `clone`, `push`, `fetch`。具備跨資料夾封裝對拷並獨立解析還原分支的能力。

### 未來預定階段：從地端走向聯網 (v0.6 ~ v1.0)
為了讓 `git4` 能夠真正與遠端伺服器（例如 GitHub）接軌，接下來必須擁抱 HTTP Smart Git 協定與 `.pack` 綜合打包機制：

- **v0.6 - `remote` 管理與 HTTP 探索**: 
  - 實作 `.git4/config` 設定檔，新增 `git4 remote add <name> <url>` 來記憶網域。
  - 開發對 HTTP 端點 `GET info/refs?service=git-upload-pack` 的通訊請求，能夠解讀來自 GitHub 等伺服器的分支名單與 SHA1 hash 發現端點。
- **v0.7 - Packfile 壓縮解碼器引擎**:
  - 真實 Git 通訊不會傳送鬆散物件，而是 `.pack` 檔。
  - 編寫能讀取、解封裝 Packfile Index (OBJ_COMMIT/TREE/BLOB) 的引擎，並試探性著手處理 Delta 二進位轉換。
- **v0.8 - 真實聯網封包拉取 (`clone` / `fetch` over HTTP)**:
  - 以 `POST git-upload-pack` 指出本地需要的 `want <hash>`。
  - 接收 Packfile 資料流，利用 v0.7 引擎解開並透過 `checkout` 展開檔案，完美實現真正透過網路複製 GitHub 開源專案！
- **v0.9 - 認證與打包推送 (`push` over HTTP)**:
  - 實作本機 Packfile 生成器，將新增的 Commit 打包。
  - 利用授權標頭處理 API 密碼認證，向 GitHub 的 `git-receive-pack` 端點推送進度。
- **v1.0 - 完整的極簡 Git Client 里程碑**:
  - 發揮 Rust 效能進行大型專案的最佳化讀取。
  - 實作實用且複雜的三方合併 (`Three-way merge`) 或解決衝突 (`conflict handling`) 等高級系統核心，使本專案能穩定上線作為日常使用！
