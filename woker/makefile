PACKAGE_NAME=woker
VERSION=0.0.1
ARCH=amd64

BUILD_DIR=build
DEB_ROOT=$(BUILD_DIR)/$(PACKAGE_NAME)_$(VERSION)
BINARY_PATH=target/release/$(PACKAGE_NAME)
INSTALL_BIN=$(DEB_ROOT)/usr/local/bin
NERDCTL_ARCHIVE=nerdctl-full-2.1.2-linux-amd64.tar.gz
NERDCTL_DEST=$(DEB_ROOT)/usr/local

all: build-deb

build-deb:
	@echo "==> Building Rust binary..."
	cargo build --release

	@echo "==> Preparing directories..."
	rm -rf $(DEB_ROOT)
	mkdir -p $(DEB_ROOT)/DEBIAN
	mkdir -p $(INSTALL_BIN)
	mkdir -p $(NERDCTL_DEST)

	@echo "==> Copying Rust binary..."
	cp $(BINARY_PATH) $(INSTALL_BIN)/
	chmod 755 $(INSTALL_BIN)/$(PACKAGE_NAME)

	@echo "==> Extracting nerdctl to DEB root..."
	tar -xzf $(NERDCTL_ARCHIVE) -C $(NERDCTL_DEST)
	rm -rf $(DEB_ROOT)/usr/local/bin/build*
	rm -rf $(DEB_ROOT)/usr/local/bin/ctr*
	rm -rf $(DEB_ROOT)/usr/local/bin/nerdctl.gomodjail
	rm -rf $(DEB_ROOT)/usr/local/bin/gomodjail
	rm -rf $(DEB_ROOT)/usr/local/bin/ctd-decoder
	rm -rf $(DEB_ROOT)/usr/local/bin/bypass4netns*
	rm -rf $(DEB_ROOT)/usr/local/bin/containerd-stargz-grpc
	rm -rf $(DEB_ROOT)/usr/local/bin/stargz-store-helper
	rm -rf $(DEB_ROOT)/usr/local/bin/tini
	rm -rf $(DEB_ROOT)/usr/local/share
	rm -rf $(DEB_ROOT)/usr/local/lib/systemd/system/buildkit.service
	rm -rf $(DEB_ROOT)/usr/local/lib/systemd/system/stargz-snapshotter.service

	@echo "==> Creating postinst script..."
	echo "#!/bin/bash" > $(DEB_ROOT)/DEBIAN/postinst
	echo "set -e" >> $(DEB_ROOT)/DEBIAN/postinst
	echo "echo '==> Enabling and starting systemd services from /usr/local/lib/systemd/system'" >> $(DEB_ROOT)/DEBIAN/postinst
	echo "systemctl daemon-reexec || true" >> $(DEB_ROOT)/DEBIAN/postinst
	echo "systemctl daemon-reload || true" >> $(DEB_ROOT)/DEBIAN/postinst
	echo "systemctl enable containerd.service || true" >> $(DEB_ROOT)/DEBIAN/postinst
	echo "systemctl start containerd.service || true" >> $(DEB_ROOT)/DEBIAN/postinst
	chmod 755 $(DEB_ROOT)/DEBIAN/postinst

	@echo "==> Creating postrm script..."
	echo "#!/bin/bash" > $(DEB_ROOT)/DEBIAN/postrm
	echo "set -e" >> $(DEB_ROOT)/DEBIAN/postrm
	echo "systemctl stop containerd.service || true" >> $(DEB_ROOT)/DEBIAN/postrm
	echo "systemctl disable containerd.service || true" >> $(DEB_ROOT)/DEBIAN/postrm
	chmod 755 $(DEB_ROOT)/DEBIAN/postrm

	@echo "==> Creating control file..."
	echo "Package: $(PACKAGE_NAME)" > $(DEB_ROOT)/DEBIAN/control
	echo "Version: $(VERSION)" >> $(DEB_ROOT)/DEBIAN/control
	echo "Section: base" >> $(DEB_ROOT)/DEBIAN/control
	echo "Priority: optional" >> $(DEB_ROOT)/DEBIAN/control
	echo "Architecture: $(ARCH)" >> $(DEB_ROOT)/DEBIAN/control
	echo "Maintainer: Chess <chess@chess.com>" >> $(DEB_ROOT)/DEBIAN/control
	echo "Description: $(PACKAGE_NAME) - Rust app with bundled nerdctl." >> $(DEB_ROOT)/DEBIAN/control

	@echo "==> Building .deb package..."
	dpkg-deb --build $(DEB_ROOT)
	mv $(DEB_ROOT).deb $(BUILD_DIR)/$(PACKAGE_NAME)_$(VERSION)_linux_$(ARCH).deb

clean:
	rm -rf $(BUILD_DIR)
	cargo clean