FROM fedora:42
LABEL dev.containers.features="fedora"

RUN dnf install -y git jq yq bash-completion wget2 \
  && dnf clean all \
  && rm -rf /var/cache/dnf