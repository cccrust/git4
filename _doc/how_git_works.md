# 深入理解 Git 的運作原理 —— 與 git4 開發歷程對照

要真正弄懂 Git 為什麼強大，最快的方法就是親手把它重寫一遍！這份文件將闡述 Git 真正的背後運作原理，並逐一對照我們的 `git4` 專案如何透過每一次的版本升級 (v0.1 ~ v0.6)，一步步把這個強大的版本控制系統從零建立起來。

---

## 核心概念：Git 是一個「內容尋址檔案系統」(Content-Addressable Filesystem)
大眾常誤以為 Git 是單純記錄「變更 (Delta)」的系統，但實際上 Git 核心是一個單純的鍵值資料庫 (Key-Value Datastore)。它將檔案內容丟進雜湊函數 (SHA-1) 算出 40 字元的長度名稱，然後把內容壓縮丟進資料庫 (`.git/objects/`) 裡。

Git 依賴三種核心物件 (Objects) 來拼湊出歷史：
1. **Blob**: 檔案內容。裡面「沒有」檔名，純粹只有資料與長度。
2. **Tree**: 目錄結構。紀錄了一組清單，把「檔名」對應到特定的 Blob Hash 或是另一個 Tree Hash。
3. **Commit**: 歷史快照。紀錄了某個時間點的 Tree、作者是誰、提交訊息，以及「它的父親 (parent) 是誰」。

加上一個稱為 **References (Refs)** 的指標系統（例如 分支與 HEAD），這些簡單的文字檔裡面只存著 40 個字元的 Commit Hash，讓整套系統活了起來。

---

## git4 的演進與對應技術原理

### 🔧 v0.1 - 基礎物件儲存與地基 (Objects & Hashes)
- **Git 原理**: Git 會對每一個物件加上專屬的檔頭 `[物件類型] [大小]\0`，再與檔案內容串接，接著進行 Zlib 壓縮，並取其 SHA-1 放進 `.git/objects/` 底下。
- **git4 的實現**: 
  - 我們引入了 `sha1` 與 `flate2`。
  - 實作了底層探測指令 `hash-object` 與 `cat-file`，完美重現了封裝與解壓縮 Git 物件的基礎能力。
  - 實作了 `write-tree` 演算法，遞迴掃描硬碟，從最底層的檔案建立 Blob，並一層一層建立 Tree，最終用 `commit-tree` 把它加上作者與父節點 (parent) 成為了 Commit。

### 🌿 v0.2 - 分支系統與狀態切換 (Branching & Checkout)
- **Git 原理**: 「開分支」在 Git 中是非常廉價的操作，因為它就只是在 `.git/refs/heads/<branch>` 開一個文字檔，把 40 個字元的 Commit Hash 寫入而已。而 `checkout` 則是指將特定 Commit 裡的 Tree 讀出來並強行解壓縮覆蓋到工作目錄上。
- **git4 的實現**:
  - 開發了 `branch` 工具，只是簡單讀寫指標檔案。
  - 實作了 `checkout`，寫出了一套 `restore_tree` 的遞迴還原引擎。讀取 Commit 中的 Tree 雜湊，把裡面記錄的 Blob 透過 Zlib 解壓縮回實體磁碟上，並使用 Unix `fs::set_permissions` 忠實還原檔案的 `100644` / `100755` 權限。

### 🚥 v0.3 - 三向狀態比對系統 (Status & The Index)
- **Git 原理**: Git 的運作有三塊地盤：`工作目錄 (Workspace)`、準備加入下次 Commit 的緩衝區 `暫存區 (Index)` 以及當前指標的快照 `HEAD`。`git status` 會算出這三者的交集與差異。
- **git4 的實現**:
  - 打造了強大的記憶體比對引擎。
  - 將 HEAD (Tree 遞迴解析)、Index (.git4/index 快取檔) 與 Workspace 內所有的檔案 Hash 收集起來。
  - 取 Index 與 HEAD 的差集得出 `to be committed`。
  - 取 Workspace 與 (Index/HEAD) 的差集得出 `not staged for commit` 與 `untracked` 新檔案。

### 🔍 v0.4 - 變更比對與合併 (Diff & Fast-Forward Merge)
- **Git 原理**: `diff` 會利用 Myers 演算法找出兩段文字最少的修改路線；而 `merge` 合併時，若目前 HEAD 為目標分支的最直系祖先，即可進行無痛的「快轉 (Fast-forward)」。
- **git4 的實現**:
  - 引入了 Rust 強大的 `similar` 套件做到逐行比對，抽出舊 Hash 物件的文字跟工作目錄的最新檔案產出 `+` 與 `-` 報表。
  - 開發了 `is_ancestor` 演算法，沿著 Commit 的 `parent` 逐個向上爬梳族譜，當確認安全後，直接用 `checkout` 將歷史強勢拉回目標版號，完成安全的 Fast-Forward。

### 📡 v0.5 - 模擬網路傳輸與遠端協作 (Clone, Push, Fetch)
- **Git 原理**: 分散式版本控制中，「推送進度」本質上就是把本機算好且遠端所缺乏的 Objects 拷貝過去對方的資料庫中，而「拉取」就是把對方的 Objects 拷貝過來，並透過 `refs/remotes/` 區隔遠端與本地指標。
- **git4 的實現**:
  - 實作出 `copy_dir_recursive` 複製器進行本地遠端 (Local Remote) 模擬。
  - `clone` 達成了對 `.git4` 目錄的完整對拷並銜接 `checkout` 展開檔案。
  - `push` 複製所有的 `objects` 去遠端，更新遠端的 HEAD；`fetch` 則拉回最新的 `objects`，並儲存對方的分支於 `.git4/refs/remotes/origin/`。

### 🌐 v0.6 - 真實網際網路通訊協定 (Smart HTTP & Pkt-Line)
- **Git 原理**: Git 在和 GitHub 之類真實伺服器連線時，並不是單純使用檔案下載。它發展了 Smart HTTP 協定，利用 `Pkt-Line Format`（以4位十六進制標明長度的封包）與伺服器互相對話取得能力聲明以及最新 Hash 指標。
- **git4 的實現**:
  - 使用純粹的同步 HTTP 請求套件 `ureq`。
  - 實作了專屬的二進位封包切割引擎，從 GitHub `/info/refs?service=git-upload-pack` 傳回的雜質串流中，精確地解析並對齊 `0000` 終止符與純十六進制長度，成功剝離出 GitHub Server 上的分支 Hash 與 Tags！

---

## 結語
透過這六個版本的疊代演進，`git4` 將看似複雜的「版本控制魔法」拆解成了一段段樸實無華的檔案讀寫、演算法遞迴以及網路字串剖析。希望這份文件能讓你深刻體會 Git「將簡單組件拼出強大系統」的核心哲學！
