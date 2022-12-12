GO              := GO111MODULE=on go
GOBUILD         := $(GO) build
GOTEST          := $(GO) test

PACKAGE_LIST  := go list ./...
PACKAGES  := $$($(PACKAGE_LIST))
PACKAGE_DIRECTORIES := $(PACKAGE_LIST) | sed 's|github.com/ti-community-infra/$(PROJECT)/||'
FILES     := $$(find $$($(PACKAGE_DIRECTORIES)) -name "*.go")

.PHONY: clean test cover fmt tidy staticcheck dev check label-dumpling-checks

staticcheck: golangci-lint
	golangci-lint run  $$($(PACKAGE_DIRECTORIES)) --timeout 500s

golangci-lint:
	$(GO) install -v github.com/golangci/golangci-lint/cmd/golangci-lint@v1.46.1
