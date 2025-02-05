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
function kgc() {
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
function kri() {
        $K8S delete ingress $1 -n $NAMESPACE
}
function krc() {
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
function kdvj() {
        $K8S describe pg $1 -n $NAMESPACE
}
function kds() {
        $K8S describe service $1 -n $NAMESPACE
}
function kdi() {
        $K8S describe ingress $1 -n $NAMESPACE
}
function kdc() {
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
        export NAMESPACE=$1
        kubectl config set-context --current --namespace=$NAMESPACE
}
function change-ns() {
    # Check if NAMESPACE is already set, if not, default to an empty string
    NAMESPACE=${NAMESPACE:-}

    echo "Current namespace: ${NAMESPACE:-default}"
    echo "-------------------------------------"
    echo "Available namespaces:"
    kubectl get ns
    echo "-------------------------------------"

    # Prompt user for input and read into NAMESPACE variable
    read -r "NAMESPACE?Enter the namespace you want to set as default (or press Enter to set 'default'): "

    # Default to 'default' if no input is provided
    if [ -z "$NAMESPACE" ]; then
        NAMESPACE="default"
    fi

    # Export the NAMESPACE variable and update kubectl context
    export NAMESPACE
    kubectl config set-context --current --namespace="$NAMESPACE"

    # Verify if the namespace was set correctly
    CURRENT_NAMESPACE=$(kubectl config view --minify --output 'jsonpath={..namespace}')
    if [ "$CURRENT_NAMESPACE" = "$NAMESPACE" ]; then
        echo "Namespace successfully set to '$NAMESPACE'."
    else
        echo "Failed to set namespace to '$NAMESPACE'."
    fi
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

