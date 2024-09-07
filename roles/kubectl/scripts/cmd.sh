# 普通用户使用需要添加
if [ -d /etc/profile.d ]; then
    for file in /etc/profile.d/*.sh; do
        [ -r "$file" ] && . "$file"
    done
fi