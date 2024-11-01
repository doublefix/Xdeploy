# 下载
wget https://nginx.org/download/nginx-1.24.0.tar.gz
wget https://gitee.com/mahqi/pubin/releases/download/cla/nginx-1.24.0.tar.gz

# 安装编译依赖
sudo apt install libpcre3 libpcre3-dev libssl-dev

# 一般编译
./configure \
--prefix=/apps/nginx

# 部分实用功能编译
./configure \
--prefix=$HOME/nginx \
--with-http_stub_status_module \
--with-http_ssl_module \
--with-http_realip_module \
--with-http_gzip_static_module \
--with-stream \
--user=$USER \
--group=$USER

# 安装
make && make install

# 开启端口
sudo setcap 'cap_net_bind_service=+ep' /home/username/nginx/sbin/nginx

# 启动
$HOME/nginx/sbin/nginx
$HOME/nginx/sbin/nginx -s stop
$HOME/nginx/sbin/nginx -s reload
ps aux | grep nginx
