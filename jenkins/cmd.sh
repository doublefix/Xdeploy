# 创建应用安装的文件夹
mkdir -p /apps/jenkins

# 添加环境变量
vim ~/.bash_profile
vim ~/.zshrc
export GENKINS_HOME=/apps/jenkins

# 使用docker-compose启动
docker-compose -f jenkins/jenkins.yaml up
# 使用nerdctl启动
nerdctl compose -f jenkins/jenkins.yaml up