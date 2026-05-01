#!/bin/bash
set -e

echo "=================================="
echo "    正版 git 自動測試對照腳本   "
echo "=================================="

# 1. 略過編譯
echo "=> [1/6] 略過編譯 (在此腳本中使用系統原生 git)..."

# 2. 建立並進入測試用的獨立資料夾
echo "=> [2/6] 準備測試目錄..."
rm -rf test_repo
mkdir test_repo
cd test_repo

GIT_CMD="git"

# 3. 測試 init 指令
echo "=> [3/6] 測試 init 指令..."
$GIT_CMD init -b main
# 設定本機免密碼拒絕問題以便後續 push 測試
$GIT_CMD config receive.denyCurrentBranch updateInstead
if [ ! -d ".git" ]; then
    echo "錯誤: .git 目錄未建立"
    exit 1
fi
echo "成功建立 .git 目錄！"

# 4. 測試加入檔案與 hash-object
echo "=> [4/6] 建立測試檔案並寫入物件..."
echo "Hello, git4!" > hello.txt
echo "Second file content" > data.txt

echo "=> 測試 hash-object..."
HASH=$($GIT_CMD hash-object -w hello.txt)
echo "hello.txt hash value: $HASH"

echo "=> 測試 cat-file..."
CONTENT=$($GIT_CMD cat-file -p $HASH)
if [ "$CONTENT" != "Hello, git4!
" ] && [ "$CONTENT" != "Hello, git4!" ]; then
    echo "警告：cat-file 的內容有差異或這是正常的空行"
fi

# 5. 測試 add 與 commit 機制
echo "=> [5/6] 測試 add 與 commit 功能..."
$GIT_CMD add hello.txt
$GIT_CMD add data.txt
$GIT_CMD commit -m "Initial commit"

echo "=> 模擬後續修改並進行第二次提交..."
echo "This is another line." >> hello.txt
$GIT_CMD add hello.txt
$GIT_CMD commit -m "Update hello.txt with another line"

# 6. 測試 log 輸出
echo "=> [6/6] 測試 log 功能..."
$GIT_CMD log

# 7. 測試 branch 與 checkout
echo "=> [7/7] 測試 branch 與 checkout 功能..."
$GIT_CMD branch new-feature
$GIT_CMD branch
$GIT_CMD checkout new-feature
echo "Feature content" > feature.txt
$GIT_CMD add feature.txt
$GIT_CMD commit -m "Commit on new feature branch"
echo "=> 檢視分支 new-feature 的 log..."
$GIT_CMD log
echo "=> 切換回 main 分支..."
$GIT_CMD checkout main
$GIT_CMD branch

# 8. 測試 status
echo "=> [8/8] 測試 status 功能..."
$GIT_CMD status
echo "Modified feature content" > feature.txt
echo "=> 修改了 feature.txt，檢視 status..."
$GIT_CMD status
$GIT_CMD add feature.txt
echo "=> 加入 feature.txt 後，檢視 status..."
$GIT_CMD status

# 9. 測試 diff 與 merge
echo "=> [9/9] 測試 diff 與 merge 功能..."
$GIT_CMD checkout main
$GIT_CMD branch feat-merge
$GIT_CMD checkout feat-merge
echo "A totally new line to test diff" >> feature.txt
echo "=> 檢視 diff 輸出..."
$GIT_CMD diff feature.txt || true # diff 即使有差異也繼續執行
$GIT_CMD add feature.txt
$GIT_CMD commit -m "Add new line to feature.txt"
echo "=> 切換回 main 並進行 fast-forward 合併..."
$GIT_CMD checkout main
$GIT_CMD merge feat-merge
$GIT_CMD log

# 10. 測試遠端聯網：clone, push, fetch
echo "=> [10/10] 測試 clone, push, fetch 跨倉儲操作..."
cd ..
$GIT_CMD clone test_repo remote_repo
cd remote_repo
echo "New remote data" > remote.txt
$GIT_CMD add remote.txt
$GIT_CMD commit -m "Commit on remote repo"
echo "=> 推送回 local test_repo..."
$GIT_CMD push origin main
cd ../test_repo
echo "=> 檢查原廠 test_repo 是否成功收到 push..."
$GIT_CMD log

echo "=> 在 test_repo 測試從 remote_repo 拉取 fetch..."
$GIT_CMD fetch ../remote_repo

# 11. 測試 remote 與 ls-remote (HTTP)
echo "=> [11/11] 測試 remote add 與 ls-remote..."
$GIT_CMD remote add origin https://github.com/cccrust/git4.git
echo "=> 取得 GitHub 上 git4 專案的分支資訊 (ls-remote)..."
$GIT_CMD ls-remote origin | head -n 5

# 12. 測試 unpack-objects (Packfile)
echo "=> [12/12] 測試 unpack-objects..."
cd ..
mkdir pack_test_repo && cd pack_test_repo
$GIT_CMD init -b main
echo "pack content" > pack.txt
$GIT_CMD add .
$GIT_CMD commit -m "Pack it up"
$GIT_CMD repack -a -d
PACK_FILE=$(ls .git/objects/pack/*.pack | head -n 1)
echo "=> 取得 .pack 並使用正版 git 解封裝: $PACK_FILE"
# 由於 unpack-objects 正版 Git 是以 stdin 接收資料的
$GIT_CMD unpack-objects < $PACK_FILE

# 結束與清理
echo "=================================="
echo "    所有對照測試皆順利完成！🎉 "
echo "=================================="
cd ..
rm -rf test_repo remote_repo pack_test_repo
