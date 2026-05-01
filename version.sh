git add -A
git commit -m "$1"
git push 
git tag $1
git push origin $1