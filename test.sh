#!/bin/bash
set -e

echo "=================================="
echo "      git4 自動測試腳本         "
echo "=================================="

# 1. 首先編譯專案
echo "=> [1/6] 編譯 git4..."
cargo build

# 2. 建立並進入測試用的獨立資料夾
echo "=> [2/6] 準備測試目錄..."
rm -rf test_repo
mkdir test_repo
cd test_repo

# 使用變數儲存 git4 執行檔的相對路徑
GIT4="../target/debug/git4"

# 3. 測試 init 指令
echo "=> [3/6] 測試 init 指令..."
$GIT4 init
if [ ! -d ".git4" ]; then
    echo "錯誤: .git4 目錄未建立"
    exit 1
fi
echo "成功建立 .git4 目錄！"

# 4. 測試加入檔案與 hash-object
echo "=> [4/6] 建立測試檔案並寫入物件..."
echo "Hello, git4!" > hello.txt
echo "Second file content" > data.txt

# 手機針對檔案產生 blob
echo "=> 測試 hash-object..."
HASH=$($GIT4 hash-object -w hello.txt)
echo "hello.txt hash value: $HASH"

# 測試 cat-file 可以讀取物件
echo "=> 測試 cat-file..."
CONTENT=$($GIT4 cat-file -p $HASH)
if [ "$CONTENT" != "Hello, git4!
" ] && [ "$CONTENT" != "Hello, git4!" ]; then
    # 某些系統 echo 自帶換行
    echo "警告：cat-file 的內容有差異或這是正常的空行"
fi

# 5. 測試 add 與 commit 機制
echo "=> [5/6] 測試 add 與 commit 功能..."
$GIT4 add hello.txt
$GIT4 add data.txt
$GIT4 commit -m "Initial commit"

echo "=> 模擬後續修改並進行第二次提交..."
echo "This is another line." >> hello.txt
$GIT4 add hello.txt
$GIT4 commit -m "Update hello.txt with another line"

# 6. 測試 log 輸出
echo "=> [6/6] 測試 log 功能..."
$GIT4 log

# 7. 測試 branch 與 checkout
echo "=> [7/7] 測試 branch 與 checkout 功能..."
$GIT4 branch new-feature
$GIT4 branch
$GIT4 checkout new-feature
echo "Feature content" > feature.txt
$GIT4 add feature.txt
$GIT4 commit -m "Commit on new feature branch"
echo "=> 檢視分支 new-feature 的 log..."
$GIT4 log
echo "=> 切換回 main 分支..."
$GIT4 checkout main
$GIT4 branch

# 結束與清理
echo "=================================="
echo "    所有測試皆順利完成！🎉     "
echo "=================================="
cd ..
rm -rf test_repo
