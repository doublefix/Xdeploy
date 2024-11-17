TAG ?= latest
HUB ?= docker.io
IMAGE_NAME = xdeploy
OSARCH=$(shell uname -m)
GIT_COMMIT=$(git rev-parse --short HEAD)-$(date +%Y%m%d)

ifneq (,$(findstring arm,$(OSARCH)))
HUB=docker.io
endif

.PHONY: image
image:
	@if command -v docker >/dev/null 2>&1; then \
	    docker build --build-arg GIT_COMMIT=$(GIT_COMMIT) -t "$(HUB)/$(IMAGE_NAME):$(TAG)" -t "$(HUB)/$(IMAGE_NAME):$(GIT_COMMIT)" . ;\
		echo "Built: $(HUB)/$(IMAGE_NAME):$(TAG) Commit: $(GIT_COMMIT)"; \
	elif command -v nerdctl >/dev/null 2>&1; then \
		nerdctl build --build-arg GIT_COMMIT=$(GIT_COMMIT) -t "$(HUB)/$(IMAGE_NAME):$(TAG)" -t "$(HUB)/$(IMAGE_NAME):$(GIT_COMMIT)" . ;\
		echo "Built: $(HUB)/$(IMAGE_NAME):$(TAG) Commit: $(GIT_COMMIT)"; \
	else \
		echo "Please install Nerdctl or Docker."; \
	fi