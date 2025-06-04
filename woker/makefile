PACKAGE_NAME=woker
VERSION=0.0.1
ARCH=amd64
BUILD_DIR=build
DEB_ROOT=$(BUILD_DIR)/$(PACKAGE_NAME)_$(VERSION)
BINARY_PATH=target/release/$(PACKAGE_NAME)
INSTALL_BIN=$(DEB_ROOT)/usr/local/bin

all: build-deb

build-deb:
	@echo "==> Building Rust binary..."
	cargo build --release

	@echo "==> Preparing directories..."
	rm -rf $(DEB_ROOT)
	mkdir -p $(DEB_ROOT)/DEBIAN
	mkdir -p $(INSTALL_BIN)

	@echo "==> Copying binary..."
	cp $(BINARY_PATH) $(INSTALL_BIN)/
	chmod 755 $(INSTALL_BIN)/$(PACKAGE_NAME)

	@echo "==> Creating control file..."
	echo "Package: $(PACKAGE_NAME)" > $(DEB_ROOT)/DEBIAN/control
	echo "Version: $(VERSION)" >> $(DEB_ROOT)/DEBIAN/control
	echo "Section: base" >> $(DEB_ROOT)/DEBIAN/control
	echo "Priority: optional" >> $(DEB_ROOT)/DEBIAN/control
	echo "Architecture: $(ARCH)" >> $(DEB_ROOT)/DEBIAN/control
	echo "Maintainer: chess <chess@chess.com>" >> $(DEB_ROOT)/DEBIAN/control
	echo "Description: $(PACKAGE_NAME) - packaged Rust app." >> $(DEB_ROOT)/DEBIAN/control

	@echo "==> Building .deb package..."
	dpkg-deb --build $(DEB_ROOT)
	mv $(DEB_ROOT).deb $(BUILD_DIR)/$(PACKAGE_NAME)_$(VERSION)_$(ARCH).deb

clean:
	rm -rf $(BUILD_DIR)
	cargo clean