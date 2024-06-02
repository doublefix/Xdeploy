# 使用docker-compose启动
docker-compose -f mysql/mysql8.yaml up
docker-compose -f mysql/mysql5.yaml up
# 使用nerdctl启动
nerdctl compose -f mysql/mysql8.yaml up
nerdctl compose -f mysql/mysql5.yaml up