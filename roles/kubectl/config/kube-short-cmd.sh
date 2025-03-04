# K8S Shortcut Command
K8S="kubectl"
NAMESPACE="default"

function kep() {
        pod=$1
        if [ $# -gt 1 ]; then
                container=$2
                $K8S exec -it $pod -n $NAMESPACE --container $container -- /bin/bash ||
                        $K8S exec -it $pod -n $NAMESPACE --container $container -- /bin/sh
        else
                $K8S exec -it $pod -n $NAMESPACE -- /bin/bash ||
                        $K8S exec -it $pod -n $NAMESPACE -- /bin/sh
        fi
}

function kebash() {
    pod=$1
    if [ $# -gt 1 ]; then
        container=$2
        $K8S exec -it $pod -n $NAMESPACE --container $container -- /bin/bash
    else
        $K8S exec -it $pod -n $NAMESPACE -- /bin/bash
    fi
}

function kezsh() {
    pod=$1
    if [ $# -gt 1 ]; then
        container=$2
        $K8S exec -it $pod -n $NAMESPACE --container $container -- /bin/zsh
    else
        $K8S exec -it $pod -n $NAMESPACE -- /bin/zsh
    fi
}

function kesh() {
    pod=$1
    if [ $# -gt 1 ]; then
        container=$2
        $K8S exec -it $pod -n $NAMESPACE --container $container -- /bin/sh
    else
        $K8S exec -it $pod -n $NAMESPACE -- /bin/sh
    fi
}

#  kubectl api-resources
function ka() {
        $K8S api-resources
}

# GET
function kgp() {
        $K8S get pod -n $NAMESPACE -o wide
}
function kgq() {
        $K8S get queue
}
function kgrc() {
        $K8S get rayclusters -owide -n $NAMESPACE
}
function kgrj() {
        $K8S get rayjobs -owide -n $NAMESPACE
}
function kgrs() {
        $K8S get rayservices -owide -n $NAMESPACE
}
function kgvj() {
        $K8S get vj -n $NAMESPACE
}
function kgpg() {
        $K8S get pg -n $NAMESPACE
}
function kgs() {
        $K8S get service -n $NAMESPACE -o wide
}
function kgvs() {
        $K8S get VirtualService -n $NAMESPACE
}
function kgsm() {
        $K8S get serviceMonitor -n $NAMESPACE
}
function kgra() {
        $K8S get RequestAuthentication -n $NAMESPACE
}
function kgap() {
        $K8S get AuthorizationPolicy -n $NAMESPACE
}
function kgj() {
        $K8S get job -n $NAMESPACE -o wide
}
function kge() {
        $K8S get event -n $NAMESPACE -o wide
}
function kgi() {
        $K8S get ingress -n $NAMESPACE -o wide
}
function kgcm() {
        $K8S get configmap -n $NAMESPACE -o wide
}
function kgd() {
        $K8S get deployment -n $NAMESPACE -o wide
}
function kgr() {
        $K8S get replicaset -n $NAMESPACE -o wide
}
function kgds() {
        $K8S get daemonset -n $NAMESPACE -o wide
}
function kgss() {
        $K8S get statefulset -n $NAMESPACE -o wide
}
function kggw() {
        $K8S get gateway -n $NAMESPACE
}
function kgns() {
        $K8S get ns
}
function kgpvc() {
        $K8S get pvc -owide
}
function kgpv() {
        $K8S get pv -owide
}
function kgcm() {
        $K8S get configmaps -n $NAMESPACE -o wide
}
# kubectl get node -owide
function kgn() {
        $K8S get node -owide
}
# kubectl get node debian -oyaml
function kyn() {
        $K8S get node $@ -oyaml
}
# kubectl describe node rocky
function kdn() {
        $K8S describe node $@
}
# kubectl top no
function ktn() {
        $K8S top no $@
}
# 查看某个命名空间下的资源占用
# kubectl top pods
function ktp() {
        $K8S top po $@
}

# Edit
function kes() {
        $K8S -n $NAMESPACE edit service $@
}
# kubectl edit queue
function keq() {
        $K8S edit queue $@
}
function kyq() {
        $K8S get queue $@ -oyaml
}
function kevj() {
        $K8S -n $NAMESPACE edit vj $@
}
function kepg() {
        $K8S -n $NAMESPACE edit pg $@
}
function kei() {
        $K8S -n $NAMESPACE edit ingress $@
}
function kec() {
        $K8S -n $NAMESPACE edit configmap $@
}
function ked() {
        $K8S -n $NAMESPACE edit deployment $@
}

# Delete
function krp() {
        $K8S delete pod $1 -n $NAMESPACE
}
function krs() {
        $K8S delete service $1 -n $NAMESPACE
}
function krrc() {
        $K8S delete rayclusters $1 -n $NAMESPACE
}
function krrj() {
        $K8S delete rayjobs $1 -n $NAMESPACE
}
function krrs() {
        $K8S delete rayservices $1 -n $NAMESPACE
}
function kri() {
        $K8S delete ingress $1 -n $NAMESPACE
}
function krcm() {
        $K8S delete configmap $1 -n $NAMESPACE
}
function krd() {
        $K8S delete deployment $1 -n $NAMESPACE
}

# Desc
function kdp() {
        $K8S describe pod $1 -n $NAMESPACE
}
function kdq() {
        $K8S describe queue $1
}
function kdvj() {
        $K8S describe vj $1 -n $NAMESPACE
}
function kdrc() {
        $K8S describe rayclusters $1 -n $NAMESPACE
}
function kdrj() {
        $K8S describe rayjobs $1 -n $NAMESPACE
}
function kdrs() {
        $K8S describe rayservices $1 -n $NAMESPACE
}
function kdvj() {
        $K8S describe pg $1 -n $NAMESPACE
}
function kds() {
        $K8S describe service $1 -n $NAMESPACE
}
function kdi() {
        $K8S describe ingress $1 -n $NAMESPACE
}
function kdcm() {
        $K8S describe configmap $1 -n $NAMESPACE
}
function kdd() {
        $K8S describe deployment $1 -n $NAMESPACE
}

# Helm
function hhl() {
        helm --namespace $NAMESPACE list $@
}
function hhi() {
        helm --namespace $NAMESPACE install $@
}
function hhu() {
        helm --namespace $NAMESPACE uninstall $@
}
function hhg() {
        helm --namespace $NAMESPACE upgrade $@
}

# Tools
function set-ns() {
        NAMESPACE=${1:-default}
        export NAMESPACE
        kubectl config set-context --current --namespace=$NAMESPACE
}

function change_ns() {
    NAMESPACE=${1:-default}
    export NAMESPACE
    kubectl config set-context --current --namespace=$NAMESPACE
}

function sns() {
        echo $NAMESPACE
}

function kccc() {
    kubectl config current-context
}

# kubectl config view --minify --flatten
# Cluster
function kgc() {
    kubectl config get-clusters
}

function krc() {
    if [ -z "$1" ]; then
        echo "请提供集群名称作为参数"
        return 1
    fi

    local NAME=$1

    read -p "请再次输入集群名称 '$NAME' 以确认删除: " CONFIRM_NAME

    if [ "$CONFIRM_NAME" = "$NAME" ]; then
        if kubectl config delete-cluster "$NAME"; then
            echo "集群 '$NAME' 已成功删除。"
        else
            echo "删除集群 '$NAME' 失败。"
            return 1
        fi
    else
        echo "删除操作已取消，输入的集群名称不匹配。"
        return 1
    fi
}
# User
function kgu() {
    kubectl config get-users
}

function kru() {
    if [ -z "$1" ]; then
        echo "请提供用户名称作为参数"
        return 1
    fi

    local NAME=$1

    read -p "请再次输入用户名称 '$NAME' 以确认删除: " CONFIRM_NAME

    if [ "$CONFIRM_NAME" = "$NAME" ]; then
        if kubectl config delete-user "$NAME"; then
            echo "用户 '$NAME' 已成功删除。"
        else
            echo "删除用户 '$NAME' 失败。"
            return 1
        fi
    else
        echo "删除操作已取消，输入的用户名称不匹配。"
        return 1
    fi
}

# Contexts
function kcgc() {
    kubectl config get-contexts
}

# kubectl config set-context kubernetes-admin@kubernetes-new --cluster=kubernetes --user=kubernetes-admin --namespace=kube-system
function kcnc() {
    if [ -z "$1" ] || [ -z "$2" ] || [ -z "$3" ] || [ -z "$4" ]; then
        echo "请提供上下文名称、集群名称、用户名称和命名空间！"
        return 1
    fi

    CONTEXT_NAME=$1
    CLUSTER_NAME=$2
    USER_NAME=$3
    NAMESPACE=$4
    
    kubectl config set-context $CONTEXT_NAME --cluster=$CLUSTER_NAME --user=$USER_NAME --namespace=$NAMESPACE
}
function kcuc() {
    if [ -z "$1" ]; then
        echo "请提供上下文名称作为参数！"
        return 1
    fi
    NAME=$1
    kubectl config use-context $NAME

    NAMESPACE=$(kubectl config view --minify --output 'jsonpath={.contexts[?(@.name=="'$NAME'")].context.namespace}')
    kubectl config set-context --current --namespace=$NAMESPACE
}

function kcrc() {
    if [ -z "$1" ]; then
        echo "请提供上下文名称作为参数！"
        return 1
    fi
    CONTEXT=$1
    kubectl config unset contexts.$CONTEXT
}


function kcp() {
        $K8S -n $NAMESPACE cp $@
}
function htest() {
        helm --namespace $NAMESPACE install htest $@ --debug --dry-run
}
function kdff() {
        $K8S delete pods $1 -n $NAMESPACE --grace-period=0 --force
}
function kaf() {
        $K8S apply -f $1
}
function kdf() {
        $K8S delete -f $1
}
function krf() {
        $K8S replace -f $1
}
function kl {
        $K8S -n $NAMESPACE logs $@
}
function clean_lost {
        for f in $(kgd | grep idp-nl | awk '{print $1}'); do krd $f & done
}

# sudo rm -rf /etc/profile.d/kube-short-cmd.sh
# sudo cp roles/kubectl/config/kube-short-cmd.sh /etc/profile.d/

